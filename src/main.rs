mod token;
mod probe;
mod constants;
mod pools;
mod utils;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::time::sleep;
use crate::pools::{Market, MarketOperation, MarketPool};
use crate::probe::Probe;
use crate::utils::read_pools;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
    let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
    let rpc_client = RpcClient::new(get_blocks);

    // read pools
    let orca_pools = read_pools("./src/pools/accounts/orca.json").unwrap();
    let meteora_pools = read_pools("./src/pools/accounts/meteora.json").unwrap();
    let raydium_pools = read_pools("./src/pools/accounts/raydium.json").unwrap();

    // concatenate all dex pools
    let pool_list = Arc::new(HashMap::from([
        (Market::ORCA, orca_pools),
        (Market::METEORA, meteora_pools),
        (Market::RAYDIUM, raydium_pools),
    ]));

    // init shared market data
    let market_pool: Arc<Mutex<Vec<MarketPool>>> = Arc::new(Mutex::new(Vec::new()));

    // start monitoring
    Probe::new(Arc::clone(&pool_list), Arc::clone(&market_pool)).start_watching().await;

    // find and save path
    // start arbitrage

    // let dd = Arc::clone(&market_data);
    // spawn(async move {
    //     loop {
    //         dd.lock().unwrap().iter().for_each(|x| {
    //             let accounts = x.1.iter().map(|x1| {x1.lamports.to_string()}).collect::<Vec<String>>().join(",");
    //             println!("size of accounts: {}", x.1.len());
    //         });
    //         let _ = sleep(Duration::from_secs(1)).await;
    //     }
    // });

    let d = Arc::clone(&market_pool);
    loop {
        println!("main thread loop");

        println!("length of accounts: {}", d.lock().unwrap().len());
        sleep(Duration::from_secs(1)).await;
    }

    // fetch pool accounts
    // let pool_accounts = pools_list.into_iter().map(|pools| {
    //     println!("fetching {:?} markets...", pools.0);
    //     let accounts = rpc_client.get_multiple_accounts(&*pools.1).unwrap();
    //
    //     let valid_accounts = accounts.iter().filter(|account| {
    //         account.is_some()
    //     }).map(|account| {
    //         account.clone().unwrap()}
    //     ).collect::<Vec<Account>>();
    //
    //     (pools.0, valid_accounts)
    // }).collect::<HashMap<Market, Vec<Account>>>();

    // parse account data
    // let markets = pool_accounts.into_iter().map(|pools| {
    //     let market = pools.1.into_iter().map(|pool_account| {
    //         resolve_market_data(&pools.0, &pool_account.data)
    //     }).collect::<Vec<Box<dyn MarketOperation>>>();
    //
    //     (pools.0, market)
    // }).collect::<HashMap<Market, Vec<Box<dyn MarketOperation>>>>();



    // let start_mint = Pubkey::from_str("").unwrap();
    // let mut path = vec!(start_mint);

    // ==========================
    // let lifinity_jup_wsol_pubkey = Pubkey::from_str("7GXdv2r3fEuzAwEBZwtNoEjgFfrZdtHyNKBTLYfFwaAM").unwrap();
    // let account_data = rpc_client.get_account_data(&lifinity_jup_wsol_pubkey).await.unwrap();
    // println!("{}", account_data.len());
    // let market = resolve_market_data(Market::LIFINITY, &account_data);
    // let a = market.get_mint_pair();
    // println!("{}, {}", a.pubkey_a, a.pubkey_b);
    // let b = market.get_pool_pair();
    // println!("{}, {}", b.pubkey_a, b.pubkey_b);
}