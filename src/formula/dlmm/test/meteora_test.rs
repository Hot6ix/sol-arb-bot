#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::str::FromStr;
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_sdk::clock::Clock;
    use solana_sdk::pubkey::Pubkey;
    use crate::formula::dlmm::bin::BinArray;
    use crate::formula::meteora_dlmm::{get_bin_array_pubkeys_for_swap, quote_exact_in, quote_exact_out};
    use crate::r#struct::account::AccountDataSerializer;
    use crate::r#struct::pools::MeteoraDlmmMarket;

    async fn get_clock(rpc_client: RpcClient) -> Result<Clock, &'static str> {
        let clock_account = rpc_client
            .get_account(&Pubkey::from_str("SysvarC1ock11111111111111111111111111111111").unwrap())
            .await
            .unwrap();

        let clock_state: Clock = bincode::deserialize(clock_account.data.as_ref()).unwrap();

        Ok(clock_state)
    }

    #[tokio::test]
    async fn test_swap_quote_exact_out() {
        // RPC client. No gPA is required.
        let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
        let rpc_client = RpcClient::new(alchemy);

        let SOL_USDC = Pubkey::from_str("HTvjzsfX3yU6BUodCjZ5vZkUrAxMDTrBs3CJaq43ashR").unwrap();

        // let lb_pair = program.account::<MeteoraDlmmMarket>(SOL_USDC).await.unwrap();

        let data = rpc_client.get_account_data(&SOL_USDC).await.unwrap();
        let lb_pair = MeteoraDlmmMarket::unpack_data(&data);

        // 3 bin arrays to left, and right is enough to cover most of the swap, and stay under 1.4m CU constraint.
        // Get 3 bin arrays to the left from the active bin
        let left_bin_array_pubkeys =
            get_bin_array_pubkeys_for_swap(SOL_USDC, &lb_pair, None, true, 3).unwrap();

        // Get 3 bin arrays to the right the from active bin
        let right_bin_array_pubkeys =
            get_bin_array_pubkeys_for_swap(SOL_USDC, &lb_pair, None, false, 3).unwrap();

        // Fetch bin arrays
        let bin_array_pubkeys = left_bin_array_pubkeys
            .into_iter()
            .chain(right_bin_array_pubkeys.into_iter())
            .collect::<Vec<Pubkey>>();

        let accounts = rpc_client
            .get_multiple_accounts(&bin_array_pubkeys)
            .await
            .unwrap();

        let bin_arrays = accounts
            .into_iter()
            .zip(bin_array_pubkeys.into_iter())
            .map(|(account, key)| {
                (
                    key,
                    BinArray::unpack_data(&account.unwrap().data)
                )
            })
            .collect::<HashMap<_, _>>();

        let usdc_token_multiplier = 1_000_000.0;
        let sol_token_multiplier = 1_000_000_000.0;

        let out_sol_amount = 1_000_000_000;
        let clock = get_clock(rpc_client).await.unwrap();

        let quote_result = quote_exact_out(
            SOL_USDC,
            &lb_pair,
            out_sol_amount,
            false,
            bin_arrays.clone(),
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        let in_amount = quote_result.amount_in + quote_result.fee;

        println!(
            "{} USDC -> exact 1 SOL",
            in_amount as f64 / usdc_token_multiplier
        );

        let quote_result = quote_exact_in(
            SOL_USDC,
            &lb_pair,
            in_amount,
            false,
            bin_arrays.clone(),
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        println!(
            "{} USDC -> {} SOL",
            in_amount as f64 / usdc_token_multiplier,
            quote_result.amount_out as f64 / sol_token_multiplier
        );

        let out_usdc_amount = 200_000_000;

        let quote_result = quote_exact_out(
            SOL_USDC,
            &lb_pair,
            out_usdc_amount,
            true,
            bin_arrays.clone(),
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        let in_amount = quote_result.amount_in + quote_result.fee;

        println!(
            "{} SOL -> exact 200 USDC",
            in_amount as f64 / sol_token_multiplier
        );

        let quote_result = quote_exact_in(
            SOL_USDC,
            &lb_pair,
            in_amount,
            true,
            bin_arrays,
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        println!(
            "{} SOL -> {} USDC",
            in_amount as f64 / sol_token_multiplier,
            quote_result.amount_out as f64 / usdc_token_multiplier
        );
    }

    #[tokio::test]
    async fn test_swap_quote_exact_in() {
        // RPC client. No gPA is required.
        let alchemy = "https://solana-mainnet.g.alchemy.com/v2/76-rZCjoPGCHXLfjHNojk5CiqX8I36AT".to_string();
        let rpc_client = RpcClient::new(alchemy);

        let SOL_USDC = Pubkey::from_str("HTvjzsfX3yU6BUodCjZ5vZkUrAxMDTrBs3CJaq43ashR").unwrap();

        let data = rpc_client.get_account_data(&SOL_USDC).await.unwrap();
        let lb_pair = MeteoraDlmmMarket::unpack_data(&data);

        // 3 bin arrays to left, and right is enough to cover most of the swap, and stay under 1.4m CU constraint.
        // Get 3 bin arrays to the left from the active bin
        let left_bin_array_pubkeys =
            get_bin_array_pubkeys_for_swap(SOL_USDC, &lb_pair, None, true, 3).unwrap();

        // Get 3 bin arrays to the right the from active bin
        let right_bin_array_pubkeys =
            get_bin_array_pubkeys_for_swap(SOL_USDC, &lb_pair, None, false, 3).unwrap();

        // Fetch bin arrays
        let bin_array_pubkeys = left_bin_array_pubkeys
            .into_iter()
            .chain(right_bin_array_pubkeys.into_iter())
            .collect::<Vec<Pubkey>>();

        let accounts = rpc_client
            .get_multiple_accounts(&bin_array_pubkeys)
            .await
            .unwrap();

        let bin_arrays = accounts
            .into_iter()
            .zip(bin_array_pubkeys.into_iter())
            .map(|(account, key)| {
                (
                    key,
                    BinArray::unpack_data(&account.unwrap().data)
                )
            })
            .collect::<HashMap<_, _>>();

        // 1 SOL -> USDC
        let in_sol_amount = 1_000_000_000;

        let clock = get_clock(rpc_client).await.unwrap();

        let quote_result = quote_exact_in(
            SOL_USDC,
            &lb_pair,
            in_sol_amount,
            true,
            bin_arrays.clone(),
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        println!(
            "1 SOL -> {:?} USDC",
            quote_result.amount_out as f64 / 1_000_000.0
        );

        // 100 USDC -> SOL
        let in_usdc_amount = 100_000_000;

        let quote_result = quote_exact_in(
            SOL_USDC,
            &lb_pair,
            in_usdc_amount,
            false,
            bin_arrays.clone(),
            None,
            clock.unix_timestamp as u64,
            clock.slot,
        )
            .unwrap();

        println!(
            "100 USDC -> {:?} SOL",
            quote_result.amount_out as f64 / 1_000_000_000.0
        );
    }
}