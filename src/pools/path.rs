use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use crate::pools::Market;
use crate::token::{MarketPool, TrackedAccount};

pub struct PathFinder {
    pub market_accounts: Arc<Mutex<MarketPool>>
}

impl PathFinder {
    pub fn a(depth: i32, mut path_map: HashMap<i32, Vec<HashSet<(Market, usize)>>>, start_mint: Pubkey, data: HashMap<Market, Vec<TrackedAccount>>) {
        data.iter().for_each(|market| {
            let index = market.1.iter().position(|account| {
                account.get_market_operation().get_mint_pair().any(start_mint)
            });

            if index.is_some() {
                let path = path_map.get(&depth);
                if path.is_none() {
                    // add to path_map
                }
                else {

                }
            }
        })
    }
    pub fn find_path(&self, start_mint: Pubkey) {
        let mut path_map: HashMap<i32, Vec<HashSet<(Market, usize)>>> = HashMap::new();
        for i in 2..5 {
            let depth = i;
            path_map.insert(depth, Vec::new());
        }
    }
}