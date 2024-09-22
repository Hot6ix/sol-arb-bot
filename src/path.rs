use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use tokio::time::Instant;
use crate::account::account::DeserializedPoolAccount;
use crate::constants::MAX_DEPTH;
use crate::r#struct::market::Market;

pub struct PathFinder {
    pub pool_accounts: Arc<Mutex<Vec<DeserializedPoolAccount>>>,
    pub path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>>,
}

impl PathFinder {
    pub fn resolve_path(&self, mint: Pubkey) {
        println!("start resolving path");
        let t = Instant::now();
        let path: Rc<RefCell<Vec<DeserializedPoolAccount>>> = Rc::new(RefCell::new(Vec::new()));
        let len = (*Arc::clone(&self.pool_accounts).lock().unwrap()).len();
        for i in 2..len + 1 {
            Self::find_path(
                Arc::clone(&self.path_list),
                Arc::clone(&self.pool_accounts),
                Rc::clone(&path),
                0,
                i,
                mint,
                mint
            )
        }

        println!("done: {:?}", t.elapsed());
    }

    fn find_path(
        path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>>,
        pools: Arc<Mutex<Vec<DeserializedPoolAccount>>>,
        path: Rc<RefCell<Vec<DeserializedPoolAccount>>>,
        start: usize,
        r: usize,
        next_mint: Pubkey,
        target_mint: Pubkey
    ) {
        if r == 0 {
            let tmp_path = Rc::clone(&path);
            if Self::validate_path(&tmp_path, &target_mint) {
                // println!("[{}]", tmp_path.borrow().iter().map(|x| {
                //     format!("({}({}) - ({}, {}))", x.market.name(), x.pubkey, x.operation.get_mint_pair().pubkey_a, x.operation.get_mint_pair().pubkey_b)
                // }).collect::<Vec<String>>().join(","));
                (*path_list.lock().unwrap()).insert(target_mint, tmp_path.take());
            }
            return;
        }
        else {
            let tmp_path = Rc::clone(&path);
            let pools = Arc::clone(&pools);

            let len = (*pools.lock().unwrap()).len();
            for i in start..len {
                let accounts = (*pools.lock().unwrap()).clone();

                let account = accounts[i].clone();
                let pair = account.operation.get_mint_pair();
                if !pair.any(next_mint) || Self::contains_dex(&account.market, &tmp_path.borrow()) {
                    continue;
                }

                tmp_path.borrow_mut().push(account.clone());
                let next_mint = if pair.pubkey_a == next_mint {
                    pair.pubkey_b
                }
                else {
                    pair.pubkey_a
                };

                Self::find_path(Arc::clone(&path_list), Arc::clone(&pools), Rc::clone(&path), i+1, r-1, next_mint, target_mint);
                tmp_path.borrow_mut().pop();

                // basic
                // let account = accounts[i].clone();
                // tmp_path.borrow_mut().push(account.clone());
                // Self::find_path(Arc::clone(&path_list), Arc::clone(&pools), Rc::clone(&path), i+1, r-1, next_mint, target_mint);
                // tmp_path.borrow_mut().pop();
            }
        }
    }

    fn validate_path(path: &Rc<RefCell<Vec<DeserializedPoolAccount>>>, target_mint: &Pubkey) -> bool {
        if MAX_DEPTH < path.borrow().len() {
            false
        }
        else {
            if path.borrow().iter().filter(|sub_path| {
                sub_path.operation.get_mint_pair().any(*target_mint)
            }).collect::<Vec<_>>().len() == 2 {
                true
            }
            else {
                false
            }
        }
    }

    fn contains_dex(market: &Market, accounts: &Vec<DeserializedPoolAccount>) -> bool {
        accounts.iter().find(|account| {
            account.market.eq(market)
        }).is_some()
    }
}