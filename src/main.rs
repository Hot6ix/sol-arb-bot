extern crate core;

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::format;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::spawn;
use tokio::time::sleep;

use path::PathFinder;

use crate::account::account::{DeserializedAccount, DeserializedPoolAccount};
use crate::arbitrageur::Arbitrageur;
use crate::formula::base::Formula;
use crate::observer::{Event, Publisher};
use crate::probe::Probe;
use crate::r#struct::market::Market;
use crate::r#struct::market::Market::RAYDIUM;
use crate::utils::read_pools;

mod probe;
mod constants;
mod utils;
mod formula;
mod account;
pub mod path;
mod r#struct;
mod temp;
mod arbitrageur;
mod observer;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
    let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
    let rpc_client = RpcClient::new(get_blocks.clone());

    // read pools
    let orca_pools = read_pools("./src/pubkey/orca.json").unwrap();
    let meteora_pools = read_pools("./src/pubkey/meteora.json").unwrap();
    let raydium_pools = read_pools("./src/pubkey/raydium.json").unwrap();

    // concatenate all dex pools
    let pool_list = Arc::new(Mutex::new(HashMap::from([
        (Market::ORCA, orca_pools),
        // (Market::METEORA, meteora_pools),
        (Market::RAYDIUM, raydium_pools),
    ])));

    // hold pool pubkey
    let pool_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>> = Arc::new(Mutex::new(Vec::new()));
    // hold pubkey in data or pda
    let shared_account_bin: Arc<Mutex<Vec<DeserializedAccount>>> = Arc::new(Mutex::new(Vec::new()));
    // hold available path list of mint
    let path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut probe = Probe::new(get_blocks, Arc::new(Mutex::new(Publisher::default())));
    // fetch pool pubkeys
    probe.fetch_pool_accounts(Arc::clone(&pool_list), Arc::clone(&pool_account_bin));

    // resolve path
    let pool_accounts = Arc::clone(&pool_account_bin);
    let path_list = Arc::clone(&path_list);

    let path_finder = PathFinder {
        pool_accounts: Arc::clone(&pool_accounts),
        path_list: Arc::clone(&path_list)
    };

    let mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    path_finder.resolve_path(mint);

    //////////////////////////////////////////////////////////////////////////////////////////////////////////

    // collect swap-related pubkeys from pool accounts
    probe.start_watching(Arc::clone(&pool_account_bin), Arc::clone(&shared_account_bin));
    probe.publisher.lock().unwrap().subscribe(Event::UpdateAccounts, || {
        println!("updated")
    });

    let shared_pool_account_bin = Arc::clone(&shared_account_bin);
    let path_list = Arc::clone(&path_list);
    spawn(async move {
        loop {
            let path_list = path_list.lock().unwrap().clone();

            let target = path_list.iter().find(|path| {
                *path.0 == mint
            }).expect(format!("no path for mint: {}", mint).as_str());

            // target.1.iter().for_each(|pool| {
            //     let related_pubkeys = pool.get_swap_related_pubkeys();
            //
            //     let related_accounts = shared_pool_account_bin.lock().unwrap().clone().into_iter().filter(|account| {
            //         related_pubkeys.iter().find(|(_, pubkey)| {
            //             *pubkey == account.get_pubkey()
            //         }).is_some()
            //     }).collect::<Vec<DeserializedAccount>>();
            //
            //     // run only single swap
            //     if let Some(target_pool) = target.1.iter().find(|pool| {
            //         pool.operation.get_formula() == Formula::ConcentratedLiquidity && pool.market == RAYDIUM
            //     }) {
            //         target_pool.operation.swap(&related_accounts);
            //     }
            //
            //     // target.1.iter().for_each(|pool| {
            //     //     pool.operation.swap(&related_accounts);
            //     // });
            // });

            // single swap test
            if let Ok(related_pubkeys) = target.1.iter().find(|pool| {
               pool.market == RAYDIUM && pool.operation.get_formula() == Formula::ConcentratedLiquidity
            }).unwrap().get_swap_related_pubkeys(Some(&rpc_client)) {
                let related_accounts = shared_pool_account_bin.lock().unwrap().clone().into_iter().filter(|account| {
                    related_pubkeys.iter().find(|(_, pubkey)| {
                        *pubkey == account.get_pubkey()
                    }).is_some()
                }).collect::<Vec<DeserializedAccount>>();

                if let Some(target_pool) = target.1.iter().find(|pool| {
                    pool.market == RAYDIUM && pool.operation.get_formula() == Formula::ConcentratedLiquidity
                }) {
                    target_pool.operation.swap(&related_accounts);
                }
            }

            let _ = sleep(Duration::from_secs(5)).await;
        }
    });

    // let path_list = Arc::clone(&base_path_list);
    // spawn(async move {
    //     loop {
    //         println!("print path list");
    //         path_list.lock().unwrap().iter().for_each(|x| {
    //             println!("[{}]", x.1.iter().map(|x1| {
    //                 format!("{}: {} ({}, {})", x1.market.name(), x1.pubkey, x1.operation.get_mint_pair().pubkey_a, x1.operation.get_mint_pair().pubkey_b)
    //             }).collect::<Vec<String>>().join(","))
    //         });
    //         let _ = sleep(Duration::from_secs(5)).await;
    //     }
    // });

    loop {
        println!("main thread loop");
        sleep(Duration::from_secs(1)).await;
    }
}

pub fn test() {

}