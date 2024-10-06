use std::mem::swap;
use std::num::TryFromIntError;
use num_bigfloat::BigFloat;
use num_traits::ToPrimitive;
use crate::formula::clmm::full_math::MulDiv;
use crate::formula::clmm::raydium_sqrt_price_math::Q64;
use crate::formula::clmm::raydium_tick_math::get_sqrt_price_at_tick;
use crate::formula::clmm::u256_math::{U128, U256};

/*
    For Raydium Concentrated Liquidity pool
 */


#[derive(Debug)]
pub struct SwapState {
    pub amount_specified_remaining: u64,
    pub amount_calculated: u64,
    pub sqrt_price_x64: u128,
    pub tick: i32,
    // pub fee_growth_global_x64: u128,
    pub fee_amount: u64,
    pub protocol_fee: u64,
    pub fund_fee: u64,
    pub liquidity: u128,
}

#[derive(Default)]
pub struct StepComputations {
    pub sqrt_price_start_x64: u128,
    pub tick_next: i32,
    pub initialized: bool,
    pub sqrt_price_next_x64: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}

pub fn calculate_amount_in_range(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    zero_for_one: bool,
    is_base_input: bool,
) -> Result<u64, TryFromIntError> {
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

pub fn get_liquidity_from_amounts(
    sqrt_ratio_x64: u128,
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_0: u64,
    amount_1: u64,
) -> u128 {
    // sqrt_ratio_a_x64 should hold the smaller value
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    if sqrt_ratio_x64 <= sqrt_ratio_a_x64 {
        // If P ≤ P_lower, only token_0 liquidity is active
        get_liquidity_from_amount_0(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_0)
    } else if sqrt_ratio_x64 < sqrt_ratio_b_x64 {
        // If P_lower < P < P_upper, active liquidity is the minimum of the liquidity provided
        // by token_0 and token_1
        u128::min(
            get_liquidity_from_amount_0(sqrt_ratio_x64, sqrt_ratio_b_x64, amount_0),
            get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_x64, amount_1),
        )
    } else {
        // If P ≥ P_upper, only token_1 liquidity is active
        get_liquidity_from_amount_1(sqrt_ratio_a_x64, sqrt_ratio_b_x64, amount_1)
    }
}

pub fn get_liquidity_from_amount_0(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_0: u64,
) -> u128 {
    // sqrt_ratio_a_x64 should hold the smaller value
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        std::mem::swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };
    let intermediate = U128::from(sqrt_ratio_a_x64)
        .mul_div_floor(
            U128::from(sqrt_ratio_b_x64),
            U128::from(Q64),
        )
        .unwrap();

    U128::from(amount_0)
        .mul_div_floor(
            intermediate,
            U128::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
        )
        .unwrap()
        .as_u128()
}

/// Computes the amount of liquidity received for a given amount of token_1 and price range
/// Calculates ΔL = Δy / (√P_upper - √P_lower)
pub fn get_liquidity_from_amount_1(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    amount_1: u64,
) -> u128 {
    // sqrt_ratio_a_x64 should hold the smaller value
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64);
    };

    U128::from(amount_1)
        .mul_div_floor(
            U128::from(Q64),
            U128::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64),
        )
        .unwrap()
        .as_u128()
}

pub fn get_delta_amount_0_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u64, TryFromIntError> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64)
    }

    let q64 = BigFloat::from(Q64);
    let num1 = BigFloat::from(liquidity).mul(&q64);
    let num2 = BigFloat::from(sqrt_ratio_b_x64 - sqrt_ratio_a_x64);

    if round_up {
        let res = u64::try_from(
            num1.mul(&num2).div(&BigFloat::from(sqrt_ratio_b_x64)).ceil().div(&BigFloat::from(sqrt_ratio_a_x64)).ceil().to_u128().unwrap()
        );
        res
    }
    else {
        let res = u64::try_from(
            num1.mul(&num2).div(&BigFloat::from(sqrt_ratio_b_x64)).floor().div(&BigFloat::from(sqrt_ratio_a_x64)).to_u128().unwrap()
        );
        res
    }
}

