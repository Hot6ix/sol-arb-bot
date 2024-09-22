use std::any::Any;
use dyn_clone::DynClone;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use crate::r#struct::market::{Market, PoolOperation};
use crate::r#struct::pools::{OrcaClmmAccount, RaydiumClmmAccount};
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
    pub fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        match self.market {
            Market::ORCA | Market::RAYDIUM | Market::METEORA | Market::LIFINITY => {
                let mut vec = vec![
                    (DeserializedAccount::PoolAccount(DeserializedPoolAccount::default()), self.pubkey)
                ];
                vec.append(&mut self.operation.get_swap_related_pubkeys());

                vec
            }
            Market::UNKNOWN => { vec![] }
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