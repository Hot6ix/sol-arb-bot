use crate::formula::clmm::u256_math::U128;

/// The minimum tick
pub const MIN_TICK: i32 = -443636;
/// The minimum tick
pub const MAX_TICK: i32 = -MIN_TICK;

/// The minimum value that can be returned from #get_sqrt_price_at_tick. Equivalent to get_sqrt_price_at_tick(MIN_TICK)
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
/// The maximum value that can be returned from #get_sqrt_price_at_tick. Equivalent to get_sqrt_price_at_tick(MAX_TICK)
pub const MAX_SQRT_PRICE_X64: u128 = 79226673521066979257578248091;

// Number 64, encoded as a U128
const NUM_64: U128 = U128([64, 0]);

const BIT_PRECISION: u32 = 16;

/// Calculates 1.0001^(tick/2) as a U64.64 number representing
/// the square root of the ratio of the two assets (token_1/token_0)
///
/// Calculates result as a U64.64
/// Each magic factor is `2^64 / (1.0001^(2^(i - 1)))` for i in `[0, 18)`.
///
/// Throws if |tick| > MAX_TICK
///
/// # Arguments
/// * `tick` - Price tick
///
pub fn get_sqrt_price_at_tick(tick: i32) -> Result<u128, &'static str> {
    let abs_tick = tick.abs() as u32;
    // require!(abs_tick <= MAX_TICK as u32, ErrorCode::TickUpperOverflow);

    // i = 0
    let mut ratio = if abs_tick & 0x1 != 0 {
        U128([0xfffcb933bd6fb800, 0])
    } else {
        // 2^64
        U128([0, 1])
    };
    // i = 1
    if abs_tick & 0x2 != 0 {
        ratio = (ratio * U128([0xfff97272373d4000, 0])) >> NUM_64
    };
    // i = 2
    if abs_tick & 0x4 != 0 {
        ratio = (ratio * U128([0xfff2e50f5f657000, 0])) >> NUM_64
    };
    // i = 3
    if abs_tick & 0x8 != 0 {
        ratio = (ratio * U128([0xffe5caca7e10f000, 0])) >> NUM_64
    };
    // i = 4
    if abs_tick & 0x10 != 0 {
        ratio = (ratio * U128([0xffcb9843d60f7000, 0])) >> NUM_64
    };
    // i = 5
    if abs_tick & 0x20 != 0 {
        ratio = (ratio * U128([0xff973b41fa98e800, 0])) >> NUM_64
    };
    // i = 6
    if abs_tick & 0x40 != 0 {
        ratio = (ratio * U128([0xff2ea16466c9b000, 0])) >> NUM_64
    };
    // i = 7
    if abs_tick & 0x80 != 0 {
        ratio = (ratio * U128([0xfe5dee046a9a3800, 0])) >> NUM_64
    };
    // i = 8
    if abs_tick & 0x100 != 0 {
        ratio = (ratio * U128([0xfcbe86c7900bb000, 0])) >> NUM_64
    };
    // i = 9
    if abs_tick & 0x200 != 0 {
        ratio = (ratio * U128([0xf987a7253ac65800, 0])) >> NUM_64
    };
    // i = 10
    if abs_tick & 0x400 != 0 {
        ratio = (ratio * U128([0xf3392b0822bb6000, 0])) >> NUM_64
    };
    // i = 11
    if abs_tick & 0x800 != 0 {
        ratio = (ratio * U128([0xe7159475a2caf000, 0])) >> NUM_64
    };
    // i = 12
    if abs_tick & 0x1000 != 0 {
        ratio = (ratio * U128([0xd097f3bdfd2f2000, 0])) >> NUM_64
    };
    // i = 13
    if abs_tick & 0x2000 != 0 {
        ratio = (ratio * U128([0xa9f746462d9f8000, 0])) >> NUM_64
    };
    // i = 14
    if abs_tick & 0x4000 != 0 {
        ratio = (ratio * U128([0x70d869a156f31c00, 0])) >> NUM_64
    };
    // i = 15
    if abs_tick & 0x8000 != 0 {
        ratio = (ratio * U128([0x31be135f97ed3200, 0])) >> NUM_64
    };
    // i = 16
    if abs_tick & 0x10000 != 0 {
        ratio = (ratio * U128([0x9aa508b5b85a500, 0])) >> NUM_64
    };
    // i = 17
    if abs_tick & 0x20000 != 0 {
        ratio = (ratio * U128([0x5d6af8dedc582c, 0])) >> NUM_64
    };
    // i = 18
    if abs_tick & 0x40000 != 0 {
        ratio = (ratio * U128([0x2216e584f5fa, 0])) >> NUM_64
    }

    // Divide to obtain 1.0001^(2^(i - 1)) * 2^32 in numerator
    if tick > 0 {
        ratio = U128::MAX / ratio;
    }

    Ok(ratio.as_u128())
}

