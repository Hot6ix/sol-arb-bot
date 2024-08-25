use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

pub fn read_pools<P: AsRef<Path>>(path: P) -> Result<Vec<Pubkey>, Box<dyn Error>> {
    let file = File::open(path).unwrap();
    let buffer_reader = BufReader::new(file);

    let data: Pools = serde_json::from_reader(buffer_reader).unwrap();
    let pools = data.pools.iter().map(|pool| {Pubkey::from_str(pool).unwrap()}).collect::<Vec<Pubkey>>();

    Ok(pools)
}

#[derive(Deserialize, Debug)]
pub struct Pools {
    pub pools: Vec<String>
}