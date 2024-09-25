use std::collections::VecDeque;
use std::ops::{Mul, Neg};
use num_bigfloat::BigFloat;
use num_traits::Zero;
use numext_fixed_uint::U128;
use crate::formula::clmm::constant::{FEE_RATE_DENOMINATOR_VALUE, MAX_SQRT_PRICE_X64, MAX_TICK, MIN_SQRT_PRICE_X64, MIN_TICK};
use crate::formula::clmm::sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output, Q64, tick_to_sqrt_price_x64};
use crate::formula::clmm::swap_state::{add_delta, calculate_amount_in_range, get_delta_amount_0_unsigned, get_delta_amount_1_unsigned, get_tick_at_sqrt_price, StepComputations, SwapState, SwapStep};
use crate::formula::clmm::tick_array::{TickArrayBitmapExtension, TickArrayState, TickState};
use crate::r#struct::pools::{AmmConfig, RaydiumClmmMarket};

pub fn swap_internal(
    amm_config: &AmmConfig,
    pool_state: &mut RaydiumClmmMarket,
    tick_array_states: &mut VecDeque<TickArrayState>,
    tick_array_bitmap_extension: &Option<&TickArrayBitmapExtension>,
    amount_specified: u128,
    sqrt_price_limit_x64: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> Result<(u64, u64), &'static str> {
    if amount_specified.is_zero() {
        return Err("Zero amount specified")
    }

    let sqrt_price_limit_x64 = if sqrt_price_limit_x64 == 0 {
        if zero_for_one {
            MIN_SQRT_PRICE_X64 + 1
        } else {
            MAX_SQRT_PRICE_X64 - 1
        }
    } else {
        sqrt_price_limit_x64
    };

    if zero_for_one {
        if sqrt_price_limit_x64 < MIN_SQRT_PRICE_X64 {
            return Err("sqrt_price_limit_x64 must greater than MIN_SQRT_PRICE_X64");
        }
        if sqrt_price_limit_x64 >= pool_state.sqrt_price_x64 {
            return Err("sqrt_price_limit_x64 must smaller than current");
        }
    } else {
        if sqrt_price_limit_x64 > MAX_SQRT_PRICE_X64 {
            return Err("sqrt_price_limit_x64 must smaller than MAX_SQRT_PRICE_X64");
        }
        if sqrt_price_limit_x64 <= pool_state.sqrt_price_x64 {
            return Err("sqrt_price_limit_x64 must greater than current");
        }
    }

    let liquidity_start = pool_state.liquidity;

    let mut state = SwapState {
        amount_specified_remaining: amount_specified,
        amount_calculated: 0,
        sqrt_price_x64: pool_state.sqrt_price_x64,
        tick: pool_state.tick_current,
        fee_growth_global_x64: if zero_for_one {
            pool_state.fee_growth_global_0_x64
        } else {
            pool_state.fee_growth_global_1_x64
        },
        fee_amount: 0,
        protocol_fee: 0,
        fund_fee: 0,
        liquidity: liquidity_start,
    };

    let (mut is_match_pool_current_tick_array, first_valid_tick_array_start_index) =
        pool_state.get_first_initialized_tick_array(tick_array_bitmap_extension, zero_for_one)?;
    let mut current_valid_tick_array_start_index = first_valid_tick_array_start_index;

    let mut tick_array_current = tick_array_states.pop_front().unwrap();

    for _ in 0..tick_array_states.len() {
        if tick_array_current.start_tick_index == current_valid_tick_array_start_index {
            break;
        }
        tick_array_current = tick_array_states
            .pop_front()
            .ok_or("not enough tick array account")?;
    }

    if tick_array_current.start_tick_index != current_valid_tick_array_start_index {
        return Err("invalid first tick array account")
    }

    /////////////////////////////////////// start of while loop
    while state.amount_specified_remaining != 0
        && state.sqrt_price_x64 != sqrt_price_limit_x64
        && state.tick < MAX_TICK
        && state.tick > MIN_TICK
    {
        let mut step = StepComputations::default();
        step.sqrt_price_start_x64 = state.sqrt_price_x64;

        let mut next_initialized_tick = if let Some(tick_state) = tick_array_current
            .next_initialized_tick(state.tick, pool_state.tick_spacing, zero_for_one)?
        {
            Box::new(*tick_state)
        } else {
            if !is_match_pool_current_tick_array {
                is_match_pool_current_tick_array = true;
                Box::new(tick_array_current.first_initialized_tick(zero_for_one)?.clone())
            } else {
                Box::new(TickState::default())
            }
        };

        if !next_initialized_tick.is_initialized() {
            let next_initialized_tick_array_index = pool_state
                .next_initialized_tick_array_start_index(
                    tick_array_bitmap_extension,
                    current_valid_tick_array_start_index,
                    zero_for_one,
                )?;
            if next_initialized_tick_array_index.is_none() {
                return Err("liquidity insufficient");
            }

            while tick_array_current.start_tick_index != next_initialized_tick_array_index.unwrap() {
                tick_array_current = tick_array_states
                    .pop_front()
                    .ok_or("not enough tick array account")?;
            }
            current_valid_tick_array_start_index = next_initialized_tick_array_index.unwrap();

            let first_initialized_tick = tick_array_current.first_initialized_tick(zero_for_one)?;
            next_initialized_tick = Box::new(first_initialized_tick.clone());
        }
        step.tick_next = next_initialized_tick.tick;
        step.initialized = next_initialized_tick.is_initialized();

        if step.tick_next < MIN_TICK {
            step.tick_next = MIN_TICK;
        } else if step.tick_next > MAX_TICK {
            step.tick_next = MAX_TICK;
        }
        // todo
        // step.sqrt_price_next_x64 = get_sqrt_price_at_tick(step.tick_next)?;
        step.sqrt_price_next_x64 = tick_to_sqrt_price_x64(&step.tick_next).ok_or("tick_to_sqrt_price_x64 failed")?;

        let target_price = if (zero_for_one && step.sqrt_price_next_x64 < sqrt_price_limit_x64)
            || (!zero_for_one && step.sqrt_price_next_x64 > sqrt_price_limit_x64)
        {
            sqrt_price_limit_x64
        } else {
            step.sqrt_price_next_x64
        };

        let swap_step = compute_swap_step(
            step.sqrt_price_start_x64,
            target_price,
            state.liquidity,
            state.amount_specified_remaining,
            u128::from(amm_config.trade_fee_rate),
            is_base_input,
            zero_for_one
        )?;

        // if zero_for_one {
        //     if swap_step.sqrt_price_next_x64 < target_price {
        //         return Err("invalid result")
        //     }
        // } else {
        //     if target_price < swap_step.sqrt_price_next_x64 {
        //         return Err("invalid result")
        //     }
        // }

        state.sqrt_price_x64 = swap_step.sqrt_price_next_x64;
        step.amount_in = swap_step.amount_in;
        step.amount_out = swap_step.amount_out;
        step.fee_amount = swap_step.fee_amount;

        if is_base_input {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_in + step.fee_amount)
                .unwrap();
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step.amount_out)
                .unwrap();
        } else {
            state.amount_specified_remaining = state
                .amount_specified_remaining
                .checked_sub(step.amount_out)
                .unwrap();

            let step_amount_calculate = step
                .amount_in
                .checked_add(step.fee_amount)
                .expect("calculate overflow");
            state.amount_calculated = state
                .amount_calculated
                .checked_add(step_amount_calculate)
                .expect("calculate overflow");
        }

        // let step_fee_amount = step.fee_amount;
        //
        // if amm_config.protocol_fee_rate > 0 {
        //     let delta = U128::from(step_fee_amount)
        //         .checked_mul(&U128::from(amm_config.protocol_fee_rate))
        //         .unwrap()
        //         .checked_div(&U128::from(FEE_RATE_DENOMINATOR_VALUE))
        //         .unwrap()
        //         .to_le_bytes();
        //     step.fee_amount = step.fee_amount.checked_sub(u128::from_le_bytes(delta)).unwrap();
        //     state.protocol_fee = state.protocol_fee.checked_add(u128::from_le_bytes(delta)).unwrap();
        // }
        //
        // if amm_config.fund_fee_rate > 0 {
        //     let delta = U128::from(step_fee_amount)
        //         .checked_mul(&U128::from(amm_config.fund_fee_rate))
        //         .unwrap()
        //         .checked_div(&U128::from(FEE_RATE_DENOMINATOR_VALUE))
        //         .unwrap()
        //         .to_le_bytes();
        //     step.fee_amount = step.fee_amount.checked_sub(u128::from_le_bytes(delta)).unwrap();
        //     state.fund_fee = state.fund_fee.checked_add(u128::from_le_bytes(delta)).unwrap();
        // }

        // if state.liquidity > 0 {
        //     // todo
        //     // let fee_growth_global_x64_delta = U128::from(step.fee_amount)
        //     //     .mul_div_floor(Q64, U128::from(state.liquidity))
        //     //     .unwrap()
        //     //     .as_u128();
        //
        //     let fee_growth_global_x64_delta = BigFloat::from(step.fee_amount)
        //         .mul(&BigFloat::from(Q64))
        //         .div(&BigFloat::from(state.liquidity))
        //         .floor()
        //         .to_u128()
        //         .unwrap();
        //
        //     state.fee_growth_global_x64 = state
        //         .fee_growth_global_x64
        //         .checked_add(fee_growth_global_x64_delta)
        //         .unwrap();
        //     state.fee_amount = state.fee_amount.checked_add(step.fee_amount).unwrap();
        // }

        if state.sqrt_price_x64 == step.sqrt_price_next_x64 {
            if step.initialized {
                let mut liquidity_net = next_initialized_tick.liquidity_net;
                // tick_array_current.update_tick_state(
                //     next_initialized_tick.tick,
                //     pool_state.tick_spacing.into(),
                //     *next_initialized_tick,
                // )?;

                if zero_for_one {
                    liquidity_net = liquidity_net.neg();
                }
                state.liquidity = add_delta(state.liquidity, liquidity_net)?;
            }

            state.tick = if zero_for_one {
                step.tick_next - 1
            } else {
                step.tick_next
            };
        } else if state.sqrt_price_x64 != step.sqrt_price_start_x64 {
            state.tick = get_tick_at_sqrt_price(state.sqrt_price_x64)?;
        }
    }
    /////////////////////////////////////// end of while loop

    // if state.tick != pool_state.tick_current {
    //     pool_state.tick_current = state.tick;
    // }
    //
    // pool_state.sqrt_price_x64 = state.sqrt_price_x64;
    //
    // if liquidity_start != state.liquidity {
    //     pool_state.liquidity = state.liquidity;
    // }

    let (amount_0, amount_1) = if zero_for_one == is_base_input {
        (
            amount_specified
                .checked_sub(state.amount_specified_remaining)
                .unwrap(),
            state.amount_calculated,
        )
    } else {
        (
            state.amount_calculated,
            amount_specified
                .checked_sub(state.amount_specified_remaining)
                .unwrap(),
        )
    };

    // if zero_for_one {
    //     pool_state.fee_growth_global_0_x64 = state.fee_growth_global_x64;
    //     pool_state.total_fees_token_0 = pool_state
    //         .total_fees_token_0
    //         .checked_add(state.fee_amount as u64)
    //         .unwrap();
    //
    //     if state.protocol_fee > 0 {
    //         pool_state.protocol_fees_token_0 = pool_state
    //             .protocol_fees_token_0
    //             .checked_add(state.protocol_fee as u64)
    //             .unwrap();
    //     }
    //     if state.fund_fee > 0 {
    //         pool_state.fund_fees_token_0 = pool_state
    //             .fund_fees_token_0
    //             .checked_add(state.fund_fee as u64)
    //             .unwrap();
    //     }
    //     pool_state.swap_in_amount_token_0 = pool_state
    //         .swap_in_amount_token_0
    //         .checked_add(u128::from(amount_0))
    //         .unwrap();
    //     pool_state.swap_out_amount_token_1 = pool_state
    //         .swap_out_amount_token_1
    //         .checked_add(u128::from(amount_1))
    //         .unwrap();
    // } else {
    //     pool_state.fee_growth_global_1_x64 = state.fee_growth_global_x64;
    //     pool_state.total_fees_token_1 = pool_state
    //         .total_fees_token_1
    //         .checked_add(state.fee_amount as u64)
    //         .unwrap();
    //
    //     if state.protocol_fee > 0 {
    //         pool_state.protocol_fees_token_1 = pool_state
    //             .protocol_fees_token_1
    //             .checked_add(state.protocol_fee as u64)
    //             .unwrap();
    //     }
    //     if state.fund_fee > 0 {
    //         pool_state.fund_fees_token_1 = pool_state
    //             .fund_fees_token_1
    //             .checked_add(state.fund_fee as u64)
    //             .unwrap();
    //     }
    //     pool_state.swap_in_amount_token_1 = pool_state
    //         .swap_in_amount_token_1
    //         .checked_add(u128::from(amount_1))
    //         .unwrap();
    //     pool_state.swap_out_amount_token_0 = pool_state
    //         .swap_out_amount_token_0
    //         .checked_add(u128::from(amount_0))
    //         .unwrap();
    // }

    Ok((amount_0 as u64, amount_1 as u64))
}

