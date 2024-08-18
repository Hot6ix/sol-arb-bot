use std::fmt::Debug;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use crate::constants::RPC_URL;
use crate::pool::PubkeyPair;

pub struct Probe {
    pub rpc_client: RpcClient,
    pub token_pool_pair: PubkeyPair
}

impl Probe {
    pub fn new(token_pool_pair: PubkeyPair) -> Probe {
        let rpc_client = RpcClient::new(RPC_URL.to_string());

        Probe {
            rpc_client,
            token_pool_pair
        }
    }

    pub async fn fetch(&self) -> Vec<Account> {
        let target_accounts = [self.token_pool_pair.pubkey_a, self.token_pool_pair.pubkey_b];
        let token_accounts = self.rpc_client.get_multiple_accounts(&target_accounts).await;

        match token_accounts {
            Ok(accounts) => {
                accounts.iter().flatten().map(|account| {
                    account.clone()
                }).collect::<Vec<Account>>()
            }
            Err(error) => {
                eprintln!("{}", error);
                vec!()
            }
        }
    }
}