/// Calculates the greatest tick value such that get_sqrt_price_at_tick(tick) <= ratio
/// Throws if sqrt_price_x64 < MIN_SQRT_RATIO or sqrt_price_x64 > MAX_SQRT_RATIO
///
/// Formula: `i = log base(√1.0001) (√P)`
pub fn get_tick_at_sqrt_price(sqrt_price_x64: u128) -> Result<i32, &'static str> {
    // second inequality must be < because the price can never reach the price at the max tick
    // require!(
    //     sqrt_price_x64 >= MIN_SQRT_PRICE_X64 && sqrt_price_x64 < MAX_SQRT_PRICE_X64,
    //     ErrorCode::SqrtPriceX64
    // );

    // Determine log_b(sqrt_ratio). First by calculating integer portion (msb)
    let msb: u32 = 128 - sqrt_price_x64.leading_zeros() - 1;
    let log2p_integer_x32 = (msb as i128 - 64) << 32;

    // get fractional value (r/2^msb), msb always > 128
    // We begin the iteration from bit 63 (0.5 in Q64.64)
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

    // 14 bit refinement gives an error margin of 2^-14 / log2 (√1.0001) = 0.8461 < 1
    // Since tick is a decimal, an error under 1 is acceptable

    // Change of base rule: multiply with 2^16 / log2 (√1.0001)
    let log_sqrt_10001_x64 = log2p_x32 * 59543866431248i128;

    // tick - 0.01
    let tick_low = ((log_sqrt_10001_x64 - 184467440737095516i128) >> 64) as i32;

    // tick + (2^-14 / log2(√1.001)) + 0.01
    let tick_high = ((log_sqrt_10001_x64 + 15793534762490258745i128) >> 64) as i32;

    Ok(if tick_low == tick_high {
        tick_low
    } else if get_sqrt_price_at_tick(tick_high).unwrap() <= sqrt_price_x64 {
        tick_high
    } else {
        tick_low
    })
}

#[cfg(test)]
mod tick_math_test {
    use super::*;
    mod get_sqrt_price_at_tick_test {
        use crate::formula::clmm::raydium_sqrt_price_math::Q64;
        use super::*;

        #[test]
        fn check_get_sqrt_price_at_tick_at_min_or_max_tick() {
            assert_eq!(
                get_sqrt_price_at_tick(MIN_TICK).unwrap(),
                MIN_SQRT_PRICE_X64
            );
            let min_sqrt_price = MIN_SQRT_PRICE_X64 as f64 / Q64 as f64;
            println!("min_sqrt_price: {}", min_sqrt_price);
            assert_eq!(
                get_sqrt_price_at_tick(MAX_TICK).unwrap(),
                MAX_SQRT_PRICE_X64
            );
            let max_sqrt_price = MAX_SQRT_PRICE_X64 as f64 / Q64 as f64;
            println!("max_sqrt_price: {}", max_sqrt_price);
        }
    }

    mod get_tick_at_sqrt_price_test {
        use super::*;

