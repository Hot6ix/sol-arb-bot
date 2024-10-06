use std::ops::{Mul, Neg};

use num_bigfloat::BigFloat;
use num_traits::Zero;

use crate::formula::clmm::constant::FEE_RATE_DENOMINATOR_VALUE;
use crate::formula::clmm::raydium_sqrt_price_math::{get_next_sqrt_price_from_input, get_next_sqrt_price_from_output};
use crate::formula::clmm::raydium_swap_state::{calculate_amount_in_range, get_delta_amount_0_unsigned, get_delta_amount_1_unsigned};

pub fn compute_swap_step(
    sqrt_price_current_x64: u128,
    sqrt_price_target_x64: u128,
    liquidity: u128,
    amount_remaining: u64,
    fee_rate: u32,
    is_base_input: bool,
    zero_for_one: bool,
) -> Result<SwapStep, &'static str> {
    let mut swap_step = SwapStep::default();

    let fee_rate_bf = BigFloat::from(fee_rate);
    let fee_rate_denominator_bf = BigFloat::from(FEE_RATE_DENOMINATOR_VALUE);

    if is_base_input {
        let amount_remaining_less_fee = BigFloat::from(amount_remaining).mul(fee_rate_denominator_bf.sub(&fee_rate_bf)).div(&fee_rate_denominator_bf).floor().to_u64().unwrap();

        let amount_in = calculate_amount_in_range(
            sqrt_price_current_x64,
            sqrt_price_target_x64,
            liquidity,
            zero_for_one,
            is_base_input
        ).unwrap();

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
        ).unwrap();

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
            ).unwrap();
        }
        if !(is_exceed && !is_base_input) {
            swap_step.amount_out = get_delta_amount_1_unsigned(
                swap_step.sqrt_price_next_x64,
                sqrt_price_current_x64,
                liquidity,
                false
            ).unwrap();
        }
    }
    else {
        if !(is_exceed && is_base_input) {
            swap_step.amount_in = get_delta_amount_1_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                true
            ).unwrap();
        }

        if !(is_exceed && !is_base_input) {
            swap_step.amount_out = get_delta_amount_0_unsigned(
                sqrt_price_current_x64,
                swap_step.sqrt_price_next_x64,
                liquidity,
                false
            ).unwrap();
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
            BigFloat::from(swap_step.amount_in).mul(&fee_rate_bf).div(&fee_rate_denominator_bf.sub(&fee_rate_bf)).ceil().to_u64().unwrap()
        };

    Ok(swap_step)
}

#[derive(Default, Debug, PartialEq)]
pub struct SwapStep {
    pub sqrt_price_next_x64: u128,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
}

#[cfg(test)]
mod unit_test {
    use crate::formula::clmm::concentrated_liquidity::{compute_swap_step, SwapStep};

    mod compute_swap_step_test {
        use std::convert::TryInto;
        use crate::formula::clmm::concentrated_liquidity::SwapStep;

        use super::*;

        const TWO_PCT: u16 = 20000;
        const Q64_RESOLUTION: u8 = 64;

        pub fn div_round_up(n: u128, d: u128) -> Result<u128, &'static str> {
            div_round_up_if(n, d, true)
        }

        pub fn div_round_up_if(n: u128, d: u128, round_up: bool) -> Result<u128, &'static str> {
            if d == 0 {
                return Err("ErrorCode::DivideByZero");
            }

            let q = n / d;

