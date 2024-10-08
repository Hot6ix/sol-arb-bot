use std::any::Any;
use std::fmt::{Debug, Display};

use dyn_clone::DynClone;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use crate::r#struct::account::{DeserializedAccount};
use crate::formula::base::Formula;
use crate::utils::PubkeyPair;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Default, Deserialize)]
pub enum Market {
    ORCA,
    RAYDIUM,
    METEORA,
    LIFINITY,
    #[default]
    UNKNOWN
}

impl Market {
    pub fn from(market: &Market) -> Market {
        match market {
            Market::ORCA => Market::ORCA,
            Market::RAYDIUM => Market::RAYDIUM,
            Market::METEORA => Market::METEORA,
            Market::LIFINITY => Market::LIFINITY,
            Market::UNKNOWN => Market::UNKNOWN,
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
            Market::UNKNOWN => String::from("UNKNOWN"),
        }
    }


}

pub trait PoolOperation: DynClone + Sync + Send {
    fn get_mint_pair(&self) -> PubkeyPair;
    fn get_pool_pair(&self) -> PubkeyPair;
    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)>;
    fn get_formula(&self) -> Formula;
    fn swap(&self, accounts: &Vec<DeserializedAccount>);
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn PoolOperation> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl Default for Box<dyn PoolOperation> {
    fn default() -> Self {
        Box::new(<DefaultMarket as Default>::default())
    }
}

pub trait AccountResolver {
    fn resolve_account<T: DeserializeOwned>(account: &Account) -> T;
}

#[derive(Copy, Clone, Default)]
struct DefaultMarket {}

impl PoolOperation for DefaultMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair::default()
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair::default()
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        Vec::default()
    }

    fn get_formula(&self) -> Formula {
        Formula::default()
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}