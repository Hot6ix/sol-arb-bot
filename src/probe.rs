use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use time::OffsetDateTime;
use tokio::spawn;
use tokio::time::sleep;
use crate::pools::Market;
use crate::token::{MarketPool, TrackedAccount};

pub struct Probe {
    pub pools: Arc<HashMap<Market, Vec<Pubkey>>>,
    pub market_accounts: Arc<Mutex<MarketPool>>
}

impl Probe {
    pub fn new(pools: Arc<HashMap<Market, Vec<Pubkey>>>, market_accounts: Arc<Mutex<MarketPool>>) -> Probe {
        Probe {
            pools,
            market_accounts
        }
    }

    pub async fn start_watching(&self) {
        let pools = Arc::clone(&self.pools);
        let market_data = Arc::clone(&self.market_accounts);

        let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
        let rpc_client: RpcClient = RpcClient::new(get_blocks);

        spawn(async move {
            loop {
                println!("updating markets...");
                let pool_accounts = pools.iter().map(|pools| {
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
                        TrackedAccount {
                            time: OffsetDateTime::now_utc(),
                            market: Market::from(pools.0),
                            account: account.clone().unwrap(),
                        }
                    }).collect::<Vec<TrackedAccount>>();

                    valid_accounts
                }).collect::<Vec<Vec<TrackedAccount>>>();

                let tracked_accounts = pool_accounts.into_iter().flatten().collect::<Vec<TrackedAccount>>();
                market_data.lock().unwrap().replace(tracked_accounts);

                let _ = sleep(Duration::from_secs(10)).await;
            }
        });
    }
}