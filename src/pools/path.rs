use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketOperation, MarketPool, MarketPoolPair, PubkeyPair};

pub struct PathFinder {
    pub market_accounts: Arc<Mutex<Vec<MarketPool>>>
}

impl PathFinder {
    pub fn resolve_path(&self, mint: Pubkey) {
    }

    fn find_path(pools: Arc<Mutex<Vec<MarketPool>>>, tmp_path: Rc<RefCell<Vec<Box<dyn MarketOperation>>>>, start: usize, r: usize, next_mint: Pubkey, target_mint: Pubkey) {
        if r == 0 {

        }
        else {
            let pools = Arc::clone(&pools);
            for i in start..pools.lock().unwrap().len() {
                let accounts = &pools.lock().unwrap()[i].accounts;
                accounts.iter().filter(|account| {
                    account.get_mint_pair().any(next_mint)
                }).for_each(|market| {
                    // let pair = market.get_mint_pair();
                    // Rc::clone(&tmp_path).borrow_mut().push((*market).clone());
                    // let new_target_mint = if pair.0 == target_mint {
                    //     pair.1
                    // }
                    // else {
                    //     pair.0
                    // };
                    //
                    // Self::base2(max_depth, Rc::clone(&arr), Rc::clone(&visited), i+1, r-1, new_target_mint, round_trip_mint);
                    // Rc::clone(&visited).borrow_mut().pop();
                });
            }
        }
    }

    pub fn base2(max_depth: &usize, arr: Rc<RefCell<Vec<(i32, Vec<(i32, i32)>)>>>, visited: Rc<RefCell<Vec<(i32, (i32, i32))>>>, start: usize, r: usize, target_mint: i32, round_trip_mint: i32) {
        if r == 0 {
            if Self::validate_path(max_depth, &*Rc::clone(&visited).borrow(), &round_trip_mint) {
                println!("{}",
                         Rc::clone(&visited).borrow().iter().map(|x1| {format!("market: {}, [{}, {}]", x1.0.to_string(), x1.1.0, x1.1.1)}).collect::<Vec<String>>().join(",")
                );
            }
            return;
        }
        else {
            for i in start..Rc::clone(&arr).borrow().len() {
                let d = Rc::clone(&arr).borrow()[i].clone();
                d.1.iter().filter(|x| {
                    x.0 == target_mint || x.1 == target_mint
                }).for_each(|x2| {
                    let pair = x2;
                    let mut target: (i32, i32) = (*pair).clone();

                    if target.0 != target_mint {
                        target = (target_mint, target.0);
                    }

                    Rc::clone(&visited).borrow_mut().push((d.0, target));

                    let new_target_mint = if pair.0 == target_mint {
                        pair.1
                    }
                    else {
                        pair.0
                    };

                    Self::base2(max_depth, Rc::clone(&arr), Rc::clone(&visited), i+1, r-1, new_target_mint, round_trip_mint);
                    Rc::clone(&visited).borrow_mut().pop();
                });
            }
        }
    }

    pub fn validate_path(max_depth: &usize, path: &Vec<(i32, (i32, i32))>, round_trip_mint: &i32) -> bool {
        if max_depth < &path.len() {
            false
        }
        else {
            if path.iter().filter(|sub_path| {
                sub_path.1.0 == *round_trip_mint || sub_path.1.1 == *round_trip_mint
            }).collect::<Vec<_>>().len() == 2 {
                true
            }
            else {
                false
            }
        }
    }
}