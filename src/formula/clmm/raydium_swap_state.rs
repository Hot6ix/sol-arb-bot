use std::mem::swap;
use num_bigfloat::BigFloat;
use crate::formula::clmm::constant::{BIT_PRECISION};
use crate::formula::clmm::sqrt_price_math::{Q64, tick_to_sqrt_price_x64};

/*
    For Raydium Concentrated Liquidity pool
 */


#[derive(Debug)]
pub struct SwapState {
    pub amount_specified_remaining: u128,
    pub amount_calculated: u128,
    pub sqrt_price_x64: u128,
    pub tick: i32,
    // pub fee_growth_global_x64: u128,
    pub fee_amount: u128,
    pub protocol_fee: u128,
    pub fund_fee: u128,
    pub liquidity: u128,
}

#[derive(Default)]
pub struct StepComputations {
    pub sqrt_price_start_x64: u128,
    pub tick_next: i32,
    pub initialized: bool,
    pub sqrt_price_next_x64: u128,
    pub amount_in: u128,
    pub amount_out: u128,
    pub fee_amount: u128,
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

pub fn get_tick_at_sqrt_price(sqrt_price_x64: u128) -> Result<i32, &'static str> {
    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    let log2p_integer_x32 = (msb as i128 - 64) << 32;

    let mut bit: i128 = 0x8000_0000_0000_0000i128;
    let mut precision = 0;
    let mut log2p_fraction_x64 = 0;

    // Log2 iterative approximation for the fractional part
    // Go through each 2^(j) bit where j < 64 in a Q64.64 number
    // Append current bit value to fraction result if r^2 Q2.126 is more than 2
    let mut r = if msb >= 64 {
        sqrt_price_x64 >> (msb - 63)
    } else {
        sqrt_price_x64 << (63 - msb)
    };

    while bit > 0 && precision < BIT_PRECISION {
        r *= r;
        let is_r_more_than_two = r >> 127 as u32;
        r >>= 63 + is_r_more_than_two;
        log2p_fraction_x64 += bit * is_r_more_than_two as i128;
        bit >>= 1;
        precision += 1;
    }
    let log2p_fraction_x32 = log2p_fraction_x64 >> 32;
    let log2p_x32 = log2p_integer_x32 + log2p_fraction_x32;

    let log_sqrt_10001_x64 = log2p_x32 * 59543866431248i128;

    // tick - 0.01
    let tick_low = ((log_sqrt_10001_x64 - 184467440737095516i128) >> 64) as i32;

    // tick + (2^-14 / log2(√1.001)) + 0.01
    let tick_high = ((log_sqrt_10001_x64 + 15793534762490258745i128) >> 64) as i32;

    Ok(if tick_low == tick_high {
        tick_low
        //todo
    // } else if get_sqrt_price_at_tick(tick_high).unwrap() <= sqrt_price_x64 {
    } else if tick_to_sqrt_price_x64(&tick_high).unwrap() <= sqrt_price_x64 {
        tick_high
    } else {
        tick_low
    })
}

// todo
// pub fn get_sqrt_price_at_tick(tick: i32) -> Result<u128, &'static str> {
//     let abs_tick = tick.abs() as u32;
//
//     // i = 0
//     let mut ratio = if abs_tick & 0x1 != 0 {
//         U128([0xfffcb933bd6fb800, 0])
//     } else {
//         // 2^64
//         U128([0, 1])
//     };
//     // i = 1
//     if abs_tick & 0x2 != 0 {
//         ratio = (ratio * U128([0xfff97272373d4000, 0])) >> NUM_64
//     };
//     // i = 2
//     if abs_tick & 0x4 != 0 {
//         ratio = (ratio * U128([0xfff2e50f5f657000, 0])) >> NUM_64
//     };
//     // i = 3
//     if abs_tick & 0x8 != 0 {
//         ratio = (ratio * U128([0xffe5caca7e10f000, 0])) >> NUM_64
//     };
//     // i = 4
//     if abs_tick & 0x10 != 0 {
//         ratio = (ratio * U128([0xffcb9843d60f7000, 0])) >> NUM_64
//     };
//     // i = 5
//     if abs_tick & 0x20 != 0 {
//         ratio = (ratio * U128([0xff973b41fa98e800, 0])) >> NUM_64
//     };
//     // i = 6
//     if abs_tick & 0x40 != 0 {
//         ratio = (ratio * U128([0xff2ea16466c9b000, 0])) >> NUM_64
//     };
//     // i = 7
//     if abs_tick & 0x80 != 0 {
//         ratio = (ratio * U128([0xfe5dee046a9a3800, 0])) >> NUM_64
//     };
//     // i = 8
//     if abs_tick & 0x100 != 0 {
//         ratio = (ratio * U128([0xfcbe86c7900bb000, 0])) >> NUM_64
//     };
//     // i = 9
//     if abs_tick & 0x200 != 0 {
//         ratio = (ratio * U128([0xf987a7253ac65800, 0])) >> NUM_64
//     };
//     // i = 10
//     if abs_tick & 0x400 != 0 {
//         ratio = (ratio * U128([0xf3392b0822bb6000, 0])) >> NUM_64
//     };
//     // i = 11
//     if abs_tick & 0x800 != 0 {
//         ratio = (ratio * U128([0xe7159475a2caf000, 0])) >> NUM_64
//     };
//     // i = 12
//     if abs_tick & 0x1000 != 0 {
//         ratio = (ratio * U128([0xd097f3bdfd2f2000, 0])) >> NUM_64
//     };
//     // i = 13
//     if abs_tick & 0x2000 != 0 {
//         ratio = (ratio * U128([0xa9f746462d9f8000, 0])) >> NUM_64
//     };
//     // i = 14
//     if abs_tick & 0x4000 != 0 {
//         ratio = (ratio * U128([0x70d869a156f31c00, 0])) >> NUM_64
//     };
//     // i = 15
//     if abs_tick & 0x8000 != 0 {
//         ratio = (ratio * U128([0x31be135f97ed3200, 0])) >> NUM_64
//     };
//     // i = 16
//     if abs_tick & 0x10000 != 0 {
//         ratio = (ratio * U128([0x9aa508b5b85a500, 0])) >> NUM_64
//     };
//     // i = 17
//     if abs_tick & 0x20000 != 0 {
//         ratio = (ratio * U128([0x5d6af8dedc582c, 0])) >> NUM_64
//     };
//     // i = 18
//     if abs_tick & 0x40000 != 0 {
//         ratio = (ratio * U128([0x2216e584f5fa, 0])) >> NUM_64
//     }
//
//     // Divide to obtain 1.0001^(2^(i - 1)) * 2^32 in numerator
//     if tick > 0 {
//         ratio = U128::MAX / ratio;
//     }
//
//     Ok(ratio.as_u128())
// }

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