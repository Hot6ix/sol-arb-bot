use std::ops::{Div, Mul};
use num_bigfloat::BigFloat;

pub fn calculate_fee(numerator: u64, denominator: u64, amount: &u64) -> BigFloat {
    BigFloat::from(numerator).div(BigFloat::from(denominator)).mul(BigFloat::from(*amount))
}