use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use crate::account::account::DeserializedPoolAccount;
use crate::constants::MAX_DEPTH;

pub struct PathFinder {
    pub market_accounts: Arc<Mutex<Vec<DeserializedPoolAccount>>>,
    pub path_list: Arc<Mutex<HashMap<Pubkey, Vec<DeserializedPoolAccount>>>>,
}

impl PathFinder {
    pub fn resolve_path(&self, mint: Pubkey) {
        let path: Rc<RefCell<Vec<DeserializedPoolAccount>>> = Rc::new(RefCell::new(Vec::new()));
        let len = (*Arc::clone(&self.market_accounts).lock().unwrap()).len();
        for i in 2..len + 1 {
            Self::find_path(
                Arc::clone(&self.path_list),
                Arc::clone(&self.market_accounts),
                Rc::clone(&path),
                0,
                i,
                mint,
                mint
            )
        }
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
                //     format!("({} - ({}, {}))", x.get_market_provider().name(), x.get_mint_pair().pubkey_a, x.get_mint_pair().pubkey_b)
                // }).collect::<Vec<String>>().join(","))
                (*path_list.lock().unwrap()).insert(target_mint, tmp_path.take());
            }
            return;
        }
        else {
            let pools = Arc::clone(&pools);
            let len = (*pools.lock().unwrap()).len();
            for i in start..len {
                let accounts = (*pools.lock().unwrap()).clone();
                accounts.iter().filter(|account| {
                    account.operation.get_mint_pair().any(next_mint)
                }).for_each(|market| {
                    let pair = market.operation.get_mint_pair();

                    let tmp_path = Rc::clone(&path);
                    tmp_path.borrow_mut().push((*market).clone());
                    let next_mint = if pair.pubkey_a == next_mint {
                        pair.pubkey_b
                    }
                    else {
                        pair.pubkey_a
                    };

                    Self::find_path(Arc::clone(&path_list), Arc::clone(&pools), Rc::clone(&path), i+1, r-1, next_mint, target_mint);
                    tmp_path.borrow_mut().pop();
                });
            }
        }
    }

    pub fn validate_path(path: &Rc<RefCell<Vec<DeserializedPoolAccount>>>, target_mint: &Pubkey) -> bool {
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
}