use std::fmt::{Display, Formatter};
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MeteoraMarket, OrcaMarket, RaydiumMarket};
use crate::pools::lifinity::LifinityMarket;

pub struct PubkeyPair {
    pub pubkey_a: Pubkey,
    pub pubkey_b: Pubkey
}

impl PubkeyPair {
    pub fn any(&self, pubkey: Pubkey) -> bool {
        self.pubkey_a == pubkey || self.pubkey_b == pubkey
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
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

impl Display for Market {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("{}", self);
        write!(f, "{}", str)
    }
}

pub trait MarketSerializer {
    fn unpack_data(data: &Vec<u8>) -> Self;
}

pub trait MarketOperation {
    fn get_mint_pair(&self) -> PubkeyPair;
    fn get_pool_pair(&self) -> PubkeyPair;
    fn get_market_provider(&self) -> Market;
}

pub fn resolve_market_data(market: &Market, data: &Vec<u8>) -> Box<dyn MarketOperation> {
    match market {
        Market::ORCA => {
            Box::new(OrcaMarket::unpack_data(data))
        }
        Market::RAYDIUM => {
            Box::new(RaydiumMarket::unpack_data(data))
        }
        Market::METEORA => {
            Box::new(MeteoraMarket::unpack_data(data))
        }
        Market::LIFINITY => {
            Box::new(LifinityMarket::unpack_data(data))
        }
    }
}