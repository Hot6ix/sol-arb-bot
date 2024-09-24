use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use solana_client::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use tokio::spawn;
use tokio::time::{Instant, sleep};

use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedDataAccount, DeserializedPoolAccount, DeserializedTokenAccount};
use crate::account::resolver::{resolve_pool_account, resolve_pool_config_account};
use crate::observer::{Event, Publisher};
use crate::r#struct::market::Market;
use crate::r#struct::token::TokenAccount;

pub struct Probe {
    pub rpc_url: String,
    pub publisher: Arc<Mutex<Publisher>>
}

impl Probe {
    pub fn new(rpc_url: String, publisher: Arc<Mutex<Publisher>>) -> Probe {
        Probe {
            rpc_url,
            publisher
        }
    }

    // fetch pool accounts one time
    pub fn fetch_pool_accounts(
        &self,
        pools: Arc<Mutex<HashMap<Market, Vec<Pubkey>>>>,
        pool_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>>
    ) {
        let rpc_client: RpcClient = RpcClient::new(&self.rpc_url);

        println!("fetching market pools...");
        let fetched_markets = pools.lock().unwrap().iter().map(|pools| {
            let accounts = Self::_fetch_accounts(&rpc_client, pools.1);

            let valid_accounts = accounts.iter().enumerate().filter(|(index, account)| {
                account.is_some()
            }).map(|(index, account)| {
                let account = account.clone().unwrap();
                let data = account.data.clone();

                let market_operation = resolve_pool_account(pools.0, &data);
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

    // fetch accounts one time
    pub fn fetch_multiple_accounts(
        &self,
        items: Vec<(Market, DeserializedAccount, Pubkey)>,
        bin: Arc<Mutex<Vec<DeserializedAccount>>>
    ) {
        let rpc_client = RpcClient::new(self.rpc_url.clone());
        Self::_fetch_multiple_accounts(&rpc_client, items, bin, None)
    }

    // fetch accounts periodically
    pub fn start_watching(
        &self,
        pool_account_bin: Arc<Mutex<Vec<DeserializedPoolAccount>>>,
        bin: Arc<Mutex<Vec<DeserializedAccount>>>
    ) {
        let get_blocks = self.rpc_url.clone();
        let rpc_client = RpcClient::new(get_blocks);
        let publisher = Arc::clone(&self.publisher);

        let items = Arc::clone(&pool_account_bin).lock().unwrap().iter().map(|account| {
            account.get_swap_related_pubkeys(Some(&rpc_client)).unwrap().into_iter().map(|item| {
                (account.market, item.0, item.1)
            }).collect::<Vec<(Market, DeserializedAccount, Pubkey)>>()
        }).into_iter().flatten().collect::<Vec<(Market, DeserializedAccount, Pubkey)>>();

        spawn(async move {
            loop {
                Self::_fetch_multiple_accounts(
                    &rpc_client,
                    items.clone(),
                    Arc::clone(&bin),
                    Some(Arc::clone(&publisher))
                );

                let _ = sleep(Duration::from_secs(10)).await;
            }
        });
    }

    fn _fetch_multiple_accounts(
        rpc_client: &RpcClient,
        items: Vec<(Market, DeserializedAccount, Pubkey)>,
        bin: Arc<Mutex<Vec<DeserializedAccount>>>,
        publisher: Option<Arc<Mutex<Publisher>>>
    ) {
        println!("fetching accounts...");
        let time = Instant::now();
        let pubkeys = items.iter().map(|item| { item.2 }).collect::<Vec<Pubkey>>();
        let accounts = Self::_fetch_accounts(&rpc_client, &pubkeys);

        let fetched_accounts = accounts.iter().enumerate().filter(|(index, account)| {
            account.is_some()
        }).map(|(index, account)| {
            let account = account.clone().unwrap();

            match items[index].1 {
                DeserializedAccount::Account(_) => {
                    DeserializedAccount::Account(DeserializedDataAccount {
                        pubkey: items[index].2,
                        account,
                        market: items[index].0,
                    })
                }
                DeserializedAccount::PoolAccount(_) => {
                    let market_operation = resolve_pool_account(&items[index].0, &account.data);
                    DeserializedAccount::PoolAccount(
                        DeserializedPoolAccount {
                            pubkey: items[index].2,
                            account,
                            market: items[index].0,
                            operation: market_operation,
                        }
                    )
                }
                DeserializedAccount::TokenAccount(_) => {
                    DeserializedAccount::TokenAccount(DeserializedTokenAccount {
                        pubkey: pubkeys[index],
                        account: account.clone(),
                        token: TokenAccount::unpack_data(&account.data),
                        market: items[index].0,
                    })
                }
                DeserializedAccount::ConfigAccount(_) => {
                    DeserializedAccount::ConfigAccount(
                        resolve_pool_config_account(&items[index].0, &account.owner, pubkeys[index], &account.data)
                    )
                }
            }
        }).collect::<Vec<DeserializedAccount>>();

        *bin.lock().unwrap() = fetched_accounts;
        if let Some(publisher) = publisher {
            publisher.lock().unwrap().notify(Event::UpdateAccounts);
        }
        println!("{:?}", time.elapsed());
    }

    fn _fetch_accounts(
        rpc_client: &RpcClient,
        pubkeys: &Vec<Pubkey>
    ) -> Vec<Option<Account>> {
        match rpc_client.get_multiple_accounts(pubkeys) {
            Ok(accounts) => {
                Some(accounts)
            }
            Err(err) => {
                eprintln!("failed to fetch pubkeys");
                None
            }
        }.unwrap_or(vec!())
    }
}