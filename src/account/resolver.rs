use std::any::Any;
use solana_sdk::pubkey::Pubkey;

use crate::account::account::{AccountDataSerializer, DeserializedConfigAccount};
use crate::constants::RAYDIUM_CLMM_DATA_LEN;
use crate::constants::RAYDIUM_CLMM_PROGRAM_PUBKEY;
use crate::constants::RAYDIUM_OPEN_BOOK_PROGRAM_PUBKEY;
use crate::r#struct::market::{Market, PoolOperation};
use crate::r#struct::pools::{MeteoraClmmMarket, OrcaClmmAccount, OrcaClmmMarket, RaydiumClmmAccount, RaydiumClmmMarket, RaydiumOpenBookMarket, WhirlpoolsConfig, WhirlpoolsConfigAccount};
use crate::r#struct::pools::lifinity::LifinityMarket;

pub fn resolve_pool_account(market: &Market, data: &Vec<u8>) -> Box<dyn PoolOperation> {
    match market {
        Market::ORCA => {
            Box::new(OrcaClmmMarket::unpack_data(data))
        }
        Market::RAYDIUM => {
            if data.len() == RAYDIUM_CLMM_DATA_LEN {
                Box::new(RaydiumClmmMarket::unpack_data(data))
            }
            else {
                Box::new(RaydiumOpenBookMarket::unpack_data(data))
            }
        }
        Market::METEORA => {
            Box::new(MeteoraClmmMarket::unpack_data(data))
        }
        Market::LIFINITY => {
            Box::new(LifinityMarket::unpack_data(data))
        }
        _ => {
            panic!("unknown pool")
        }
    }
}

pub fn resolve_pool_config_account(market: &Market, owner_pubkey: &Pubkey, account_pubkey: Pubkey, data: &Vec<u8>) -> DeserializedConfigAccount {
    match market {
        Market::ORCA => {
            DeserializedConfigAccount::OrcaClmmConfigAccount(
                OrcaClmmAccount::resolve_account(account_pubkey, data)
            )
        }
        Market::RAYDIUM => {
            match owner_pubkey.to_string().as_str() {
                RAYDIUM_CLMM_PROGRAM_PUBKEY => {
                    DeserializedConfigAccount::RaydiumClmmConfigAccount(
                        RaydiumClmmAccount::resolve_account(account_pubkey, data)
                    )
                }
                RAYDIUM_OPEN_BOOK_PROGRAM_PUBKEY => {
                    panic!("unknown account: RaydiumCpmmAccount")
                }
                _ => {
                    DeserializedConfigAccount::EmptyConfigAccount
                }
            }
        }
        Market::METEORA => {
            todo!()
        }
        Market::LIFINITY => {
            todo!()
        }
        _ => {
            todo!()
        }
    }
}

pub fn resolve_token_data() {}