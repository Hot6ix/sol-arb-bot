mod probe;
mod constants;
mod utils;
mod formula;
mod account;
pub mod path;
mod r#struct;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::spawn;
use tokio::time::sleep;
use crate::account::account::DeserializedPoolAccount;
use path::PathFinder;
use crate::probe::Probe;
use crate::r#struct::market::Market;
use crate::utils::read_pools;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
    let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
    let rpc_client = RpcClient::new(get_blocks.clone());

    // read pools
    let orca_pools = read_pools("./src/pubkey/orca.json").unwrap();
    // let meteora_pools = read_pools("./src/pools/pubkey/meteora.json").unwrap();
    let raydium_pools = read_pools("./src/pubkey/raydium.json").unwrap();

    // concatenate all dex pools
    let pool_list = Arc::new(HashMap::from([
        (Market::ORCA, orca_pools),
        // (Market::METEORA, meteora_pools),
        (Market::RAYDIUM, raydium_pools),
    ]));

    // hold pool pubkey
    let pool_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>> = Arc::new(Mutex::new(Vec::new()));
    // hold pubkey in data or pda
    let shared_market_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>> = Arc::new(Mutex::new(Vec::new()));
    // hold available path list of mint
    let base_path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>> = Arc::new(Mutex::new(HashMap::new()));

    let probe = Probe::new(get_blocks);
    // fetch pool pubkeys
    probe.fetch_pool_accounts(Arc::clone(&pool_list), Arc::clone(&pool_account_bin));

    // collect swap-related pubkeys
    let res = Arc::clone(&pool_account_bin).lock().unwrap().iter().map(|account| {
        account.get_swap_related_pubkeys().into_iter().map(|item| { (account.market, item.0, item.1) }).collect::<Vec<(Market, String, Pubkey)>>()
    }).into_iter().flatten().collect::<Vec<(Market, String, Pubkey)>>();

    // resolve path
    let market_pool = Arc::clone(&pool_account_bin);
    let path_list = Arc::clone(&base_path_list);
    spawn(async move {
        loop {
            let path_finder = PathFinder {
                market_accounts: Arc::clone(&market_pool),
                path_list: Arc::clone(&path_list)
            };

            let mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
            path_finder.resolve_path(mint);

            let _ = sleep(Duration::from_secs(5)).await;
        }
    });

    let path_list = Arc::clone(&base_path_list);
    spawn(async move {
        loop {
            println!("print path list");
            path_list.lock().unwrap().iter().for_each(|x| {
                println!("[{}]", x.1.iter().map(|x1| {
                    format!("({}, {})", x1.operation.get_mint_pair().pubkey_a, x1.operation.get_mint_pair().pubkey_b)
                }).collect::<Vec<String>>().join(","))
            });
            let _ = sleep(Duration::from_secs(5)).await;
        }
    });

    loop {
        println!("main thread loop");
        sleep(Duration::from_secs(1)).await;
    }
}
