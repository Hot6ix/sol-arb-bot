use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use tokio::spawn;
use tokio::time::sleep;
use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedPoolAccount, DeserializedTokenAccount};
use crate::account::account::DeserializedAccount::ConfigAccount;
use crate::account::resolver::resolve_market_data;
use crate::constants::{TOKEN_ACCOUNT_DATA_LEN, TOKEN_PROGRAM_PUBKEY};
use crate::r#struct::market::ConfigAccount::RaydiumClmmConfigAccount;
use crate::r#struct::market::Market;
use crate::r#struct::pools::{AmmConfig, RaydiumClmmAccount};

pub struct Probe {
    pub rpc_url: String
}

impl Probe {
    pub fn new(rpc_url: String) -> Probe {
        Probe {
            rpc_url
        }
    }

    pub fn fetch_pool_accounts(
        &self,
        pools: Arc<HashMap<Market, Vec<Pubkey>>>,
        pool_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>>
    ) {
        let rpc_client: RpcClient = RpcClient::new(&self.rpc_url);

        println!("fetching market pools...");
        let fetched_markets = pools.iter().map(|pools| {
            let accounts = Self::fetch_accounts(&rpc_client, pools.1);

            let valid_accounts = accounts.iter().enumerate().filter(|(index, account)| {
                account.is_some()
            }).map(|(index, account)| {
                let account = account.clone().unwrap();
                let data = account.data.clone();

                let market_operation = resolve_market_data(pools.0, &data);
                DeserializedPoolAccount {
                    pubkey: (&*pools.1)[index],
                    account,
                    market: (*pools.0).clone(),
                    operation: market_operation,
                }
            }).collect::<Vec<DeserializedPoolAccount>>();

            valid_accounts
        }).collect::<Vec<Vec<DeserializedPoolAccount>>>().into_iter().flatten().collect::<Vec<DeserializedPoolAccount>>();

        *pool_account_bin.lock().unwrap() = fetched_markets;
    }

    pub fn fetch_multiple_accounts(
        &self,
        items: Vec<(String, Pubkey)>,
        market: Option<Market>,
        bin: Arc<Mutex<Vec<DeserializedAccount>>>
    ) {
        let rpc_client: RpcClient = RpcClient::new(&self.rpc_url);

        println!("fetching pubkeys...");
        let pubkeys = items.iter().map(|item| { item.1 }).collect::<Vec<Pubkey>>();
        let accounts = Self::fetch_accounts(&rpc_client, &pubkeys);

        let fetched_accounts = accounts.iter().enumerate().filter(|(index, account)| {
            account.is_some()
        }).map(|(index, account)| {
            let account = account.clone().unwrap();

            // token account
            if account.owner.to_string() == TOKEN_PROGRAM_PUBKEY && account.data.len() == TOKEN_ACCOUNT_DATA_LEN {
                return DeserializedAccount::TokenAccount(DeserializedTokenAccount {
                    pubkey: pubkeys[index],
                    account,
                })
            }

            if market.is_some() {
                match market.unwrap() {
                    Market::ORCA => { todo!() }
                    Market::RAYDIUM => {
                        todo!()
                        // ConfigAccount(
                        //     RaydiumClmmConfigAccount(RaydiumClmmAccount::AmmConfig(AmmConfig::unpack_data(&account.data)))
                        // )
                    }
                    Market::METEORA => { todo!() }
                    Market::LIFINITY => { todo!() }
                }
            }
            else {
                DeserializedAccount::Account(account)
            }
        }).collect::<Vec<DeserializedAccount>>();

        *bin.lock().unwrap() = fetched_accounts;
    }

    pub fn start_watching(
        &self,
    ) {
        // let pools = Arc::clone(&self.pools);
        // let market_accounts = Arc::clone(&self.market_accounts);

        let get_blocks = self.rpc_url.clone();
        let rpc_client: RpcClient = RpcClient::new(get_blocks);

        spawn(async move {
            loop {
                println!("updating markets...");
                // let fetched_markets = pools.iter().map(|pools| {
                //     let pubkey = match rpc_client.get_multiple_accounts(&*pools.1) {
                //         Ok(pubkey) => {
                //             Some(pubkey)
                //         }
                //         Err(err) => {
                //             eprintln!("failed to fetch market: {}", pools.0.name());
                //             None
                //         }
                //     }.unwrap_or(vec!());
                //
                //     let valid_accounts = pubkey.iter().filter(|account| {
                //         account.is_some()
                //     }).map(|account| {
                //         let data = account.clone().unwrap().data;
                //
                //         resolve_market_data(pools.0, &data)
                //     }).collect::<Vec<Arc<dyn MarketOperation>>>();
                //
                //     MarketPool {
                //         market: (*pools.0).clone(),
                //         pubkey: valid_accounts,
                //     }
                // }).collect::<Vec<MarketPool>>();
                //
                // // todo: replace
                // *market_accounts.lock().unwrap() = fetched_markets;

                let _ = sleep(Duration::from_secs(10)).await;
            }
        });
    }

    fn fetch_accounts(
        rpc_client: &RpcClient,
        pubkeys: &Vec<Pubkey>
    ) -> Vec<Option<Account>> {
        match rpc_client.get_multiple_accounts(pubkeys) {
            Ok(accounts) => {
                Some(accounts)
            }
            Err(err) => {
                eprintln!("failed to fetch pubkey");
                None
            }
        }.unwrap_or(vec!())
    }
}