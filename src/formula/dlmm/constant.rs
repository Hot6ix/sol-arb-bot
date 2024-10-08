pub const NUM_REWARDS: usize = 2;
pub const SCALE_OFFSET: u8 = 64;
pub const BASIS_POINT_MAX: i32 = 10000;
pub const MAX_BIN_PER_ARRAY: usize = 70;
pub const FEE_PRECISION: u64 = 1_000_000_000;
pub const MAX_FEE_RATE: u64 = 100_000_000;
pub const EXTENSION_BIN_ARRAY_BITMAP_SIZE: usize = 12;
pub const BIN_ARRAY_BITMAP_SIZE: i32 = 512;
pub const MIN_BIN_ID: i32 = -443636;
pub const MAX_BIN_ID: i32 = 443636;

pub const BIN_ARRAY: &[u8] = b"bin_array";