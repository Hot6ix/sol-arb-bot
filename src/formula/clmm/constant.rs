use numext_fixed_uint::U128;

pub const MIN_TICK: i32 = -443636;
pub const MAX_TICK: i32 = -MIN_TICK;
pub const TICK_ARRAY_SIZE: i32 = 60;
pub const TICK_ARRAY_BITMAP_SIZE: i32 = 512;
pub const NUM_64: U128 = U128([64, 0]);

pub const BIT_PRECISION: u32 = 16;
pub const FEE_RATE_DENOMINATOR_VALUE: u32 = 1_000_000;
pub const TICK_ARRAY_SIZE_USIZE: usize = 60;
pub const MIN_SQRT_PRICE_X64: u128 = 4295048016;
pub const MAX_SQRT_PRICE_X64: u128 = 79226673521066979257578248091;

pub const POOL_SEED: &str = "pool";
pub const POOL_VAULT_SEED: &str = "pool_vault";
pub const POOL_REWARD_VAULT_SEED: &str = "pool_reward_vault";
pub const POOL_TICK_ARRAY_BITMAP_SEED: &str = "pool_tick_array_bitmap_extension";
pub const TICK_ARRAY_SEED: &str = "tick_array";