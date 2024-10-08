use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use crate::r#struct::account::{DeserializedAccount, DeserializedPoolAccount};

pub struct Arbitrageur {
    shared_account_bin: Arc<Mutex<Vec<DeserializedAccount>>>,
    path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>>
}

impl Arbitrageur {
    pub fn new(
        shared_account_bin: Arc<Mutex<Vec<DeserializedAccount>>>,
        path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>>
    ) -> Arbitrageur {
        Arbitrageur {
            shared_account_bin,
            path_list
        }
    }

    pub fn arbitrage_single(
        &self,
        target_mint: Pubkey,
        init_amount: u64
    ) {
        if let Some(path_list) = self.path_list.lock().unwrap().get(&target_mint) {
            path_list.iter().for_each(|pool| {
                let accounts = self.shared_account_bin.lock().unwrap().iter().filter(|account| {
                    account.get_market() == pool.market
                }).map(|account| {
                    account.clone()
                }).collect::<Vec<DeserializedAccount>>();

                pool.operation.swap(&accounts);
            })
        }
    }
}