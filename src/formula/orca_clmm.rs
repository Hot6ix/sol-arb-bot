use crate::formula::clmm::concentrated_liquidity::compute_swap_step;
use crate::formula::clmm::constant::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64};
use crate::formula::clmm::orca_swap_state::{checked_mul_div, next_tick_cross_update, NO_EXPLICIT_SQRT_PRICE_LIMIT, NUM_REWARDS, PostSwapUpdate, PROTOCOL_FEE_RATE_MUL_VALUE, Q64_RESOLUTION, SwapTickSequence, Tick, TICK_ARRAY_SIZE, TickUpdate};
use crate::formula::clmm::raydium_swap_state::add_delta;
use crate::formula::clmm::sqrt_price_math::{sqrt_price_x64_to_tick, tick_to_sqrt_price_x64};
use crate::r#struct::pools::{OrcaClmmMarket, WhirlpoolRewardInfo};

pub fn swap_internal(
    whirlpool: &OrcaClmmMarket,
    swap_tick_sequence: &mut SwapTickSequence,
    amount: u64,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    timestamp: u64,
) -> Result<PostSwapUpdate, &'static str> {
    let adjusted_sqrt_price_limit = if sqrt_price_limit == NO_EXPLICIT_SQRT_PRICE_LIMIT {
        if a_to_b {
            MIN_SQRT_PRICE_X64
        } else {
            MAX_SQRT_PRICE_X64
        }
    } else {
        sqrt_price_limit
    };

    if !(MIN_SQRT_PRICE_X64..=MAX_SQRT_PRICE_X64).contains(&adjusted_sqrt_price_limit) {
        return Err("ErrorCode::SqrtPriceOutOfBounds");
    }

    if a_to_b && adjusted_sqrt_price_limit > whirlpool.sqrt_price
        || !a_to_b && adjusted_sqrt_price_limit < whirlpool.sqrt_price
    {
        return Err("ErrorCode::InvalidSqrtPriceLimitDirection");
    }

    if amount == 0 {
        return Err("ErrorCode::ZeroTradableAmount");
    }

    let tick_spacing = whirlpool.tick_spacing;
    let fee_rate = whirlpool.fee_rate;
    let protocol_fee_rate = whirlpool.protocol_fee_rate;
    let next_reward_infos = next_whirlpool_reward_infos(whirlpool, timestamp)?;

    let mut amount_remaining: u128 = amount as u128;
    let mut amount_calculated: u128 = 0;
    let mut curr_sqrt_price = whirlpool.sqrt_price;
    let mut curr_tick_index = whirlpool.tick_current_index;
    let mut curr_liquidity = whirlpool.liquidity;
    let mut curr_protocol_fee: u64 = 0;
    let mut curr_array_index: usize = 0;
    let mut curr_fee_growth_global_input = if a_to_b {
        whirlpool.fee_growth_global_a
    } else {
        whirlpool.fee_growth_global_b
    };

    while amount_remaining > 0 && adjusted_sqrt_price_limit != curr_sqrt_price {
        let (next_array_index, next_tick_index) = swap_tick_sequence
            .get_next_initialized_tick_index(
                curr_tick_index,
                tick_spacing,
                a_to_b,
                curr_array_index,
            )?;

        let (next_tick_sqrt_price, sqrt_price_target) =
            get_next_sqrt_prices(next_tick_index, adjusted_sqrt_price_limit, a_to_b);

        let swap_computation = compute_swap_step(
            curr_sqrt_price,
            sqrt_price_target,
            curr_liquidity,
            amount_remaining,
            fee_rate as u128,
            amount_specified_is_input,
            a_to_b,
        )?;

        if amount_specified_is_input {
            amount_remaining = amount_remaining
                .checked_sub(swap_computation.amount_in)
                .ok_or("ErrorCode::AmountRemainingOverflow")?;
            amount_remaining = amount_remaining
                .checked_sub(swap_computation.fee_amount)
                .ok_or("ErrorCode::AmountRemainingOverflow")?;

            amount_calculated = amount_calculated
                .checked_add(swap_computation.amount_out)
                .ok_or("ErrorCode::AmountCalcOverflow")?;
        } else {
            amount_remaining = amount_remaining
                .checked_sub(swap_computation.amount_out)
                .ok_or("ErrorCode::AmountRemainingOverflow")?;

            amount_calculated = amount_calculated
                .checked_add(swap_computation.amount_in)
                .ok_or("ErrorCode::AmountCalcOverflow")?;
            amount_calculated = amount_calculated
                .checked_add(swap_computation.fee_amount)
                .ok_or("ErrorCode::AmountCalcOverflow")?;
        }

        let (next_protocol_fee, next_fee_growth_global_input) = calculate_fees(
            swap_computation.fee_amount as u64,
            protocol_fee_rate,
            curr_liquidity,
            curr_protocol_fee,
            curr_fee_growth_global_input,
        );
        curr_protocol_fee = next_protocol_fee;
        curr_fee_growth_global_input = next_fee_growth_global_input;

        if swap_computation.sqrt_price_next_x64 == next_tick_sqrt_price {
            let (next_tick, next_tick_initialized) = swap_tick_sequence
                .get_tick(next_array_index, next_tick_index, tick_spacing)
                .map_or_else(|_| (None, false), |tick| (Some(tick), tick.initialized));

            if next_tick_initialized {
                let (fee_growth_global_a, fee_growth_global_b) = if a_to_b {
                    (curr_fee_growth_global_input, whirlpool.fee_growth_global_b)
                } else {
                    (whirlpool.fee_growth_global_a, curr_fee_growth_global_input)
                };

                // let signed_liquidity_net = if a_to_b {
                //     -next_tick.liquidity_net
                // } else {
                //     next_tick.liquidity_net
                // };
                // let next_liquidity = add_delta(curr_liquidity, signed_liquidity_net)?;
                ///////////////////////
                let (update, next_liquidity) = calculate_update(
                    next_tick.unwrap(),
                    a_to_b,
                    curr_liquidity,
                    fee_growth_global_a,
                    fee_growth_global_b,
                    &next_reward_infos,
                )?;

                curr_liquidity = next_liquidity;
                swap_tick_sequence.update_tick(
                    next_array_index,
                    next_tick_index,
                    tick_spacing,
                    &update,
                )?;
            }

            let tick_offset = swap_tick_sequence.get_tick_offset(
                next_array_index,
                next_tick_index,
                tick_spacing,
            )?;

            curr_array_index = if (a_to_b && tick_offset == 0)
                || (!a_to_b && tick_offset == TICK_ARRAY_SIZE as isize - 1)
            {
                next_array_index + 1
            } else {
                next_array_index
            };

            curr_tick_index = if a_to_b {
                next_tick_index - 1
            } else {
                next_tick_index
            };
        } else if swap_computation.sqrt_price_next_x64 != curr_sqrt_price {
            // curr_tick_index = tick_index_from_sqrt_price(&swap_computation.sqrt_price_next_x64);
            curr_tick_index = sqrt_price_x64_to_tick(&swap_computation.sqrt_price_next_x64).unwrap();
        }

        curr_sqrt_price = swap_computation.sqrt_price_next_x64;
    }

    if amount_remaining > 0 && !amount_specified_is_input && sqrt_price_limit == NO_EXPLICIT_SQRT_PRICE_LIMIT {
        return Err("ErrorCode::PartialFillError");
    }

    let (amount_a, amount_b) = if a_to_b == amount_specified_is_input {
        (amount - amount_remaining as u64, amount_calculated as u64)
    } else {
        (amount_calculated as u64, amount - amount_remaining as u64)
    };

    Ok(PostSwapUpdate {
        amount_a,
        amount_b,
        next_liquidity: curr_liquidity,
        next_tick_index: curr_tick_index,
        next_sqrt_price: curr_sqrt_price,
        next_fee_growth_global: curr_fee_growth_global_input,
        next_reward_infos: [WhirlpoolRewardInfo::default(); 3],
        next_protocol_fee: curr_protocol_fee,
    })
}

