mod token;
mod probe;
mod constants;
mod pools;

use std::str::FromStr;
use actix::{Actor};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use crate::pools::{Market, resolve_market_data};

#[actix::main]
async fn main() {
    println!("Hello, world!");

    let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
    let get_blocks = "https://go.getblock.io/bd8eab2bbe6e448b84ca2ae3b282b819".to_string();
    let rpc_client = RpcClient::new(get_blocks);

    // let orca_wsol_orca_pubkey = Pubkey::from_str("Hxw77h9fEx598afiiZunwHaX3vYu9UskDk9EpPNZp1mG").unwrap();
    // let orca_jup_wsol_pubkey = Pubkey::from_str("C1MgLojNLWBKADvu9BHdtgzz1oZX4dZ5zGdGcgvvW8Wz").unwrap();
    // let account_data = rpc_client.get_account_data(&orca_jup_wsol_pubkey).await.unwrap();
    // let orca = OrcaMarket::unpack_data(&account_data);

    // let meteora_jup_wsol_pubkey = Pubkey::from_str("7qt1qBnQ5CNNpMH1no6jYAzuyazP5QWXsUZB7dot5kga").unwrap();
    // let account_data = rpc_client.get_account_data(&meteora_jup_wsol_pubkey).await.unwrap();
    // println!("{}", account_data.len());
    // let meteora = MeteoraMarket::unpack_data(&account_data);
    // println!("token_x: {}, token_y: {}", meteora.token_x_mint, meteora.token_y_mint);

    // let raydium_jup_wsol_pubkey = Pubkey::from_str("EZVkeboWeXygtq8LMyENHyXdF5wpYrtExRNH9UwB1qYw").unwrap();
    // let account_data = rpc_client.get_account_data(&raydium_jup_wsol_pubkey).await.unwrap();

    let lifinity_jup_wsol_pubkey = Pubkey::from_str("7GXdv2r3fEuzAwEBZwtNoEjgFfrZdtHyNKBTLYfFwaAM").unwrap();
    let account_data = rpc_client.get_account_data(&lifinity_jup_wsol_pubkey).await.unwrap();
    println!("{}", account_data.len());
    let market = resolve_market_data(Market::LIFINITY, &account_data);
    let a = market.get_mint_pair();
    println!("{}, {}", a.pubkey_a, a.pubkey_b);
    let b = market.get_pool_pair();
    println!("{}, {}", b.pubkey_a, b.pubkey_b);



    // let account_data_as_string = {
    //     let d = &account_data;
    //     println!("len: {}", d.iter().len());
    //     d.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")
    // };
    // println!("{}", account_data_as_string);

    // let p2 = Pubkey::from_str("2LecshUwdy9xi7meFgHtFJQNSKk4KdTrcpvaB56dP2NQ").unwrap();
    // let aa = {
    //     p2.to_bytes().iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")
    // };
    // println!("{}", aa.to_string());

    // let probe = Probe::new(pair);
    // let res = probe.fetch().await;
    // res.iter().for_each(|x| {
    //     let token_account = TokenAccount::unpack_data(&x.data);
    //     println!("amount: {},", token_account.amount);
    // })



    // let probe = Probe::create(|ctx| {
    //     // aldrin jungle-usdc pair
    //     let pair = PubkeyPair {
    //         pubkey_a: Pubkey::from_str("9oZSaBCvLVV4GStGwaZhzTGpX5yGhzZXwbYVbX4BAc5A").unwrap(),
    //         pubkey_b: Pubkey::from_str("5KiPd7vTx2y8yNDQeHeSQn6FCjmF28Gzz9tF1XTF5RHE").unwrap(),
    //     };
    //
    //     Probe {
    //         rpc_client: RpcClient::new(RPC_URL.to_string()),
    //         token_pool_pair: pair
    //     }
    // });

    // let result = probe.send(Fetch()).await;
}
