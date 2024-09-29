use std::collections::VecDeque;

#[cfg(test)]
mod swap_test {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::str::FromStr;
    use std::vec;
    use solana_sdk::pubkey::Pubkey;
    use crate::account::account::AccountDataSerializer;
    use crate::formula::clmm::constant::{POOL_TICK_ARRAY_BITMAP_SEED};
    use crate::formula::clmm::raydium_swap_state::{add_delta, get_delta_amounts_signed, get_liquidity_from_amounts};
    use crate::formula::clmm::raydium_tick_array::{TickArrayBitmapExtension, TickArrayState, TickState};
    use crate::formula::clmm::raydium_tick_array::tick_array_bitmap_extension_test::{build_tick_array_bitmap_extension_info, BuildExtensionAccountInfo};
    use crate::formula::clmm::raydium_tick_math::get_sqrt_price_at_tick;
    use crate::formula::raydium_clmm::swap_internal;
    use crate::r#struct::pools::{AmmConfig, RaydiumClmmMarket, RaydiumRewardInfo};
    use crate::r#struct::pools::pool_test::build_pool;

    const PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";

    pub fn get_tick_array_states_mut(
        deque_tick_array_states: &VecDeque<TickArrayState>,
    ) -> VecDeque<TickArrayState> {
        let mut tick_array_states = VecDeque::new();

        for tick_array_state in deque_tick_array_states {
            tick_array_states.push_back(tick_array_state.clone());
        }
        tick_array_states
    }

