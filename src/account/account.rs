use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use crate::formula::base::Formula;

use crate::formula::clmm::constant::TICK_ARRAY_SEED;
use crate::formula::clmm::tick_array::{TickArrayBitmapExtension, TickArrayBitmapExtensionAccount, TickArrayState, TickArrayStateAccount};
use crate::r#struct::market::{Market, PoolOperation};
use crate::r#struct::pools::{OrcaClmmAccount, RaydiumClmmAccount, RaydiumClmmMarket};
use crate::r#struct::token::TokenAccount;

#[derive(Clone)]
pub enum DeserializedAccount {
    Account(DeserializedDataAccount),
    PoolAccount(DeserializedPoolAccount),
    TokenAccount(DeserializedTokenAccount),
    ConfigAccount(DeserializedConfigAccount)
}

impl DeserializedAccount {
    pub fn get_pubkey(&self) -> Pubkey {
        match self {
            DeserializedAccount::Account(account) => {
                account.pubkey
            }
            DeserializedAccount::PoolAccount(account) => {
                account.pubkey
            }
            DeserializedAccount::TokenAccount(account) => {
                account.pubkey
            }
            DeserializedAccount::ConfigAccount(account) => {
                account.get_pubkey()
            }
        }
    }

    pub fn get_market(&self) -> Market {
        match self {
            DeserializedAccount::Account(account) => {
                account.market
            }
            DeserializedAccount::PoolAccount(account) => {
                account.market
            }
            DeserializedAccount::TokenAccount(account) => {
                account.market
            }
            DeserializedAccount::ConfigAccount(account) => {
                account.get_market()
            }
        }
    }
}



#[derive(Clone, Default, PartialEq)]
pub enum DeserializedConfigAccount {
    RaydiumClmmConfigAccount(RaydiumClmmAccount),
    OrcaClmmConfigAccount(OrcaClmmAccount),
    #[default]
    EmptyConfigAccount
}

impl DeserializedConfigAccount {
    pub fn get_pubkey(&self) -> Pubkey {
        match self {
            DeserializedConfigAccount::RaydiumClmmConfigAccount(account) => {
                account.get_pubkey()
            }
            DeserializedConfigAccount::OrcaClmmConfigAccount(account) => {
                account.get_pubkey()
            }
            _ => {
                Pubkey::default()
            }
        }
    }

    pub fn get_market(&self) -> Market {
        match self {
            DeserializedConfigAccount::RaydiumClmmConfigAccount(account) => {
                account.get_market()
            }
            DeserializedConfigAccount::OrcaClmmConfigAccount(account) => {
                account.get_market()
            }
            _ => {
                Market::UNKNOWN
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct DeserializedPoolAccount {
    pub pubkey: Pubkey,
    pub account: Account,
    pub market: Market,
    pub operation: Box<dyn PoolOperation>
}

impl DeserializedPoolAccount {
    pub fn get_swap_related_pubkeys(&self, rpc_client: Option<&RpcClient>) -> Result<Vec<(DeserializedAccount, Pubkey)>, &'static str> {
        match self.market {
            Market::ORCA | Market::METEORA | Market::LIFINITY => {
                let mut vec = vec![
                    (DeserializedAccount::PoolAccount(DeserializedPoolAccount::default()), self.pubkey)
                ];
                vec.append(&mut self.operation.get_swap_related_pubkeys());

                Ok(vec)
            }
            Market::RAYDIUM => {
                let mut vec = vec![
                    (DeserializedAccount::PoolAccount(DeserializedPoolAccount::default()), self.pubkey)
                ];
                vec.append(&mut self.operation.get_swap_related_pubkeys());

                // since this step does not know swap direction, find both ways of tick array states pubkeys
                if self.operation.get_formula() == Formula::ConcentratedLiquidity {
                    // get tick array states
                    let tick_array_bitmap_extension_pubkey = TickArrayBitmapExtension::key(&self.account.owner, &self.pubkey).expect("failed to get tick_array_bitmap_extension pubkey");
                    let tick_array_bitmap_extension_account = rpc_client.unwrap().get_account(&tick_array_bitmap_extension_pubkey).expect("failed to fetch tick_array_bitmap_extension");
                    let tick_array_bitmap_extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_account.data);
                    vec.push((DeserializedAccount::ConfigAccount(DeserializedConfigAccount::RaydiumClmmConfigAccount(RaydiumClmmAccount::TickArrayBitmapExtension(TickArrayBitmapExtensionAccount::default()))), tick_array_bitmap_extension_pubkey));

                    let market = self.operation.as_any().downcast_ref::<RaydiumClmmMarket>().expect("failed to downcast");

                    for i in 0..2 {
                        let zero_for_one = if i % 2 == 0 { true } else { false };

                        let (_, mut current_valid_tick_array_start_index) = market.get_first_initialized_tick_array(
                            &Some(&tick_array_bitmap_extension), true
                        ).unwrap();
                        let current_tick_array_state = TickArrayState::key(
                            &self.account.owner,
                            &[
                                &TICK_ARRAY_SEED.as_bytes(),
                                &self.pubkey.as_ref(),
                                &current_valid_tick_array_start_index.to_be_bytes()
                            ]
                        ).expect("failed to get current_tick_array_state");
                        vec.push((DeserializedAccount::ConfigAccount(DeserializedConfigAccount::RaydiumClmmConfigAccount(RaydiumClmmAccount::TickArrayState(TickArrayStateAccount::default()))), current_tick_array_state));

                        for _ in 0..5 {
                            let next_tick_array_index = market.next_initialized_tick_array_start_index(
                                &Some(&tick_array_bitmap_extension),
                                current_valid_tick_array_start_index,
                                zero_for_one
                            ).expect("failed to get next_tick_array_index");

                            if next_tick_array_index.is_none() {
                                break;
                            }
                            current_valid_tick_array_start_index = next_tick_array_index.unwrap();
                            let tick_array_state = TickArrayState::key(
                                &self.account.owner,
                                &[
                                    &TICK_ARRAY_SEED.as_bytes(),
                                    &self.pubkey.as_ref(),
                                    &current_valid_tick_array_start_index.to_be_bytes()
                                ]
                            ).expect("failed to get tick_array_state");
                            // todo tick_array_state array does not need to have 5 items
                            vec.push((DeserializedAccount::ConfigAccount(DeserializedConfigAccount::RaydiumClmmConfigAccount(RaydiumClmmAccount::TickArrayState(TickArrayStateAccount::default()))), tick_array_state));
                        }
                    }
                }

                Ok(vec)
            }
            Market::UNKNOWN => { Err("unknown market") }
        }
    }

    pub fn equals(&self, to: &DeserializedPoolAccount) -> bool {
        self.pubkey == to.pubkey
    }
}

#[derive(Clone, Default)]
pub struct DeserializedDataAccount {
    pub pubkey: Pubkey,
    pub account: Account,
    pub market: Market,
}

#[derive(Clone, Default)]
pub struct DeserializedTokenAccount {
    pub pubkey: Pubkey,
    pub account: Account,
    pub token: TokenAccount,
    pub market: Market,
}

impl DeserializedTokenAccount {
    pub fn get_amount(&self) -> u64 {
        self.token.amount
    }
}

pub trait AccountDataSerializer {
    fn unpack_data(data: &Vec<u8>) -> Self;
}