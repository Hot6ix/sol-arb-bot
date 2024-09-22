use std::cell::RefMut;
use std::collections::VecDeque;
use std::io::Error;
use std::mem::swap;
use std::ops::Mul;
use num_bigfloat::BigFloat;
use crate::formula::clmm::sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output, Q64};
use crate::r#struct::pools::{AmmConfig, RaydiumClmmMarket};

// pub fn swap_internal(
//     amm_config: &AmmConfig,
//     pool_state: &mut RefMut<RaydiumClmmMarket>,
//     tick_array_states: &mut VecDeque<RefMut<TickArrayState>>,
//     tickarray_bitmap_extension: &Option<TickArrayBitmapExtension>,
//     amount_specified: u64,
//     sqrt_price_limit_x64: u128,
//     zero_for_one: bool,
//     is_base_input: bool,
// ) {
//     let (mut is_match_pool_current_tick_array, mut first_valid_tick_array_start_index);
//
//     let mut tick_array_current;
// }

pub fn compute_swap_step(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    amount_remaining: u128,
    fee_rate: u128,
    fee_rate_denominator: u128,
    is_base_input: bool,
    zero_for_one: bool,
) -> Result<SwapStep, Error> {
    let mut swap_step = SwapStep::default();

    let fee_rate_bf = BigFloat::from(fee_rate);
    let fee_rate_denominator_bf = BigFloat::from(fee_rate_denominator);

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

pub fn calculate_amount_in_range(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> u128 {
    if is_base_input {
        if zero_for_one {
            get_delta_amount_0_unsigned(
                sqrt_price_target_x64,
                sqrt_price_current_x64,
                liquidity,
                true
            )
        }
        else {
            get_delta_amount_1_unsigned(
                sqrt_price_current_x64,
                sqrt_price_target_x64,
                liquidity,
                true
            )
        }
    }
    else {
        if zero_for_one {
            get_delta_amount_1_unsigned(
                sqrt_price_target_x64,
                sqrt_price_current_x64,
                liquidity,
                false
            )
        }
        else {
            get_delta_amount_0_unsigned(
                sqrt_price_current_x64,
                sqrt_price_target_x64,
                liquidity,
                false
            )
        }
    }
}

pub fn get_delta_amount_0_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64)
    }

    let q64 = BigFloat::from(Q64);
    let num1 = BigFloat::from(liquidity).mul(&q64);
    let num2 = BigFloat::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64);

    if round_up {
        num1.mul(&num2).div(&BigFloat::from(sqrt_ratio_b_x64)).ceil().div(&BigFloat::from(sqrt_ratio_a_x64)).ceil().to_u128().unwrap()
    }
    else {
        num1.mul(&num2).div(&BigFloat::from(sqrt_ratio_b_x64)).floor().div(&BigFloat::from(sqrt_ratio_a_x64)).to_u128().unwrap()
    }
}

pub fn get_delta_amount_1_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> u128 {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64)
    }

    let q64 = BigFloat::from(Q64);
    if round_up {
        BigFloat::from(liquidity).mul(&BigFloat::from(sqrt_ratio_b_x64).sub(&BigFloat::from(sqrt_ratio_a_x64)))
            .div(&q64)
            .ceil()
            .to_u128().unwrap()
    }
    else {
        BigFloat::from(liquidity).mul(&BigFloat::from(sqrt_ratio_b_x64).sub(&BigFloat::from(sqrt_ratio_a_x64)))
            .div(&q64)
            .floor()
            .to_u128().unwrap()
    }
}

#[derive(Default, Debug)]
pub struct SwapStep {
    pub sqrt_price_next_x64: u128,
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee_amount: u128,
}