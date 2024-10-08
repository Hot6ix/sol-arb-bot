use std::ops::{Add, Mul, Sub};
use num_bigfloat::BigFloat;

pub struct DefaultConstantProduct {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub decimal_diff: i32,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64
}

impl ConstantProductBase for DefaultConstantProduct {
    fn calculate_fee(&self, amount_in: u64) -> BigFloat {
        BigFloat::from(self.swap_fee_numerator) / BigFloat::from(self.swap_fee_denominator) * BigFloat::from(amount_in)
    }

    fn calculate_liquidity(&self) -> u128 {
        u128::from(self.token_a_amount) * u128::from(self.token_b_amount)
    }

    fn swap(&self, amount_in: u64, zero_for_one: bool) -> BigFloat {
        let fee = BigFloat::from(self.calculate_fee(amount_in));

        let amount_in_with_fee = BigFloat::from(amount_in).sub(fee);
        let amount_out = (BigFloat::from(self.token_b_amount).mul(amount_in_with_fee)) / (BigFloat::from(self.token_a_amount).add(amount_in_with_fee));

        amount_out
    }
}

pub trait ConstantProductBase {
    fn calculate_fee(&self, amount_in: u64) -> BigFloat;
    fn calculate_liquidity(&self) -> u128;
    fn swap(&self, amount_in: u64, zero_for_one: bool) -> BigFloat;
}