            Ok(if round_up && n % d > 0 { q + 1 } else { q })
        }

        #[test]
        fn swap_a_to_b_input() {
            // Example calculation
            let amount = 100u128;
            let init_liq = 1296;
            let init_price = 9;
            let price_limit = 4;

            // Calculate fee given fee percentage
            let fee_amount = div_round_up(amount * u128::from(TWO_PCT), 1_000_000)
                .ok()
                .unwrap();

            let init_b = init_liq * init_price;
            let init_a = init_liq / init_price;

            let amount_in = amount - fee_amount;
            let new_a = init_a + amount_in;

            // Calculate next price
            let next_price = div_round_up(init_liq << Q64_RESOLUTION, new_a)
                .ok()
                .unwrap();

            // b - new_b
            let amount_out = init_b - div_round_up(init_liq * init_liq, new_a).ok().unwrap();
            test_swap(
                100,
                TWO_PCT,
                init_liq,
                init_price << Q64_RESOLUTION,
                price_limit << Q64_RESOLUTION,
                true,
                true,
                SwapStep {
                    sqrt_price_next_x64: next_price,
                    amount_in: amount_in.try_into().unwrap(),
                    amount_out: amount_out.try_into().unwrap(),
                    fee_amount: fee_amount.try_into().unwrap(),
                },
            );
        }
        #[test]
        fn swap_a_to_b_input_zero() {
            test_swap(
                0,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    sqrt_price_next_x64: 9 << Q64_RESOLUTION,
                    amount_in: 0,
                    amount_out: 0,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_a_to_b_input_zero_liq() {
            test_swap(
                100,
                TWO_PCT,
                0,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 4 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_a_to_b_input_max() {
            test_swap(
                1000,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                true,
                true,
                SwapStep {
                    amount_in: 180,
                    amount_out: 6480,
                    sqrt_price_next_x64: 4 << Q64_RESOLUTION,
                    fee_amount: 4,
                },
            );
        }

        #[test]
        fn swap_a_to_b_input_max_1pct_fee() {
            test_swap(
                1000,
                TWO_PCT / 2,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                true,
                true,
                SwapStep {
                    amount_in: 180,
                    amount_out: 6480,
                    sqrt_price_next_x64: 4 << Q64_RESOLUTION,
                    fee_amount: 2,
                },
            );
        }

        #[test]
        fn swap_a_to_b_output() {
            test_swap(
                4723,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                false,
                true,
                SwapStep {
                    amount_in: 98,
                    amount_out: 4723,
                    // sqrt_price_next_x64: 98795409425631171116,
                    sqrt_price_next_x64: 98795409425631171115,
                    fee_amount: 2,
                },
            );
        }

        #[test]
        fn swap_a_to_b_output_max() {
            test_swap(
                10000,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                false,
                true,
                SwapStep {
                    amount_in: 180,
                    amount_out: 6480,
                    sqrt_price_next_x64: 4 << Q64_RESOLUTION,
                    fee_amount: 4,
                },
            );
        }

        #[test]
        fn swap_a_to_b_output_zero() {
            test_swap(
                0,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                false,
                true,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 9 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_a_to_b_output_zero_liq() {
            test_swap(
                100,
                TWO_PCT,
                0,
                9 << Q64_RESOLUTION,
                4 << Q64_RESOLUTION,
                false,
                true,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 4 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_b_to_a_input() {
            test_swap(
                2000,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    amount_in: 1960,
                    amount_out: 20,
                    sqrt_price_next_x64: 193918550355107200012,
                    fee_amount: 40,
                },
            );
        }

        #[test]
        fn swap_b_to_a_input_max() {
            test_swap(
                20000,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    amount_in: 9072,
                    amount_out: 63,
                    sqrt_price_next_x64: 16 << Q64_RESOLUTION,
                    fee_amount: 186,
                },
            );
        }

        #[test]
        fn swap_b_to_a_input_zero() {
            test_swap(
                0,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 9 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_b_to_a_input_zero_liq() {
            test_swap(
                100,
                TWO_PCT,
                0,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                true,
                false,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 16 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_b_to_a_output() {
            test_swap(
                20,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                false,
                false,
                SwapStep {
                    amount_in: 1882,
                    amount_out: 20,
                    sqrt_price_next_x64: 192798228383286926568,
                    fee_amount: 39,
                },
            );
        }

        #[test]
        fn swap_b_to_a_output_max() {
            test_swap(
                80,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                false,
                false,
                SwapStep {
                    amount_in: 9072,
                    amount_out: 63,
                    sqrt_price_next_x64: 16 << Q64_RESOLUTION,
                    fee_amount: 186,
                },
            );
        }

        #[test]
        fn swap_b_to_a_output_zero() {
            test_swap(
                0,
                TWO_PCT,
                1296,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                false,
                false,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 9 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }

        #[test]
        fn swap_b_to_a_output_zero_liq() {
            test_swap(
                100,
                TWO_PCT,
                0,
                9 << Q64_RESOLUTION,
                16 << Q64_RESOLUTION,
                false,
                false,
                SwapStep {
                    amount_in: 0,
                    amount_out: 0,
                    sqrt_price_next_x64: 16 << Q64_RESOLUTION,
                    fee_amount: 0,
                },
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn test_swap(
        amount_remaining: u64,
        fee_rate: u16,
        liquidity: u128,
        sqrt_price_current: u128,
        sqrt_price_target_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
        expected: SwapStep,
    ) {
        let swap_computation = compute_swap_step(
            sqrt_price_current,
            sqrt_price_target_limit,
            liquidity,
            amount_remaining,
            fee_rate as u32,
            amount_specified_is_input,
            a_to_b,
        );

        if let Ok(ref s) = swap_computation {
            println!("{}", s.amount_in);
            println!("{}", s.amount_out);
        }
        assert_eq!(swap_computation.ok().unwrap(), expected);
    }
}