fn get_next_sqrt_prices(
    next_tick_index: i32,
    sqrt_price_limit: u128,
    a_to_b: bool,
) -> (u128, u128) {
    // let next_tick_price = sqrt_price_from_tick_index(next_tick_index);
    let next_tick_price = tick_to_sqrt_price_x64(&next_tick_index).unwrap();
    let next_sqrt_price_limit = if a_to_b {
        sqrt_price_limit.max(next_tick_price)
    } else {
        sqrt_price_limit.min(next_tick_price)
    };
    (next_tick_price, next_sqrt_price_limit)
}

fn calculate_update(
    tick: &Tick,
    a_to_b: bool,
    liquidity: u128,
    fee_growth_global_a: u128,
    fee_growth_global_b: u128,
    reward_infos: &[WhirlpoolRewardInfo; NUM_REWARDS],
) -> Result<(TickUpdate, u128), &'static str> {
    // Use updated fee_growth for crossing tick
    // Use -liquidity_net if going left, +liquidity_net going right
    let signed_liquidity_net = if a_to_b {
        -tick.liquidity_net
    } else {
        tick.liquidity_net
    };

    let update =
        next_tick_cross_update(tick, fee_growth_global_a, fee_growth_global_b, reward_infos)?;

    // Update the global liquidity to reflect the new current tick
    // let next_liquidity = add_liquidity_delta(liquidity, signed_liquidity_net)?;
    let next_liquidity = add_delta(liquidity, signed_liquidity_net)?;

    Ok((update, next_liquidity))
}

