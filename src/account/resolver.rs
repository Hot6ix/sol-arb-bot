use std::sync::Arc;
use crate::account::account::AccountDataSerializer;
use crate::constants::RAYDIUM_CLMM_DATA_LEN;
use crate::r#struct::market::{Market, PoolOperation};
use crate::r#struct::pools::{MeteoraClmmMarket, OrcaClmmMarket, RaydiumClmmMarket, RaydiumCpmmMarket};
use crate::r#struct::pools::lifinity::LifinityMarket;

pub fn resolve_market_data(market: &Market, data: &Vec<u8>) -> Arc<dyn PoolOperation> {
    match market {
        Market::ORCA => {
            Arc::new(OrcaClmmMarket::unpack_data(data))
        }
        Market::RAYDIUM => {
            if data.len() == RAYDIUM_CLMM_DATA_LEN {
                Arc::new(RaydiumClmmMarket::unpack_data(data))
            }
            else {
                Arc::new(RaydiumCpmmMarket::unpack_data(data))
            }
        }
        Market::METEORA => {
            Arc::new(MeteoraClmmMarket::unpack_data(data))
        }
        Market::LIFINITY => {
            Arc::new(LifinityMarket::unpack_data(data))
        }
    }
}

pub fn resolve_token_data() {}