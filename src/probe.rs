use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use tokio::spawn;
use tokio::time::sleep;
use crate::pools::{Market, MarketOperation, MarketPool, resolve_market_data};

pub struct Probe {
    pub pools: Arc<HashMap<Market, Vec<Pubkey>>>,
    pub market_accounts: Arc<Mutex<Vec<MarketPool>>>
}

impl Probe {
    pub fn new(pools: Arc<HashMap<Market, Vec<Pubkey>>>, market_accounts: Arc<Mutex<Vec<MarketPool>>>) -> Probe {
        Probe {
            pools,
            market_accounts
        }
    }

    pub async fn start_watching(&self) {
        let pools = Arc::clone(&self.pools);
        let market_accounts = Arc::clone(&self.market_accounts);

        let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
        let rpc_client: RpcClient = RpcClient::new(get_blocks);

        spawn(async move {
            loop {
                println!("updating markets...");
                let _ = pools.iter().map(|pools| {
                    let accounts = match rpc_client.get_multiple_accounts(&*pools.1) {
                        Ok(accounts) => {
                            Some(accounts)
                        }
                        Err(err) => {
                            eprintln!("failed to fetch market: {}", pools.0.name());
                            None
                        }
                    }.unwrap_or(vec!());

                    let valid_accounts = accounts.iter().filter(|account| {
                        account.is_some()
                    }).map(|account| {
                        let data = account.clone().unwrap().data;

                        resolve_market_data(pools.0, &data)
                    }).collect::<Vec<Box<dyn MarketOperation>>>();

                    MarketPool {
                        market: (*pools.0).clone(),
                        accounts: valid_accounts,
                    }
                }).collect::<Vec<MarketPool>>();

                let _ = sleep(Duration::from_secs(10)).await;
            }
        });
    }
}