fn calculate_fees(
    fee_amount: u64,
    protocol_fee_rate: u16,
    curr_liquidity: u128,
    curr_protocol_fee: u64,
    curr_fee_growth_global_input: u128,
) -> (u64, u128) {
    let mut next_protocol_fee = curr_protocol_fee;
    let mut next_fee_growth_global_input = curr_fee_growth_global_input;
    let mut global_fee = fee_amount;
    if protocol_fee_rate > 0 {
        let delta = calculate_protocol_fee(global_fee, protocol_fee_rate);
        global_fee -= delta;
        next_protocol_fee = next_protocol_fee.wrapping_add(delta);
    }

    if curr_liquidity > 0 {
        next_fee_growth_global_input = next_fee_growth_global_input
            .wrapping_add(((global_fee as u128) << Q64_RESOLUTION) / curr_liquidity);
    }
    (next_protocol_fee, next_fee_growth_global_input)
}

fn calculate_protocol_fee(global_fee: u64, protocol_fee_rate: u16) -> u64 {
    ((global_fee as u128) * (protocol_fee_rate as u128) / PROTOCOL_FEE_RATE_MUL_VALUE)
        .try_into()
        .unwrap()
}

pub fn next_whirlpool_reward_infos(
    whirlpool: &OrcaClmmMarket,
    next_timestamp: u64,
) -> Result<[WhirlpoolRewardInfo; NUM_REWARDS], &'static str> {
    let curr_timestamp = whirlpool.reward_last_updated_timestamp;
    if next_timestamp < curr_timestamp {
        return Err("ErrorCode::InvalidTimestamp");
    }

    if whirlpool.liquidity == 0 || next_timestamp == curr_timestamp {
        return Ok(whirlpool.reward_infos);
    }

    let mut next_reward_infos = whirlpool.reward_infos;
    let time_delta = u128::from(next_timestamp - curr_timestamp);
    for reward_info in next_reward_infos.iter_mut() {
        if !reward_info.initialized() {
            continue;
        }

        let reward_growth_delta = checked_mul_div(
            time_delta,
            reward_info.emissions_per_second_x64,
            whirlpool.liquidity,
        )
            .unwrap_or(0);

        let curr_growth_global = reward_info.growth_global_x64;
        reward_info.growth_global_x64 = curr_growth_global.wrapping_add(reward_growth_delta);
    }

    Ok(next_reward_infos)
}

#[cfg(test)]
mod swap_test {
    use crate::formula::clmm::orca_swap_state::SwapTickSequence;
    use crate::formula::clmm::sqrt_price_math::tick_to_sqrt_price_x64;
    use crate::formula::clmm::test::liquidity_test_fixture::create_whirlpool_reward_infos;
    use crate::formula::clmm::test::swap_test_fixture::{assert_swap, assert_swap_tick_state, SwapTestExpectation, SwapTestFixture, SwapTestFixtureInfo, TestTickInfo, TickExpectation, TS_8};

    #[test]
    fn zero_l_across_tick_range_b_to_a() {
        let swap_test_info = SwapTestFixture::new(SwapTestFixtureInfo {
            tick_spacing: TS_8,
            liquidity: 0,
            curr_tick_index: 255,
            start_tick_index: 0,
            trade_amount: 100_000,
            // sqrt_price_limit: sqrt_price_from_tick_index(1720),
            sqrt_price_limit: tick_to_sqrt_price_x64(&1720i32).unwrap(),
            amount_specified_is_input: false,
            a_to_b: false,
            array_1_ticks: &vec![TestTickInfo {
                // p1
                index: 448,
                liquidity_net: 0,
                ..Default::default()
            }],
            array_2_ticks: Some(&vec![TestTickInfo {
                // p1
                index: 720,
                liquidity_net: 0,
                ..Default::default()
            }]),
            array_3_ticks: Some(&vec![]),
            reward_infos: create_whirlpool_reward_infos(100, 10),
            fee_growth_global_a: 100,
            fee_growth_global_b: 100,
            ..Default::default()
        });
        let mut tick_sequence = SwapTickSequence::new(
            swap_test_info.tick_arrays[0].clone(),
            Some(swap_test_info.tick_arrays[1].clone()),
            Some(swap_test_info.tick_arrays[2].clone()),
        );
        let post_swap = swap_test_info.run(&mut tick_sequence, 100);
        assert_swap(
            &post_swap,
            &SwapTestExpectation {
                traded_amount_a: 0,
                traded_amount_b: 0,
                end_tick_index: 1720,
                end_liquidity: 0,
                end_reward_growths: [10, 10, 10],
            },
        );
        let tick_lower = tick_sequence.get_tick(0, 448, TS_8).unwrap();
        assert_swap_tick_state(
            tick_lower,
            &TickExpectation {
                fee_growth_outside_a: 100,
                fee_growth_outside_b: 100,
                reward_growths_outside: [10, 10, 10],
            },
        );
        let tick_upper = tick_sequence.get_tick(1, 720, TS_8).unwrap();
        assert_swap_tick_state(
            tick_upper,
            &TickExpectation {
                fee_growth_outside_a: 100,
                fee_growth_outside_b: 100,
                reward_growths_outside: [10, 10, 10],
            },
        );
    }
}