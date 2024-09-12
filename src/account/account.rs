use std::sync::Arc;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use crate::r#struct::market::{ConfigAccount, Market, PoolOperation};

pub enum DeserializedAccount {
    Account(Account),
    PoolAccount(DeserializedPoolAccount),
    TokenAccount(DeserializedTokenAccount),
    ConfigAccount(ConfigAccount)
}

#[derive(Clone)]
pub struct DeserializedPoolAccount {
    pub pubkey: Pubkey,
    pub account: Account,
    pub market: Market,
    pub operation: Arc<dyn PoolOperation>
}

impl DeserializedPoolAccount {
    pub fn get_swap_related_pubkeys(&self) -> Vec<(String, Pubkey)> {
        match self.market {
            Market::ORCA => { vec![] }
            Market::RAYDIUM => {
                let mut vec = vec![
                    (stringify!(self.pubkey).to_string(), self.pubkey)
                ];
                vec.append(&mut self.operation.get_swap_related_pubkeys());

                vec
            }
            Market::METEORA => { vec![] }
            Market::LIFINITY => { vec![] }
        }
    }
}

pub struct DeserializedTokenAccount {
    pub pubkey: Pubkey,
    pub account: Account,
}

pub trait AccountDataSerializer {
    fn unpack_data(data: &Vec<u8>) -> Self;
}