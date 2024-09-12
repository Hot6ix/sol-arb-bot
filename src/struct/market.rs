use std::fmt::{Debug, Display};

use solana_sdk::pubkey::Pubkey;
use crate::formula::base::Formula;
use crate::r#struct::pools::{OrcaClmmAccount, RaydiumClmmAccount};
use crate::utils::PubkeyPair;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Market {
    ORCA,
    RAYDIUM,
    METEORA,
    LIFINITY
}

impl Market {
    pub fn from(market: &Market) -> Market {
        match market {
            Market::ORCA => Market::ORCA,
            Market::RAYDIUM => Market::RAYDIUM,
            Market::METEORA => Market::METEORA,
            Market::LIFINITY => Market::LIFINITY,
        }
    }
}

impl Market {
    pub fn name(&self) -> String {
        match self {
            Market::ORCA => String::from("ORCA"),
            Market::RAYDIUM => String::from("RAYDIUM"),
            Market::METEORA => String::from("METEORA"),
            Market::LIFINITY => String::from("LIFINITY"),
        }
    }


}

pub trait PoolOperation: Sync + Send {
    fn get_mint_pair(&self) -> PubkeyPair;
    fn get_pool_pair(&self) -> PubkeyPair;
    fn get_swap_related_pubkeys(&self) -> Vec<(String, Pubkey)>;
    fn get_formula(&self) -> Formula;
}

pub trait AccountResolver {
    fn resolve_account<T>(data: &Vec<u8>) -> T;
}

pub enum ConfigAccount {
    RaydiumClmmConfigAccount(RaydiumClmmAccount),
    OrcaClmmConfigAccount(OrcaClmmAccount)
}