        #[test]
        fn test_sqrt_price_to_tick() {
            let sqrt_price_x64: Vec<u128> = vec![
                7182147241917313386,
                7174399016327223095,
                7174386368720733565,
                7174388168782077692,
                7174954712407921105,
                MAX_SQRT_PRICE_X64,
                MIN_SQRT_PRICE_X64,
                40038806028187328673,
                40040807918442726785,
                40042809908790148078,
                40044811999234571724,
                40046814189781028227,
                40048816480434496316,
                40050818871200006995,
                40052821362082540436,
                40054823953087122549,
                40056826644218744243,
                40058829435482431925,
                40060832326883166784,
                40062835318425977113,
                40064838410115852039,
                40066841601957820377,
                40068844893956864293,
                40070848286118011711,
                40072851778446255527,
                40074855370946624180,
                40076859063624100820,
                40078862856483714286,
                40080866749530458674,
                40082870742769363323,
                40084874836205412387,
                40086879029843635811,
                40088883323689028482,
                40090887717746620836,
                40092892212021398047,
                40094896806518392439,
                40096901501242597126,
                40098906296199044947,
                40100911191392722055,
                40102916186828660400,
                40104921282511856870,
                40106926478447343933,
                40108931774640108728,
                40110937171095189807,
                40112942667817573878,
                40114948264812299995,
                40116953962084356305,
                40118959759638776770,
                40120965657480560287,
                40122971655614741310,
                40124977754046309010,
                40126983952780299724,
                40128990251821710573,
                40130996651175578421,
                40133003150846893413,
                40135009750840691521,
                40137016451161973645
            ];

            sqrt_price_x64.iter().for_each(|x| {
                println!("{}", get_tick_at_sqrt_price(*x).unwrap())
            })
        }

        #[test]
        fn test_tick_to_sqrt_price() {
            let mut a: Vec<i32> = (-15500i32..-15550i32).collect::<Vec<i32>>();
            let mut b: Vec<i32> = (15500i32..15550i32).collect::<Vec<i32>>();
            let mut tick: Vec<i32> = vec![];
            tick.append(&mut a);
            tick.append(&mut b);

            tick.iter().for_each(|x| {
                println!("{}", get_sqrt_price_at_tick(*x).unwrap())
            })
        }

        #[test]
        fn check_get_tick_at_sqrt_price_at_min_or_max_sqrt_price() {
            assert_eq!(
                get_tick_at_sqrt_price(MIN_SQRT_PRICE_X64).unwrap(),
                MIN_TICK,
            );

            // we can't reach MAX_SQRT_PRICE_X64
            assert_eq!(
                get_tick_at_sqrt_price(MAX_SQRT_PRICE_X64 - 1).unwrap(),
                MAX_TICK - 1,
            );
        }
    }

    #[test]
    fn test_sqrt_price_from_tick_index_at_max() {
        let r = get_tick_at_sqrt_price(MAX_SQRT_PRICE_X64).unwrap();
        assert_eq!(&r, &MAX_TICK);
    }

    #[test]
    fn test_sqrt_price_from_tick_index_at_max_sub_one() {
        let sqrt_price_x64 = MAX_SQRT_PRICE_X64 - 1;
        let r = get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
        assert_eq!(&r, &(MAX_TICK - 1));
    }

    #[test]
    fn tick_round_down() {
        // tick is negative
        let sqrt_price_x64 = get_sqrt_price_at_tick(-28861).unwrap();
        let mut tick = get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
        assert_eq!(tick, -28861);
        tick = get_tick_at_sqrt_price(sqrt_price_x64 + 1).unwrap();
        assert_eq!(tick, -28861);
        tick = get_tick_at_sqrt_price(get_sqrt_price_at_tick(-28860).unwrap() - 1).unwrap();
        assert_eq!(tick, -28861);
        tick = get_tick_at_sqrt_price(sqrt_price_x64 - 1).unwrap();
        assert_eq!(tick, -28862);

        // tick is positive
        let sqrt_price_x64 = get_sqrt_price_at_tick(28861).unwrap();
        tick = get_tick_at_sqrt_price(sqrt_price_x64).unwrap();
        assert_eq!(tick, 28861);
        tick = get_tick_at_sqrt_price(sqrt_price_x64 + 1).unwrap();
        assert_eq!(tick, 28861);
        tick = get_tick_at_sqrt_price(get_sqrt_price_at_tick(28862).unwrap() - 1).unwrap();
        assert_eq!(tick, 28861);
        tick = get_tick_at_sqrt_price(sqrt_price_x64 - 1).unwrap();
        assert_eq!(tick, 28860);
    }
}
