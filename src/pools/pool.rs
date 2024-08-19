use solana_sdk::pubkey::Pubkey;
use crate::pools::{MeteoraMarket, OrcaMarket, RaydiumMarket};
use crate::pools::lifinity::LifinityMarket;

pub struct PubkeyPair {
    pub pubkey_a: Pubkey,
    pub pubkey_b: Pubkey
}

pub enum Market {
    ORCA,
    RAYDIUM,
    METEORA,
    LIFINITY
}

pub trait MarketSerializer {
    fn unpack_data(data: &Vec<u8>) -> Self;
}

pub trait MarketOperation {
    fn get_mint_pair(&self) -> PubkeyPair;
    fn get_pool_pair(&self) -> PubkeyPair;
}

pub fn resolve_market_data(market: Market, data: &Vec<u8>) -> Box<dyn MarketOperation> {
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