    fn build_swap_param<'info>(
        tick_current: i32,
        tick_spacing: u16,
        sqrt_price_x64: u128,
        liquidity: u128,
        tick_array_infos: Vec<TickArrayInfo>,
    ) -> (
        AmmConfig,
        RefCell<RaydiumClmmMarket>,
        VecDeque<TickArrayState>,
        // todo
        // RefCell<ObservationState>,
    ) {
        let amm_config = AmmConfig {
            trade_fee_rate: 1000,
            tick_spacing,
            ..Default::default()
        };
        let pool_state = build_pool(tick_current, tick_spacing, sqrt_price_x64, liquidity);

        // todo
        // let observation_state = RefCell::new(ObservationState::default());
        // observation_state.borrow_mut().pool_id = pool_state.borrow().key(&PROGRAM_PUBKEY);

        let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();
        let mut tick_array_states: VecDeque<TickArrayState> = VecDeque::new();
        for tick_array_info in tick_array_infos {
            tick_array_states.push_back(build_tick_array_with_tick_states(
                pool_state.borrow().key(&program_pubkey),
                tick_array_info.start_tick_index,
                tick_spacing,
                tick_array_info.ticks,
            ));
            pool_state
                .borrow_mut()
                .flip_tick_array_bit(None, tick_array_info.start_tick_index)
                .unwrap();
        }

        // todo
        // (amm_config, pool_state, tick_array_states, observation_state)
        (amm_config, pool_state, tick_array_states)
    }

    pub struct OpenPositionParam {
        pub amount_0: u64,
        pub amount_1: u64,
        // pub liquidity: u128,
        pub tick_lower: i32,
        pub tick_upper: i32,
    }

    fn setup_swap_test<'info>(
        start_tick: i32,
        tick_spacing: u16,
        position_params: Vec<OpenPositionParam>,
        zero_for_one: bool,
    ) -> (
        AmmConfig,
        RefCell<RaydiumClmmMarket>,
        VecDeque<TickArrayState>,
        // todo
        // RefCell<ObservationState>,
        TickArrayBitmapExtension,
        u64,
        u64,
    ) {
        let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();
        let amm_config = AmmConfig {
            trade_fee_rate: 1000,
            tick_spacing,
            ..Default::default()
        };

        let pool_state_refcel = build_pool(
            start_tick,
            tick_spacing,
            get_sqrt_price_at_tick(start_tick).unwrap(),
            0,
        );

        // todo
        // let observation_state = RefCell::new(ObservationState::default());

        let param = &mut BuildExtensionAccountInfo::default();
        param.key = Pubkey::find_program_address(
            &[
                POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                pool_state_refcel.borrow().key(&program_pubkey).as_ref(),
            ],
            &program_pubkey,
        )
            .0;
        let bitmap_extension = build_tick_array_bitmap_extension_info(param);
        let mut tick_array_states: VecDeque<TickArrayState> = VecDeque::new();
        let mut sum_amount_0: u64 = 0;
        let mut sum_amount_1: u64 = 0;
        {
            let mut pool_state = pool_state_refcel.borrow_mut();
            // todo
            // observation_state.borrow_mut().pool_id = pool_state.key();

            let mut tick_array_map = HashMap::new();

            for position_param in position_params {
                let liquidity = get_liquidity_from_amounts(
                    pool_state.sqrt_price_x64,
                    get_sqrt_price_at_tick(position_param.tick_lower).unwrap(),
                    get_sqrt_price_at_tick(position_param.tick_upper).unwrap(),
                    position_param.amount_0,
                    position_param.amount_1,
                );

                let (amount_0, amount_1) = get_delta_amounts_signed(
                    start_tick,
                    get_sqrt_price_at_tick(start_tick).unwrap(),
                    position_param.tick_lower,
                    position_param.tick_upper,
                    liquidity as i128,
                )
                    .unwrap();
                sum_amount_0 += amount_0 as u64;
                sum_amount_1 += amount_1 as u64;
                let tick_array_lower_start_index =
                    TickArrayState::get_array_start_index(position_param.tick_lower, tick_spacing);

                if !tick_array_map.contains_key(&tick_array_lower_start_index) {
                    let mut tick_array_refcel = build_tick_array_with_tick_states(
                        pool_state.key(&program_pubkey),
                        tick_array_lower_start_index,
                        tick_spacing,
                        vec![],
                    );
                    let mut tick_array_lower = tick_array_refcel.clone();

                    let tick_lower = tick_array_lower
                        .get_tick_state_mut(position_param.tick_lower, tick_spacing)
                        .unwrap();
                    tick_lower.tick = position_param.tick_lower;
                    tick_lower
                        .update(
                            pool_state.tick_current,
                            i128::try_from(liquidity).unwrap(),
                            0,
                            0,
                            false,
                            &[RaydiumRewardInfo::default(); 3],
                        )
                        .unwrap();

                    tick_array_map.insert(tick_array_lower_start_index, tick_array_refcel);
                } else {
                    let tick_array_lower = tick_array_map
                        .get_mut(&tick_array_lower_start_index)
                        .unwrap();
                    let mut tick_array_lower_borrow_mut = tick_array_lower;
                    let tick_lower = tick_array_lower_borrow_mut
                        .get_tick_state_mut(position_param.tick_lower, tick_spacing)
                        .unwrap();

                    tick_lower
                        .update(
                            pool_state.tick_current,
                            i128::try_from(liquidity).unwrap(),
                            0,
                            0,
                            false,
                            &[RaydiumRewardInfo::default(); 3],
                        )
                        .unwrap();
                }
                let tick_array_upper_start_index =
                    TickArrayState::get_array_start_index(position_param.tick_upper, tick_spacing);
                if !tick_array_map.contains_key(&tick_array_upper_start_index) {
                    let mut tick_array_refcel = build_tick_array_with_tick_states(
                        pool_state.key(&program_pubkey),
                        tick_array_upper_start_index,
                        tick_spacing,
                        vec![],
                    );
                    let mut tick_array_upper = tick_array_refcel.clone();

                    let tick_upper = tick_array_upper
                        .get_tick_state_mut(position_param.tick_upper, tick_spacing)
                        .unwrap();
                    tick_upper.tick = position_param.tick_upper;

                    tick_upper
                        .update(
                            pool_state.tick_current,
                            i128::try_from(liquidity).unwrap(),
                            0,
                            0,
                            true,
                            &[RaydiumRewardInfo::default(); 3],
                        )
                        .unwrap();

                    tick_array_map.insert(tick_array_upper_start_index, tick_array_refcel);
                } else {
                    let tick_array_upper = tick_array_map
                        .get_mut(&tick_array_upper_start_index)
                        .unwrap();

                    let mut tick_array_upperr_borrow_mut = tick_array_upper;
                    let tick_upper = tick_array_upperr_borrow_mut
                        .get_tick_state_mut(position_param.tick_upper, tick_spacing)
                        .unwrap();

                    tick_upper
                        .update(
                            pool_state.tick_current,
                            i128::try_from(liquidity).unwrap(),
                            0,
                            0,
                            true,
                            &[RaydiumRewardInfo::default(); 3],
                        )
                        .unwrap();
                }
                if pool_state.tick_current >= position_param.tick_lower
                    && pool_state.tick_current < position_param.tick_upper
                {
                    pool_state.liquidity = add_delta(
                        pool_state.liquidity,
                        i128::try_from(liquidity).unwrap(),
                    )
                        .unwrap();
                }
            }
            for (tickarray_start_index, tick_array_info) in tick_array_map {
                tick_array_states.push_back(tick_array_info);
                pool_state
                    .flip_tick_array_bit(Some(&bitmap_extension), tickarray_start_index)
                    .unwrap();
            }

            use std::convert::identity;
            if zero_for_one {
                tick_array_states.make_contiguous().sort_by(|a, b| {
                    identity(b.start_tick_index)
                        .cmp(&identity(a.start_tick_index))
                });
            } else {
                tick_array_states.make_contiguous().sort_by(|a, b| {
                    identity(a.start_tick_index)
                        .cmp(&identity(b.start_tick_index))
                });
            }
        }
        // todo
        let bitmap_extension_state = TickArrayBitmapExtension::unpack_data(&bitmap_extension.data.borrow().to_vec());
        // let bitmap_extension_state =
        //     *AccountLoader::<TickArrayBitmapExtension>::try_from(&bitmap_extension)
        //         .unwrap()
        //         .load()
        //         .unwrap()
        //         .deref();

        (
            amm_config,
            pool_state_refcel,
            tick_array_states,
            // todo
            // observation_state,
            bitmap_extension_state,
            sum_amount_0,
            sum_amount_1,
        )
    }

    pub struct TickArrayInfo {
        pub start_tick_index: i32,
        pub ticks: Vec<TickState>,
    }

    pub fn build_tick_array(
        start_index: i32,
        tick_spacing: u16,
        initialized_tick_offsets: Vec<usize>,
    ) -> RefCell<TickArrayState> {
        let mut new_tick_array = TickArrayState::default();
        new_tick_array
            .initialize(start_index, tick_spacing, Pubkey::default())
            .unwrap();

        for offset in initialized_tick_offsets {
            let mut new_tick = TickState::default();
            // Indicates tick is initialized
            new_tick.liquidity_gross = 1;
            new_tick.tick = start_index + (offset * tick_spacing as usize) as i32;
            new_tick_array.ticks[offset] = new_tick;
        }
        RefCell::new(new_tick_array)
    }

    pub fn build_tick_array_with_tick_states(
        pool_id: Pubkey,
        start_index: i32,
        tick_spacing: u16,
        tick_states: Vec<TickState>,
    ) -> TickArrayState {
        let mut new_tick_array = TickArrayState::default();
        new_tick_array
            .initialize(start_index, tick_spacing, pool_id)
            .unwrap();

        for tick_state in tick_states {
            assert!(tick_state.tick != 0);
            let offset = new_tick_array
                .get_tick_offset_in_array(tick_state.tick, tick_spacing)
                .unwrap();
            new_tick_array.ticks[offset] = tick_state;
        }
        new_tick_array
    }

    pub fn build_tick(tick: i32, liquidity_gross: u128, liquidity_net: i128) -> RefCell<TickState> {
        let mut new_tick = TickState::default();
        new_tick.tick = tick;
        new_tick.liquidity_gross = liquidity_gross;
        new_tick.liquidity_net = liquidity_net;
        RefCell::new(new_tick)
    }

    fn build_tick_with_fee_reward_growth(
        tick: i32,
        fee_growth_outside_0_x64: u128,
        fee_growth_outside_1_x64: u128,
        reward_growths_outside_x64: u128,
    ) -> RefCell<TickState> {
        let mut new_tick = TickState::default();
        new_tick.tick = tick;
        new_tick.fee_growth_outside_0_x64 = fee_growth_outside_0_x64;
        new_tick.fee_growth_outside_1_x64 = fee_growth_outside_1_x64;
        new_tick.reward_growths_outside_x64 = [reward_growths_outside_x64, 0, 0];
        RefCell::new(new_tick)
    }

    #[cfg(test)]
    mod cross_tick_array_test {
        use crate::formula::raydium_clmm::swap_internal;
        use super::*;

        #[test]
        fn zero_for_one_base_input_test() {
            let mut tick_current = -32395;
            let mut liquidity = 5124165121219;
            let mut sqrt_price_x64 = 3651942632306380802;
            // let (amm_config, pool_state, mut tick_array_states, observation_state) =
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
                // &mut observation_state.borrow_mut(),
                &None,
                12188240002,
                3049500711113990606,
                true,
                true,
                // oracle::block_timestamp_mock() as u32,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current < tick_current);
            assert!(
                pool_state.borrow().tick_current > -32460
                    && pool_state.borrow().tick_current < -32400
            );
            assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
            assert!(pool_state.borrow().liquidity == (liquidity + 277065331032));
            assert!(amount_0 == 12188240002);

            tick_current = pool_state.borrow().tick_current;
            sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
            liquidity = pool_state.borrow().liquidity;

            // cross the tickarray boundary(-32400) in last step, now tickarray_current is the tickarray with start_index -36000,
            // so we pop the tickarray with start_index -32400
            // in this swap we will cross the tick(-32460), but not reach next tick (-32520)
            tick_array_states.pop_front();
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                // &mut observation_state.borrow_mut(),
                &None,
                121882400020,
                3049500711113990606,
                true,
                true,
                // oracle::block_timestamp_mock() as u32,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current < tick_current);
            assert!(
                pool_state.borrow().tick_current > -32520
                    && pool_state.borrow().tick_current < -32460
            );
            assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
            assert!(pool_state.borrow().liquidity == (liquidity - 536061033698));
            assert!(amount_0 == 121882400020);

            tick_current = pool_state.borrow().tick_current;
            sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
            liquidity = pool_state.borrow().liquidity;

            // swap in tickarray with start_index -36000, cross the tick -32520
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                // &mut observation_state.borrow_mut(),
                &None,
                60941200010,
                3049500711113990606,
                true,
                true,
                // block_timestamp_mock() as u32,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current < tick_current);
            assert!(
                pool_state.borrow().tick_current > -32580
                    && pool_state.borrow().tick_current < -32520
            );
            assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
            assert!(pool_state.borrow().liquidity == (liquidity - 790917615645));
            assert!(amount_0 == 60941200010);
        }
    }

    #[test]
    fn zero_for_one_base_output_test() {
        let mut tick_current = -32395;
        let mut liquidity = 5124165121219;
        let mut sqrt_price_x64 = 3651942632306380802;
        // let (amm_config, pool_state, mut tick_array_states, observation_state) =
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
            // &mut observation_state.borrow_mut(),
            &None,
            477470480,
            3049500711113990606,
            true,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current < tick_current);
        assert!(
            pool_state.borrow().tick_current > -32460
                && pool_state.borrow().tick_current < -32400
        );
        assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity + 277065331032));
        assert!(amount_1 == 477470480);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // cross the tickarray boundary(-32400) in last step, now tickarray_current is the tickarray with start_index -36000,
        // so we pop the tickarray with start_index -32400
        // in this swap we will cross the tick(-32460), but not reach next tick (-32520)
        tick_array_states.pop_front();
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            4751002622,
            3049500711113990606,
            true,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current < tick_current);
        assert!(
            pool_state.borrow().tick_current > -32520
                && pool_state.borrow().tick_current < -32460
        );
        assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 536061033698));
        assert!(amount_1 == 4751002622);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // swap in tickarray with start_index -36000
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            2358130642,
            3049500711113990606,
            true,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current < tick_current);
        assert!(
            pool_state.borrow().tick_current > -32580
                && pool_state.borrow().tick_current < -32520
        );
        assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 790917615645));
        assert!(amount_1 == 2358130642);
    }

    #[test]
    fn one_for_zero_base_input_test() {
        let mut tick_current = -32470;
        let mut liquidity = 5124165121219;
        let mut sqrt_price_x64 = 3638127228312488926;
        // let (amm_config, pool_state, mut tick_array_states, observation_state) =
        let (amm_config, pool_state, mut tick_array_states) =
            build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![
                    TickArrayInfo {
                        start_tick_index: -36000,
                        ticks: vec![
                            build_tick(-32460, 1194569667438, 536061033698).take(),
                            build_tick(-32520, 790917615645, 790917615645).take(),
                            build_tick(-32580, 152146472301, 128451145459).take(),
                            build_tick(-32640, 2625605835354, -1492054447712).take(),
                        ],
                    },
                    TickArrayInfo {
                        start_tick_index: -32400,
                        ticks: vec![
                            build_tick(-32400, 277065331032, -277065331032).take(),
                            build_tick(-29220, 1330680689, -1330680689).take(),
                            build_tick(-28860, 6408486554, -6408486554).take(),
                        ],
                    },
                ],
            );

        // just cross the tickarray boundary(-32460), hasn't reached the next tick array initialized tick
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            887470480,
            5882283448660210779,
            false,
            true,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -32460
                && pool_state.borrow().tick_current < -32400
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity + 536061033698));
        assert!(amount_1 == 887470480);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // cross the tickarray boundary(-32460) in last step, but not reached tick -32400, because -32400 is the next tickarray boundary,
        // so the tickarray_current still is the tick array with start_index -36000
        // in this swap we will cross the tick(-32400), but not reach next tick (-29220)
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            3087470480,
            5882283448660210779,
            false,
            true,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -32400
                && pool_state.borrow().tick_current < -29220
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 277065331032));
        assert!(amount_1 == 3087470480);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // swap in tickarray with start_index -32400, cross the tick -29220
        tick_array_states.pop_front();
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            200941200010,
            5882283448660210779,
            false,
            true,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -29220
                && pool_state.borrow().tick_current < -28860
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 1330680689));
        assert!(amount_1 == 200941200010);
    }

    #[test]
    fn one_for_zero_base_output_test() {
        let mut tick_current = -32470;
        let mut liquidity = 5124165121219;
        let mut sqrt_price_x64 = 3638127228312488926;
        // let (amm_config, pool_state, mut tick_array_states, observation_state) =
        let (amm_config, pool_state, mut tick_array_states) =
            build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![
                    TickArrayInfo {
                        start_tick_index: -36000,
                        ticks: vec![
                            build_tick(-32460, 1194569667438, 536061033698).take(),
                            build_tick(-32520, 790917615645, 790917615645).take(),
                            build_tick(-32580, 152146472301, 128451145459).take(),
                            build_tick(-32640, 2625605835354, -1492054447712).take(),
                        ],
                    },
                    TickArrayInfo {
                        start_tick_index: -32400,
                        ticks: vec![
                            build_tick(-32400, 277065331032, -277065331032).take(),
                            build_tick(-29220, 1330680689, -1330680689).take(),
                            build_tick(-28860, 6408486554, -6408486554).take(),
                        ],
                    },
                ],
            );

        // just cross the tickarray boundary(-32460), hasn't reached the next tick array initialized tick
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            22796232052,
            5882283448660210779,
            false,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -32460
                && pool_state.borrow().tick_current < -32400
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity + 536061033698));
        assert!(amount_0 == 22796232052);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // cross the tickarray boundary(-32460) in last step, but not reached tick -32400, because -32400 is the next tickarray boundary,
        // so the tickarray_current still is the tick array with start_index -36000
        // in this swap we will cross the tick(-32400), but not reach next tick (-29220)
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            79023558189,
            5882283448660210779,
            false,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -32400
                && pool_state.borrow().tick_current < -29220
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 277065331032));
        assert!(amount_0 == 79023558189);

        tick_current = pool_state.borrow().tick_current;
        sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;
        liquidity = pool_state.borrow().liquidity;

        // swap in tickarray with start_index -32400, cross the tick -29220
        tick_array_states.pop_front();
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            // &mut observation_state.borrow_mut(),
            &None,
            4315086194758,
            5882283448660210779,
            false,
            false,
            // oracle::block_timestamp_mock() as u32,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current > tick_current);
        assert!(
            pool_state.borrow().tick_current > -29220
                && pool_state.borrow().tick_current < -28860
        );
        assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
        assert!(pool_state.borrow().liquidity == (liquidity - 1330680689));
        assert!(amount_0 == 4315086194758);
    }

    #[cfg(test)]
    mod find_next_initialized_tick_test {
        use super::*;

        #[test]
        fn zero_for_one_current_tick_array_not_initialized_test() {
            let tick_current = -28776;
            let liquidity = 624165121219;
            let sqrt_price_x64 = get_sqrt_price_at_tick(tick_current).unwrap();
            let (amm_config, pool_state, tick_array_states) = build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![TickArrayInfo {
                    start_tick_index: -32400,
                    ticks: vec![
                        build_tick(-32400, 277065331032, -277065331032).take(),
                        build_tick(-29220, 1330680689, -1330680689).take(),
                        build_tick(-28860, 6408486554, -6408486554).take(),
                    ],
                }],
            );

            // find the first initialzied tick(-28860) and cross it in tickarray
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                12188240002,
                get_sqrt_price_at_tick(-32400).unwrap(),
                true,
                true,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current < tick_current);
            assert!(
                pool_state.borrow().tick_current > -29220
                    && pool_state.borrow().tick_current < -28860
            );
            assert!(pool_state.borrow().sqrt_price_x64 < sqrt_price_x64);
            assert!(pool_state.borrow().liquidity == (liquidity + 6408486554));
            assert!(amount_0 == 12188240002);
        }

        #[test]
        fn one_for_zero_current_tick_array_not_initialized_test() {
            let tick_current = -32405;
            let liquidity = 1224165121219;
            let sqrt_price_x64 = get_sqrt_price_at_tick(tick_current).unwrap();
            let (amm_config, pool_state, tick_array_states) = build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![TickArrayInfo {
                    start_tick_index: -32400,
                    ticks: vec![
                        build_tick(-32400, 277065331032, -277065331032).take(),
                        build_tick(-29220, 1330680689, -1330680689).take(),
                        build_tick(-28860, 6408486554, -6408486554).take(),
                    ],
                }],
            );

            // find the first initialzied tick(-32400) and cross it in tickarray
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                12188240002,
                get_sqrt_price_at_tick(-28860).unwrap(),
                false,
                true,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current > tick_current);
            assert!(
                pool_state.borrow().tick_current > -32400
                    && pool_state.borrow().tick_current < -29220
            );
            assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
            assert!(pool_state.borrow().liquidity == (liquidity - 277065331032));
            assert!(amount_1 == 12188240002);
        }
    }

    #[cfg(test)]
    mod liquidity_insufficient_test {
        use super::*;
        #[test]
        fn no_enough_initialized_tickarray_in_pool_test() {
            let tick_current = -28776;
            let liquidity = 121219;
            let sqrt_price_x64 = get_sqrt_price_at_tick(tick_current).unwrap();
            let (amm_config, pool_state, tick_array_states) = build_swap_param(
                tick_current,
                60,
                sqrt_price_x64,
                liquidity,
                vec![TickArrayInfo {
                    start_tick_index: -32400,
                    ticks: vec![build_tick(-28860, 6408486554, -6408486554).take()],
                }],
            );

            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                12188240002,
                get_sqrt_price_at_tick(-32400).unwrap(),
                true,
                true,
            );
            assert!(result.is_err());
            assert_eq!(
                result.unwrap_err(),
                "missing tick array bitmap extension account"
            );
        }
    }

    #[test]
    fn explain_why_zero_for_one_less_or_equal_current_tick() {
        let tick_current = -28859;
        let mut liquidity = 121219;
        let sqrt_price_x64 = get_sqrt_price_at_tick(tick_current).unwrap();
        let (amm_config, pool_state, tick_array_states) = build_swap_param(
            tick_current,
            60,
            sqrt_price_x64,
            liquidity,
            vec![TickArrayInfo {
                start_tick_index: -32400,
                ticks: vec![
                    build_tick(-32400, 277065331032, -277065331032).take(),
                    build_tick(-29220, 1330680689, -1330680689).take(),
                    build_tick(-28860, 6408486554, -6408486554).take(),
                ],
            }],
        );

        // not cross tick(-28860), but pool.tick_current = -28860
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            25,
            get_sqrt_price_at_tick(-32400).unwrap(),
            true,
            true,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current < tick_current);
        assert!(pool_state.borrow().tick_current == -28860);
        assert!(
            pool_state.borrow().sqrt_price_x64 > get_sqrt_price_at_tick(-28860).unwrap()
        );
        assert!(pool_state.borrow().liquidity == liquidity);
        assert!(amount_0 == 25);

        // just cross tick(-28860), pool.tick_current = -28861
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            3,
            get_sqrt_price_at_tick(-32400).unwrap(),
            true,
            true,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current < tick_current);
        assert!(pool_state.borrow().tick_current == -28861);
        assert!(
            pool_state.borrow().sqrt_price_x64 > get_sqrt_price_at_tick(-28861).unwrap()
        );
        assert!(pool_state.borrow().liquidity == liquidity + 6408486554);
        assert!(amount_0 == 3);

        liquidity = pool_state.borrow().liquidity;

        // we swap just a little amount, let pool tick_current also equal -28861
        // but pool.sqrt_price_x64 > tick_math::get_sqrt_price_at_tick(-28861)
        let (amount_0, amount_1) = swap_internal(
            &amm_config,
            &mut pool_state.borrow_mut(),
            &mut get_tick_array_states_mut(&tick_array_states),
            &None,
            50,
            get_sqrt_price_at_tick(-32400).unwrap(),
            true,
            true,
        )
            .unwrap();
        println!("amount_0:{},amount_1:{}", amount_0, amount_1);
        assert!(pool_state.borrow().tick_current == -28861);
        assert!(
            pool_state.borrow().sqrt_price_x64 > get_sqrt_price_at_tick(-28861).unwrap()
        );
        assert!(pool_state.borrow().liquidity == liquidity);
        assert!(amount_0 == 50);
    }

    #[cfg(test)]
    mod swap_edge_test {
        use super::*;

        #[test]
        fn zero_for_one_swap_edge_case() {
            let mut tick_current = -28859;
            let liquidity = 121219;
            let mut sqrt_price_x64 = get_sqrt_price_at_tick(tick_current).unwrap();
            let (amm_config, pool_state, tick_array_states) = build_swap_param(
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
                        start_tick_index: -28800,
                        ticks: vec![build_tick(-28800, 3726362727, -3726362727).take()],
                    },
                ],
            );

            // zero for one, just cross tick(-28860),  pool.tick_current = -28861 and pool.sqrt_price_x64 = tick_math::get_sqrt_price_at_tick(-28860)
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                27,
                get_sqrt_price_at_tick(-32400).unwrap(),
                true,
                true,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current < tick_current);
            assert!(pool_state.borrow().tick_current == -28861);
            assert!(
                pool_state.borrow().sqrt_price_x64
                    == get_sqrt_price_at_tick(-28860).unwrap()
            );
            assert!(pool_state.borrow().liquidity == liquidity + 6408486554);
            assert!(amount_0 == 27);

            tick_current = pool_state.borrow().tick_current;
            sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;

            // we swap just a little amount, it is completely taken by fees, the sqrt price and the tick will remain the same
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                1,
                get_sqrt_price_at_tick(-32400).unwrap(),
                true,
                true,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current == tick_current);
            assert!(pool_state.borrow().tick_current == -28861);
            assert!(pool_state.borrow().sqrt_price_x64 == sqrt_price_x64);

            tick_current = pool_state.borrow().tick_current;
            sqrt_price_x64 = pool_state.borrow().sqrt_price_x64;

            // reverse swap direction, one_for_zero
            // Actually, the loop for this swap was executed twice because the previous swap happened to have `pool.tick_current` exactly on the boundary that is divisible by `tick_spacing`.
            // In the first iteration of this swap's loop, it found the initial tick (-28860), but at this point, both the initial and final prices were equal to the price at tick -28860.
            // This did not meet the conditions for swapping so both swap_amount_input and swap_amount_output were 0. The actual output was calculated in the second iteration of the loop.
            let (amount_0, amount_1) = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &None,
                10,
                get_sqrt_price_at_tick(-28800).unwrap(),
                false,
                true,
            )
                .unwrap();
            println!("amount_0:{},amount_1:{}", amount_0, amount_1);
            assert!(pool_state.borrow().tick_current > tick_current);
            assert!(pool_state.borrow().sqrt_price_x64 > sqrt_price_x64);
            assert!(
                pool_state.borrow().tick_current > -28860
                    && pool_state.borrow().tick_current <= -28800
            );
        }
    }

    #[cfg(test)]
    mod sqrt_price_limit_optimization_min_specified_test {
        use crate::formula::clmm::raydium_sqrt_price_math::Q64;
        use crate::formula::clmm::raydium_tick_math::{MAX_SQRT_PRICE_X64, MAX_TICK, MIN_SQRT_PRICE_X64, MIN_TICK};
        use super::*;
        #[test]
        fn zero_for_one_base_input_with_min_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = true;
            let is_base_input = true;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX - 1;
            let amount_1 = u64::MAX - 1;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = 1;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified,
                MIN_SQRT_PRICE_X64 + 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }

        #[test]
        fn zero_for_one_base_out_with_min_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = true;
            let is_base_input = false;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX - 1;
            let amount_1 = u64::MAX - 1;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = 1;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified,
                MIN_SQRT_PRICE_X64 + 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }

        #[test]
        fn one_for_zero_base_in_with_min_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = false;
            let is_base_input = true;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX - 1;
            let amount_1 = u64::MAX - 1;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = 1;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified,
                MAX_SQRT_PRICE_X64 - 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }
        #[test]
        fn one_for_zero_base_out_with_min_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = false;
            let is_base_input = false;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX - 1;
            let amount_1 = u64::MAX - 1;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = 1;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified,
                MAX_SQRT_PRICE_X64 - 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }
    }
    #[cfg(test)]
    mod sqrt_price_limit_optimization_max_specified_test {
        use crate::formula::clmm::raydium_sqrt_price_math::Q64;
        use crate::formula::clmm::raydium_tick_math::{MAX_SQRT_PRICE_X64, MAX_TICK, MIN_SQRT_PRICE_X64, MIN_TICK};
        use super::*;
        #[test]
        fn zero_for_one_base_input_with_max_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = true;
            let is_base_input = true;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX / 2;
            let amount_1 = u64::MAX / 2;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = u64::MAX / 2;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified as u128,
                MIN_SQRT_PRICE_X64 + 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }

        #[test]
        fn zero_for_one_base_out_with_max_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = true;
            let is_base_input = false;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX / 2;
            let amount_1 = u64::MAX / 2;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = u64::MAX / 4;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified as u128,
                MIN_SQRT_PRICE_X64 + 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }

        #[test]
        fn one_for_zero_base_in_with_max_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = false;
            let is_base_input = true;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX / 2;
            let amount_1 = u64::MAX / 2;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = u64::MAX / 2;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified as u128,
                MAX_SQRT_PRICE_X64 - 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }
        #[test]
        fn one_for_zero_base_out_with_min_amount_specified() {
            let tick_spacing = 10;
            let zero_for_one = false;
            let is_base_input = false;
            let tick_lower = MIN_TICK + 1;
            let tick_upper = MAX_TICK - 1;
            let tick_current = 0;
            let amount_0 = u64::MAX / 2;
            let amount_1 = u64::MAX / 2;

            let (
                amm_config,
                pool_state,
                tick_array_states,
                bitmap_extension_state,
                sum_amount_0,
                sum_amount_1,
            ) = setup_swap_test(
                tick_current,
                tick_spacing as u16,
                vec![OpenPositionParam {
                    amount_0: amount_0,
                    amount_1: amount_1,
                    tick_lower: tick_lower,
                    tick_upper: tick_upper,
                }],
                zero_for_one,
            );
            println!(
                "sum_amount_0: {}, sum_amount_1: {}",
                sum_amount_0, sum_amount_1,
            );
            let amount_specified = u64::MAX / 4;
            let result = swap_internal(
                &amm_config,
                &mut pool_state.borrow_mut(),
                &mut get_tick_array_states_mut(&tick_array_states),
                &Some(&bitmap_extension_state),
                amount_specified as u128,
                MAX_SQRT_PRICE_X64 - 1,
                zero_for_one,
                is_base_input,
            );
            println!("{:#?}", result);
            let pool = pool_state.borrow();
            let sqrt_price_x64 = pool.sqrt_price_x64;
            let sqrt_price = sqrt_price_x64 as f64 / Q64 as f64;
            println!("price: {}", sqrt_price * sqrt_price);
        }
    }
}