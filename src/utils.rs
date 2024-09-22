use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use crate::r#struct::market::Market;

#[derive(Copy, Clone, Debug, Default)]
pub struct PubkeyPair {
    pub pubkey_a: Pubkey,
    pub pubkey_b: Pubkey
}

impl PubkeyPair {
    pub fn any(&self, pubkey: Pubkey) -> bool {
        self.pubkey_a == pubkey || self.pubkey_b == pubkey
    }

    pub fn all(&self, pubkey_a: Pubkey, pubkey_b: Pubkey) -> bool {
        (self.pubkey_a == pubkey_a && self.pubkey_b == pubkey_b) || (self.pubkey_a == pubkey_b && self.pubkey_b == pubkey_a)
    }

    pub fn none(&self, pubkey_a: Pubkey, pubkey_b: Pubkey) -> bool {
        (self.pubkey_a != pubkey_a && self.pubkey_b != pubkey_b) || (self.pubkey_a != pubkey_b && self.pubkey_b != pubkey_a)
    }
}

pub fn read_pools<P: AsRef<Path>>(path: P) -> Result<Vec<Pubkey>, Box<dyn Error>> {
    let file = File::open(path).unwrap();
    let buffer_reader = BufReader::new(file);

    let data: Pools = serde_json::from_reader(buffer_reader).unwrap();
    let pools = data.pools.iter().map(|pool| {Pubkey::from_str(pool).unwrap()}).collect::<Vec<Pubkey>>();

    Ok(pools)
}

pub fn is_pool_account_pubkey(pools: Arc<Mutex<HashMap<Market, Vec<Pubkey>>>>, pubkey: &Pubkey) -> bool {
    pools.lock().unwrap().iter().any(|pool| {
        pool.1.iter().any(|pool_pubkey| { *pool_pubkey == *pubkey })
    })
}

#[derive(Deserialize, Debug)]
pub struct Pools {
    pub pools: Vec<String>
}