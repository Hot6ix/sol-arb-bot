/*
    this test is to verify multiple pools swap that have same formula return same result
    e.g.
 */

#[cfg(test)]
mod complex_test {
    use crate::formula::clmm::orca_swap_state::{PostSwapUpdate, SwapTickSequence};
    use crate::formula::clmm::test::raydium_swap_test::swap_test::{build_swap_param, build_tick, get_tick_array_states_mut, TickArrayInfo};
    use crate::formula::clmm::test::swap_test_fixture::{SwapTestFixture, SwapTestFixtureInfo, TS_128};
    use crate::formula::raydium_clmm::swap_internal;

    pub fn default_orca_swap(
        liquidity: u128,
        tick_current: i32,
        start_tick_index: i32,
        amount: u64,
        sqrt_price_x64_limit: u128,
        a_to_b: bool,
        amount_specified_is_input: bool,
    ) -> PostSwapUpdate {
        let swap_test_info = SwapTestFixture::new(SwapTestFixtureInfo {
            tick_spacing: TS_128,
            liquidity,
            curr_tick_index: tick_current,
            start_tick_index,
            trade_amount: amount,
            sqrt_price_limit: sqrt_price_x64_limit,
            amount_specified_is_input,
            a_to_b,
            ..Default::default()
        });
        let mut tick_sequence = SwapTickSequence::new(
            swap_test_info.tick_arrays[0].to_owned(),
            Some(swap_test_info.tick_arrays[1].to_owned()),
            Some(swap_test_info.tick_arrays[2].to_owned()),
        );
        let post_swap = swap_test_info.run(&mut tick_sequence, 100);

        post_swap
    }

    #[test]
    pub fn test_a() {
        let mut tick_current = -32395;
        let mut start_tick_index = vec![-32400, -36000];
        let mut liquidity = 5124165121219;
        let mut sqrt_price_x64 = 3651942632306380802;
        let sqrt_price_x64_limit = 3049500711113990606u128;
        let mut amount = 12188240002u64;

        let (amm_config, pool_state, mut tick_array_states) =
            build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![
                    TickArrayInfo {
                        start_tick_index: start_tick_index[0],
                        ticks: vec![
                            build_tick(-32400, 277065331032, -277065331032).take(),
                            build_tick(-29220, 1330680689, -1330680689).take(),
                            build_tick(-28860, 6408486554, -6408486554).take(),
                        ],
                    },
                    TickArrayInfo {
                        start_tick_index: start_tick_index[1],
                        ticks: vec![
                            build_tick(-32460, 1194569667438, 536061033698).take(),
                            build_tick(-32520, 790917615645, 790917615645).take(),
                            build_tick(-32580, 152146472301, 128451145459).take(),
                            build_tick(-32640, 2625605835354, -1492054447712).take(),
                        ],
                    },
                ],
            );

        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            amount,
            sqrt_price_x64_limit,
            true,
            true,
        ).unwrap();

        let post_swap = default_orca_swap(
            liquidity,
            tick_current,
            start_tick_index[0],
            amount,
            sqrt_price_x64_limit,
            true,
            true
        );

        println!("r1(raydium): {}, {}", amount_0, amount_1);
        println!("r1(orca): {}, {}", post_swap.amount_a, post_swap.amount_b);

        ////////////////////////////////////////////////////
        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;
        amount = 121882400020;

        tick_array_states.pop_front();
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            amount,
            sqrt_price_x64_limit,
            true,
            true,
        )
            .unwrap();

        let post_swap = default_orca_swap(
            liquidity,
            tick_current,
            start_tick_index[1],
            amount,
            sqrt_price_x64_limit,
            true,
            true
        );

        println!("r2(raydium): {}, {}", amount_0, amount_1);
        println!("r2(orca): {}, {}", post_swap.amount_a, post_swap.amount_b);

        ////////////////////////////////////////////////////
        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;
        amount = 60941200010;

        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            amount,
            sqrt_price_x64_limit,
            true,
            true,
        )
            .unwrap();

        let post_swap = default_orca_swap(
            liquidity,
            tick_current,
            start_tick_index[1],
            amount,
            sqrt_price_x64_limit,
            true,
            true
        );

        println!("r3(raydium): {}, {}", amount_0, amount_1);
        println!("r3(orca): {}, {}", post_swap.amount_a, post_swap.amount_b);
    }

    #[test]
    pub fn test_b() {
        let mut tick_current = -32395;
        let mut liquidity = 5124165121219;
        let mut sqrt_price_x64 = 3651942632306380802;

        let mut start_tick_index = vec![-36000];
        let sqrt_price_x64_limit = 3049500711113990606u128;
        let amount = 477470480u64;

        let (amm_config, pool_state, mut tick_array_states) =
            build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![
                    TickArrayInfo {
                        start_tick_index: -32400,
                        ticks: vec![
                            build_tick(-32400, 277065331032, -277065331032).take(),
                            build_tick(-29220, 1330680689, -1330680689).take(),
                            build_tick(-28860, 6408486554, -6408486554).take(),
                        ],
                    },
                    TickArrayInfo {
                        start_tick_index: -36000,
                        ticks: vec![
                            build_tick(-32460, 1194569667438, 536061033698).take(),
                            build_tick(-32520, 790917615645, 790917615645).take(),
                            build_tick(-32580, 152146472301, 128451145459).take(),
                            build_tick(-32640, 2625605835354, -1492054447712).take(),
                        ],
                    },
                ],
            );

        // just cross the tickarray boundary(-32400), hasn't reached the next tick array initialized tick
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            amount,
            sqrt_price_x64_limit,
            true,
            false,
        ).unwrap();

        let post_swap = default_orca_swap(
            liquidity,
            tick_current,
            start_tick_index[0],
            amount,
            sqrt_price_x64_limit,
            true,
            false,
        );

        println!("raydium: {}, {}", amount_1, amount_0);
        println!("orca: {}, {}", post_swap.amount_b, post_swap.amount_a);
    }
}