pub fn get_delta_amount_1_unsigned(
    mut sqrt_ratio_a_x64: u128,
    mut sqrt_ratio_b_x64: u128,
    liquidity: u128,
    round_up: bool,
) -> Result<u64, TryFromIntError> {
    if sqrt_ratio_a_x64 > sqrt_ratio_b_x64 {
        swap(&mut sqrt_ratio_a_x64, &mut sqrt_ratio_b_x64)
    }

    let q64 = BigFloat::from(Q64);
    if round_up {
        u64::try_from(
            BigFloat::from(liquidity).mul(&BigFloat::from(sqrt_ratio_b_x64).sub(&BigFloat::from(sqrt_ratio_a_x64)))
                .div(&q64)
                .ceil()
                .to_u128().unwrap()
        )
    }
    else {
        u64::try_from(
            BigFloat::from(liquidity).mul(&BigFloat::from(sqrt_ratio_b_x64).sub(&BigFloat::from(sqrt_ratio_a_x64)))
                .div(&q64)
                .floor()
                .to_u128().unwrap()
        )
    }
}

pub fn get_delta_amount_0_signed(
    sqrt_ratio_a_x64: u128,
    sqrt_ratio_b_x64: u128,
    liquidity: i128,
) -> Result<u64, TryFromIntError> {
    if liquidity < 0 {
        get_delta_amount_0_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(-liquidity).unwrap(),
            false,
        )
    } else {
        get_delta_amount_0_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(liquidity).unwrap(),
            true,
        )
    }
}

/// Helper function to get signed delta amount_1 for given liquidity and price range
pub fn get_delta_amount_1_signed(
    sqrt_ratio_a_x64: u128,
    sqrt_ratio_b_x64: u128,
    liquidity: i128,
) -> Result<u64, TryFromIntError> {
    if liquidity < 0 {
        get_delta_amount_1_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(-liquidity).unwrap(),
            false,
        )
    } else {
        get_delta_amount_1_unsigned(
            sqrt_ratio_a_x64,
            sqrt_ratio_b_x64,
            u128::try_from(liquidity).unwrap(),
            true,
        )
    }
}

pub fn get_delta_amounts_signed(
    tick_current: i32,
    sqrt_price_x64_current: u128,
    tick_lower: i32,
    tick_upper: i32,
    liquidity_delta: i128,
) -> Result<(u64, u64), &'static str> {
    let mut amount_0 = 0;
    let mut amount_1 = 0;
    if tick_current < tick_lower {
        amount_0 = get_delta_amount_0_signed(
            get_sqrt_price_at_tick(tick_lower)?,
            get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )
            .unwrap();
    } else if tick_current < tick_upper {
        amount_0 = get_delta_amount_0_signed(
            sqrt_price_x64_current,
            get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )
            .unwrap();
        amount_1 = get_delta_amount_1_signed(
            get_sqrt_price_at_tick(tick_lower)?,
            sqrt_price_x64_current,
            liquidity_delta,
        )
            .unwrap();
    } else {
        amount_1 = get_delta_amount_1_signed(
            get_sqrt_price_at_tick(tick_lower)?,
            get_sqrt_price_at_tick(tick_upper)?,
            liquidity_delta,
        )
            .unwrap();
    }
    Ok((amount_0, amount_1))
}

pub fn add_delta(x: u128, y: i128) -> Result<u128, &'static str> {
    let z: u128;
    if y < 0 {
        z = x - u128::try_from(-y).unwrap();
        if x <= z {
            return Err("liquidity sub value error")
        }
    } else {
        z = x + u128::try_from(y).unwrap();
        if z < x {
            return Err("liquidity add value error")
        }
    }

    Ok(z)
}

#[cfg(test)]
pub mod unit_test {

}
