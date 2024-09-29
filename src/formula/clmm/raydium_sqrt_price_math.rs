use std::ops::Add;
use std::str::FromStr;

use num_bigfloat::BigFloat;
use num_bigint::BigInt;

// 2^64 = 18446744073709551616
pub const Q64: u128 = 18_446_744_073_709_551_616u128;

pub fn tick_to_sqrt_price_x64(tick: &i32) -> Option<u128> {
    BigFloat::from(1.0001f64).pow(&BigFloat::from(*tick).div(&BigFloat::from(2))).mul(&BigFloat::from(Q64)).to_u128()
}

pub fn sqrt_price_x64_to_tick(sqrt_price_x64: &u128) -> Option<i32> {
    let tick = BigFloat::from(*sqrt_price_x64).div(&BigFloat::from(Q64)).log(&BigFloat::from(1.0001f64)).mul(&BigFloat::from(2u8)).floor().to_i64();
    match i32::try_from(tick?) {
        Ok(tick) => { Some(tick) }
        Err(_) => { None }
    }
}

pub fn price_to_sqrt_price_x64(price: &f64, decimal_diff: &i8) -> u128 {
    let decimals = BigFloat::from(10u32).pow(&BigFloat::from(*decimal_diff));
    BigFloat::from(*price).div(&decimals).sqrt().mul(&BigFloat::from(Q64)).floor().to_u128().unwrap()
}

pub fn sqrt_price_x64_to_price(sqrt_price_x64: &u128, decimal_diff: &i8) -> f64 {
    let decimals = BigFloat::from(10u32).pow(&BigFloat::from(*decimal_diff));
    BigFloat::from(*sqrt_price_x64).div(&BigFloat::from(Q64)).pow(&BigFloat::from(2u8)).mul(&decimals).to_f64()
}

pub fn get_next_sqrt_price_from_input(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_in: u128,
    zero_for_one: bool,
) -> u128 {
    if zero_for_one {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_in, true)
    }
    else {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_in, true)
    }
}

pub fn get_next_sqrt_price_from_output(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount_out: u128,
    zero_for_one: bool,
) -> u128 {
    if zero_for_one {
        get_next_sqrt_price_from_amount_1_rounding_down(sqrt_price_x64, liquidity, amount_out, false)
    }
    else {
        get_next_sqrt_price_from_amount_0_rounding_up(sqrt_price_x64, liquidity, amount_out, false)
    }
}

pub fn get_next_sqrt_price_from_amount_0_rounding_up(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u128,
    add: bool,
) -> u128 {
    if amount == 0 {
        return sqrt_price_x64;
    }

    let liquidity_shifted = BigInt::from(liquidity) << 64i32;
    let numerator = BigFloat::from_str(liquidity_shifted.to_string().as_str()).unwrap();
    let amount_bf = BigFloat::from(amount);
    let sqrt_price_x64_bf = BigFloat::from(sqrt_price_x64);

    if add {
        let product = amount_bf.mul(&sqrt_price_x64_bf);
        let denominator = numerator.add(product);
        if denominator >= numerator {
            return numerator.mul(&sqrt_price_x64_bf).div(&denominator).ceil().to_u128().unwrap()
        }

        let value = numerator.div(&sqrt_price_x64_bf).add(amount_bf);
        numerator.div(&value).add(BigFloat::from((numerator % value > BigFloat::default()) as u8)).to_u128().unwrap()
    }
    else {
        let product = amount_bf.mul(&sqrt_price_x64_bf);
        let denominator = numerator.sub(&product);
        numerator.mul(&sqrt_price_x64_bf).div(&denominator).ceil().to_u128().unwrap()
    }
}

pub fn get_next_sqrt_price_from_amount_1_rounding_down(
    sqrt_price_x64: u128,
    liquidity: u128,
    amount: u128,
    add: bool,
) -> u128 {
    let sqrt_price_x64_bf = BigFloat::from(sqrt_price_x64);
    let liquidity_bf = BigFloat::from(liquidity);

    if add {
        let amount_shifted = BigInt::from(amount) << 64i32;
        let quotient = BigFloat::from_str(amount_shifted.to_string().as_str()).unwrap().div(&liquidity_bf);
        sqrt_price_x64_bf.add(&quotient).to_u128().unwrap()
    } else {
        let amount_shifted = BigInt::from(amount) << 64i32; // U256::from(u128::from(amount) << fixed_point_64::RESOLUTION)
        let value = BigFloat::from_str(amount_shifted.to_string().as_str()).unwrap();
        let quotient = value.div(&liquidity_bf).add(BigFloat::from((value % liquidity_bf > BigFloat::default()) as u8));
        sqrt_price_x64_bf.sub(&quotient).to_u128().unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::formula::clmm::constant::{MAX_SQRT_PRICE_X64, MIN_SQRT_PRICE_X64};
    use crate::formula::clmm::raydium_sqrt_price_math::{price_to_sqrt_price_x64, sqrt_price_x64_to_price, sqrt_price_x64_to_tick, tick_to_sqrt_price_x64};

    #[test]
    fn test_tick_to_sqrt_price_x64() {
        let tick = -18867i32;
        let sqrt_price_x64 = tick_to_sqrt_price_x64(&tick);
        println!("{}", sqrt_price_x64.unwrap());
    }

    #[test]
    fn test_sqrt_price_x64_to_tick() {
        let sqrt_price_x64: Vec<u128> = vec![
            7182147241917313386,
            7174399016327223095,
            7174386368720733565,
            7174388168782077692,
            7174954712407921105,
            MAX_SQRT_PRICE_X64,
            MIN_SQRT_PRICE_X64
        ];

        let ticks: Vec<i32> = vec![
            -18867,
            -18889,
            -18889,
            -18889,
            -18887,
            443636,
            -443636
        ];

        sqrt_price_x64.iter().enumerate().for_each(|(index, p)| {
            let tick = sqrt_price_x64_to_tick(p);
            assert_eq!(tick.unwrap(), ticks[index])
        })
    }

    #[test]
    fn test_price_to_sqrt_price_x64() {
        let price = 151.37f64;
        let sqrt_price_x64 = price_to_sqrt_price_x64(&price, &-3i8);
        println!("{}", sqrt_price_x64)
    }

    #[test]
    fn test_sqrt_price_x64_to_price() {
        let sqrt_price_x64 = 7168359157675602364u128;
        let price = sqrt_price_x64_to_price(&sqrt_price_x64, &3i8);
        println!("{}", price)
    }
}