pub fn compute_swap_step(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    amount_remaining: u128,
    fee_rate: u128,
    is_base_input: bool,
    zero_for_one: bool,
) -> Result<SwapStep, &'static str> {
    let mut swap_step = SwapStep::default();

    let fee_rate_bf = BigFloat::from(fee_rate);
    let fee_rate_denominator_bf = BigFloat::from(FEE_RATE_DENOMINATOR_VALUE);

    if is_base_input {
        let amount_remaining_less_fee = BigFloat::from(amount_remaining).mul(fee_rate_denominator_bf.sub(&fee_rate_bf)).div(&fee_rate_denominator_bf).floor().to_u128().unwrap();

        let amount_in = calculate_amount_in_range(
            sqrt_price_current_x64,
            sqrt_price_target_x64,
            liquidity,
            zero_for_one,
            is_base_input
        );

        swap_step.amount_in = amount_in;
        swap_step.sqrt_price_next_x64 =
            if amount_remaining_less_fee >= swap_step.amount_in {
                sqrt_price_target_x64
            }
            else {
                get_next_sqrt_price_from_input(
                    sqrt_price_current_x64,
                    liquidity,
                    amount_remaining_less_fee,
                    zero_for_one
                )
            }
    }
    else {
        let amount_out = calculate_amount_in_range(
            sqrt_price_current_x64,
            sqrt_price_target_x64,
            liquidity,
            zero_for_one,
            is_base_input
        );

        swap_step.amount_out = amount_out;
        swap_step.sqrt_price_next_x64 =
            if amount_remaining >= swap_step.amount_out {
                sqrt_price_target_x64
            }
            else {
                get_next_sqrt_price_from_output(
                    sqrt_price_current_x64,
                    liquidity,
                    amount_remaining,
                    zero_for_one
                )
            }
    }

    let is_exceed = sqrt_price_target_x64 == swap_step.sqrt_price_next_x64;
    if zero_for_one {
        if !(is_exceed && is_base_input) {
            swap_step.amount_in = get_delta_amount_0_unsigned(
                swap_step.sqrt_price_next_x64,
                sqrt_price_current_x64,
                liquidity,
                true
            );
        }
        if !(is_exceed && !is_base_input) {
            swap_step.amount_out = get_delta_amount_1_unsigned(
                swap_step.sqrt_price_next_x64,
                sqrt_price_current_x64,
                liquidity,
                false
            );
        }
    }
    else {
        if !(is_exceed && is_base_input) {
            swap_step.amount_in = get_delta_amount_1_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                true
            );
        }

        if !(is_exceed && !is_base_input) {
            swap_step.amount_out = get_delta_amount_0_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                false
            );
        }
    }

    if !is_base_input && swap_step.amount_out > amount_remaining {
        swap_step.amount_out = amount_remaining;
    }

    swap_step.fee_amount =
        if is_base_input && swap_step.sqrt_price_next_x64 != sqrt_price_target_x64 {
            amount_remaining - swap_step.amount_in
        }
        else {
            // swap_step.amount_in * fee_rate / (fee_rate_denominator - fee_rate)
            BigFloat::from(swap_step.amount_in).mul(&fee_rate_bf).div(&fee_rate_denominator_bf.sub(&fee_rate_bf)).ceil().to_u128().unwrap()
        };

    Ok(swap_step)
}