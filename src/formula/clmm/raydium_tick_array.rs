use std::ops::BitXor;
use arrayref::{array_ref, array_refs};
use num_traits::Zero;
use solana_sdk::pubkey::Pubkey;

use crate::account::account::AccountDataSerializer;
use crate::formula::clmm::constant::{MAX_TICK, MIN_TICK, POOL_TICK_ARRAY_BITMAP_SEED, TICK_ARRAY_BITMAP_SIZE, TICK_ARRAY_SIZE, TICK_ARRAY_SIZE_USIZE};
use crate::formula::clmm::raydium_swap_state::add_delta;
use crate::formula::clmm::u256_math::{U1024, U512};
use crate::r#struct::market::Market;
use crate::r#struct::pools::{RaydiumRewardInfo};

#[repr(packed)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TickState {
    pub tick: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_0_x64: u128,
    pub fee_growth_outside_1_x64: u128,
    pub reward_growths_outside_x64: [u128; 3],
    pub padding: [u32; 13],
}

impl Default for TickState {
    fn default() -> Self {
        TickState {
            tick: i32::default(),
            liquidity_net: i128::default(),
            liquidity_gross: u128::default(),
            fee_growth_outside_0_x64: u128::default(),
            fee_growth_outside_1_x64: u128::default(),
            reward_growths_outside_x64: [u128::default(); 3],
            padding: [u32::default(); 13],
        }
    }
}

impl AccountDataSerializer for TickState {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 168];
        let (tick, liquidity_net, liquidity_gross, fee_growth_outside_0_x64, fee_growth_outside_1_x64, reward_growths_outside_x64, padding) =
            array_refs![src, 4, 16, 16, 16, 16, 48, 52];

        TickState {
            tick: i32::from_le_bytes(*tick),
            liquidity_net: i128::from_le_bytes(*liquidity_net),
            liquidity_gross: u128::from_le_bytes(*liquidity_gross),
            fee_growth_outside_0_x64: u128::from_le_bytes(*fee_growth_outside_0_x64),
            fee_growth_outside_1_x64: u128::from_le_bytes(*fee_growth_outside_1_x64),
            reward_growths_outside_x64: bytemuck::cast(*reward_growths_outside_x64),
            padding: Self::unpack_padding(*padding),
        }
    }
}

impl TickState {
    pub fn check_is_out_of_boundary(tick: i32) -> bool {
        tick < MIN_TICK || tick > MAX_TICK
    }

    pub fn update(
        &mut self,
        tick_current: i32,
        liquidity_delta: i128,
        fee_growth_global_0_x64: u128,
        fee_growth_global_1_x64: u128,
        upper: bool,
        reward_infos: &[RaydiumRewardInfo; 3],
    ) -> Result<bool, &'static str> {
        let liquidity_gross_before = self.liquidity_gross;
        let liquidity_gross_after =
            add_delta(liquidity_gross_before, liquidity_delta)?;

        // Either liquidity_gross_after becomes 0 (uninitialized) XOR liquidity_gross_before
        // was zero (initialized)
        let flipped = (liquidity_gross_after == 0) != (liquidity_gross_before == 0);
        if liquidity_gross_before == 0 {
            // by convention, we assume that all growth before a tick was initialized happened _below_ the tick
            if self.tick <= tick_current {
                self.fee_growth_outside_0_x64 = fee_growth_global_0_x64;
                self.fee_growth_outside_1_x64 = fee_growth_global_1_x64;
                self.reward_growths_outside_x64 = RaydiumRewardInfo::get_reward_growths(reward_infos);
            }
        }

        self.liquidity_gross = liquidity_gross_after;

        // when the lower (upper) tick is crossed left to right (right to left),
        // liquidity must be added (removed)
        self.liquidity_net = if upper {
            self.liquidity_net.checked_sub(liquidity_delta)
        } else {
            self.liquidity_net.checked_add(liquidity_delta)
        }
            .unwrap();
        Ok(flipped)
    }

    pub fn cross(
        &mut self,
        fee_growth_global_0_x64: u128,
        fee_growth_global_1_x64: u128,
        reward_infos: &[RaydiumRewardInfo; 3],
    ) -> i128 {
        self.fee_growth_outside_0_x64 = fee_growth_global_0_x64
            .checked_sub(self.fee_growth_outside_0_x64)
            .unwrap();
        self.fee_growth_outside_1_x64 = fee_growth_global_1_x64
            .checked_sub(self.fee_growth_outside_1_x64)
            .unwrap();

        for i in 0..3 {
            if !reward_infos[i].initialized() {
                continue;
            }

            self.reward_growths_outside_x64[i] = reward_infos[i]
                .reward_growth_global_x64
                .checked_sub(self.reward_growths_outside_x64[i])
                .unwrap();
        }

        self.liquidity_net
    }

    pub fn is_initialized(&self) -> bool {
        self.liquidity_gross != 0
    }

    fn unpack_data_set(data: [u8; 10080]) -> [TickState; 60] {
        let mut vec: Vec<TickState> = Vec::new();

        data.chunks_exact(168).for_each(|array| {
            vec.push(TickState::unpack_data(&array.to_vec()))
        });

        vec.try_into().unwrap()
    }

    fn unpack_padding(data: [u8; 52]) -> [u32; 13] {
        let mut vec: Vec<u32> = Vec::new();

        data.chunks_exact(4).for_each(|array| {
            vec.push(u32::from_le_bytes(array.try_into().unwrap()))
        });

        vec.try_into().unwrap()
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct TickArrayStateAccount {
    pub pubkey: Pubkey,
    pub market: Market,
    pub tick_array_state: TickArrayState
}

#[derive(Clone, Debug, PartialEq)]
pub struct TickArrayState {
    pub pool_id: Pubkey, // 32
    pub start_tick_index: i32, // 4
    pub ticks: [TickState; 60], // 168 * 60
    pub initialized_tick_count: u8, // 1
    pub recent_epoch: u64, // 8
    pub padding: [u8; 107], // 107
}

impl Default for TickArrayState {
    fn default() -> Self {
        TickArrayState {
            pool_id: Default::default(),
            start_tick_index: i32::default(),
            ticks: [TickState::default(); 60],
            initialized_tick_count: u8::default(),
            recent_epoch: u64::default(),
            padding: [u8::default(); 107],
        }
    }
}

impl AccountDataSerializer for TickArrayState {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 10240];
        let (discriminator, pool_id, start_tick_index, ticks, initialized_tick_count, recent_epoch, padding) =
            array_refs![src, 8, 32, 4, 10080, 1, 8, 107];

        TickArrayState {
            pool_id: Pubkey::new_from_array(*pool_id),
            start_tick_index: i32::from_le_bytes(*start_tick_index),
            ticks: TickState::unpack_data_set(*ticks),
            initialized_tick_count: u8::from_le_bytes(*initialized_tick_count),
            recent_epoch: u64::from_le_bytes(*recent_epoch),
            padding: *padding,
        }
    }
}

impl TickArrayState {
    pub fn initialize(
        &mut self,
        start_index: i32,
        tick_spacing: u16,
        pool_key: Pubkey,
    ) -> Result<(), &'static str> {
        TickArrayState::check_is_valid_start_index(start_index, tick_spacing);
        self.start_tick_index = start_index;
        self.pool_id = pool_key;
        Ok(())
    }

    pub fn get_tick_state_mut(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
    ) -> Result<&mut TickState, &'static str> {
        let offset_in_array = self.get_tick_offset_in_array(tick_index, tick_spacing)?;
        Ok(&mut self.ticks[offset_in_array])
    }

    pub fn get_array_start_index(tick_index: i32, tick_spacing: u16) -> i32 {
        let ticks_in_array = TickArrayState::tick_count(tick_spacing);
        let mut start = tick_index / ticks_in_array;
        if tick_index < 0 && tick_index % ticks_in_array != 0 {
            start = start - 1
        }
        start * ticks_in_array
    }

    pub fn tick_count(tick_spacing: u16) -> i32 {
        TICK_ARRAY_SIZE * i32::from(tick_spacing)
    }
    pub fn first_initialized_tick(&mut self, zero_for_one: bool) -> Result<&mut TickState, &'static str> {
        if zero_for_one {
            let mut i = TICK_ARRAY_SIZE - 1;
            while i >= 0 {
                if self.ticks[i as usize].is_initialized() {
                    return Ok(self.ticks.get_mut(i as usize).unwrap());
                }
                i = i - 1;
            }
        } else {
            let mut i = 0;
            while i < TICK_ARRAY_SIZE_USIZE {
                if self.ticks[i].is_initialized() {
                    return Ok(self.ticks.get_mut(i).unwrap());
                }
                i = i + 1;
            }
        }
        Err("invalid tick array")
    }

    pub fn next_initialized_tick(
        &mut self,
        current_tick_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> Result<Option<&mut TickState>, &'static str> {
        let current_tick_array_start_index =
            TickArrayState::get_array_start_index(current_tick_index, tick_spacing);
        if current_tick_array_start_index != self.start_tick_index {
            return Ok(None);
        }
        let mut offset_in_array =
            (current_tick_index - self.start_tick_index) / i32::from(tick_spacing);

        if zero_for_one {
            while offset_in_array >= 0 {
                if self.ticks[offset_in_array as usize].is_initialized() {
                    return Ok(self.ticks.get_mut(offset_in_array as usize));
                }
                offset_in_array = offset_in_array - 1;
            }
        } else {
            offset_in_array = offset_in_array + 1;
            while offset_in_array < TICK_ARRAY_SIZE {
                if self.ticks[offset_in_array as usize].is_initialized() {
                    return Ok(self.ticks.get_mut(offset_in_array as usize));
                }
                offset_in_array = offset_in_array + 1;
            }
        }
        Ok(None)
    }

    pub fn update_tick_state(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        tick_state: TickState,
    ) -> Result<(), &'static str> {
        let offset_in_array = self.get_tick_offset_in_array(tick_index, tick_spacing)?;
        self.ticks[offset_in_array] = tick_state;
        // self.recent_epoch = get_recent_epoch()?;
        Ok(())
    }

    pub(crate) fn get_tick_offset_in_array(&self, tick_index: i32, tick_spacing: u16) -> Result<usize, &'static str> {
        let start_tick_index = TickArrayState::get_array_start_index(tick_index, tick_spacing);
        if start_tick_index != self.start_tick_index {
            return Err("invalid tick array")
        }

        let offset_in_array =
            ((tick_index - self.start_tick_index) / i32::from(tick_spacing)) as usize;
        Ok(offset_in_array)
    }

    pub fn key(program_id: &Pubkey, seeds: &[&[u8]]) -> Option<Pubkey> {
        if let Some((pubkey, _)) = Pubkey::try_find_program_address(
            seeds,
            program_id
        ) {
            Some(pubkey)
        }
        else {
            None
        }
    }

    pub fn check_is_valid_start_index(tick_index: i32, tick_spacing: u16) -> bool {
        if TickState::check_is_out_of_boundary(tick_index) {
            if tick_index > MAX_TICK {
                return false;
            }
            let min_start_index =
                TickArrayState::get_array_start_index(MIN_TICK, tick_spacing);
            return tick_index == min_start_index;
        }
        tick_index % TickArrayState::tick_count(tick_spacing) == 0
    }
}

pub type TickArrayBitmap = [u64; 8];

#[derive(Clone, Default, PartialEq)]
pub struct TickArrayBitmapExtensionAccount {
    pub pubkey: Pubkey,
    pub market: Market,
    pub tick_array_bitmap_extension: TickArrayBitmapExtension
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TickArrayBitmapExtension {
    pub pool_id: Pubkey,
    pub positive_tick_array_bitmap: [[u64; 8]; 14], // 896
    pub negative_tick_array_bitmap: [[u64; 8]; 14],
}

impl AccountDataSerializer for TickArrayBitmapExtension {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 1832];
        let (discriminator, pool_id, positive_tick_array_bitmap, negative_tick_array_bitmap) =
            array_refs![src, 8, 32, 896, 896];

        TickArrayBitmapExtension {
            pool_id: Pubkey::new_from_array(*pool_id),
            positive_tick_array_bitmap: TickArrayBitmapExtension::unpack_tick_array_bitmap(*positive_tick_array_bitmap),
            negative_tick_array_bitmap: TickArrayBitmapExtension::unpack_tick_array_bitmap(*negative_tick_array_bitmap),
        }
    }
}

impl TickArrayBitmapExtension {
    pub fn flip_tick_array_bit(
        &mut self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> Result<(), &'static str> {
        let (offset, tick_array_bitmap) = self.get_bitmap(tick_array_start_index, tick_spacing)?;
        let tick_array_offset_in_bitmap =
            Self::tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);
        let tick_array_bitmap = U512(tick_array_bitmap);
        let mask = U512::one() << tick_array_offset_in_bitmap;
        if tick_array_start_index < 0 {
            self.negative_tick_array_bitmap[offset] = tick_array_bitmap.bitxor(mask).0;
        } else {
            self.positive_tick_array_bitmap[offset] = tick_array_bitmap.bitxor(mask).0;
        }
        Ok(())
    }

    pub fn tick_array_offset_in_bitmap(tick_array_start_index: i32, tick_spacing: u16) -> i32 {
        let m = tick_array_start_index.abs() % max_tick_in_tickarray_bitmap(tick_spacing);
        let mut tick_array_offset_in_bitmap = m / TickArrayState::tick_count(tick_spacing);
        if tick_array_start_index < 0 && m != 0 {
            tick_array_offset_in_bitmap = TICK_ARRAY_BITMAP_SIZE - tick_array_offset_in_bitmap;
        }
        tick_array_offset_in_bitmap // 342
    }

    pub fn check_tick_array_is_initialized(
        &self,
        tick_array_start_index: i32,
        tick_spacing: u16,
    ) -> Result<(bool, i32), &'static str> {
        let (_, tick_array_bitmap) = self.get_bitmap(tick_array_start_index, tick_spacing)?;

        let tick_array_offset_in_bitmap =
            tick_array_offset_in_bitmap(tick_array_start_index, tick_spacing);

        if U512(tick_array_bitmap).bit(tick_array_offset_in_bitmap as usize) {
            return Ok((true, tick_array_start_index));
        }
        Ok((false, tick_array_start_index))
    }

    fn get_bitmap(&self, tick_index: i32, tick_spacing: u16) -> Result<(usize, TickArrayBitmap), &'static str> {
        let offset = get_bitmap_offset(tick_index, tick_spacing)?; // -20520, 1 -> 0
        if tick_index < 0 {
            Ok((offset, self.negative_tick_array_bitmap[offset]))
        } else {
            Ok((offset, self.positive_tick_array_bitmap[offset]))
        }
    }

    pub fn next_initialized_tick_array_from_one_bitmap(
        &self,
        last_tick_array_start_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> Result<(bool, i32), &'static str> {
        let multiplier = TickArrayState::tick_count(tick_spacing);
        let next_tick_array_start_index = if zero_for_one {
            last_tick_array_start_index - multiplier
        } else {
            last_tick_array_start_index + multiplier
        };
        let min_tick_array_start_index =
            TickArrayState::get_array_start_index(MIN_TICK, tick_spacing);
        let max_tick_array_start_index =
            TickArrayState::get_array_start_index(MAX_TICK, tick_spacing);

        if next_tick_array_start_index < min_tick_array_start_index
            || next_tick_array_start_index > max_tick_array_start_index
        {
            return Ok((false, next_tick_array_start_index));
        }

        let (_, tick_array_bitmap) = self.get_bitmap(next_tick_array_start_index, tick_spacing)?;

        Ok(Self::next_initialized_tick_array_in_bitmap(
            tick_array_bitmap,
            next_tick_array_start_index,
            tick_spacing,
            zero_for_one,
        ))
    }

    pub fn next_initialized_tick_array_in_bitmap(
        tick_array_bitmap: TickArrayBitmap,
        next_tick_array_start_index: i32,
        tick_spacing: u16,
        zero_for_one: bool,
    ) -> (bool, i32) {
        let (bitmap_min_tick_boundary, bitmap_max_tick_boundary) =
            get_bitmap_tick_boundary(next_tick_array_start_index, tick_spacing);

        let tick_array_offset_in_bitmap =
            tick_array_offset_in_bitmap(next_tick_array_start_index, tick_spacing);
        if zero_for_one {
            let offset_bit_map = U512(tick_array_bitmap)
                << (TICK_ARRAY_BITMAP_SIZE - 1 - tick_array_offset_in_bitmap);

            let next_bit = if offset_bit_map.is_zero() {
                None
            } else {
                Some(u16::try_from(offset_bit_map.leading_zeros()).unwrap())
            };

            if next_bit.is_some() {
                let next_array_start_index = next_tick_array_start_index
                    - i32::from(next_bit.unwrap()) * TickArrayState::tick_count(tick_spacing);
                return (true, next_array_start_index);
            } else {
                return (false, bitmap_min_tick_boundary);
            }
        } else {
            let offset_bit_map = U512(tick_array_bitmap) >> tick_array_offset_in_bitmap;

            let next_bit = if offset_bit_map.is_zero() {
                None
            } else {
                Some(u16::try_from(offset_bit_map.trailing_zeros()).unwrap())
            };
            if next_bit.is_some() {
                let next_array_start_index = next_tick_array_start_index
                    + i32::from(next_bit.unwrap()) * TickArrayState::tick_count(tick_spacing);
                return (true, next_array_start_index);
            } else {
                return (
                    false,
                    bitmap_max_tick_boundary - TickArrayState::tick_count(tick_spacing),
                );
            }
        }
    }

    pub fn key(program_id: &Pubkey, pool_id: &Pubkey) -> Option<Pubkey> {
        if let Some((pubkey, _)) = Pubkey::try_find_program_address(
            &[POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(), pool_id.as_ref()],
            program_id
        ) {
            Some(pubkey)
        }
        else {
            None
        }
    }

    fn unpack_tick_array_bitmap(data: [u8; 896]) -> [[u64; 8]; 14]{
        // negative_tick_array_bitmap: [[0u64;8]; 14],
        let mut vec: Vec<[u64; 8]> = Vec::new();
        data.chunks_exact(64).for_each(|tick| {
            let mut tick_vec: Vec<u64> = Vec::new();

            tick.chunks(8).for_each(|tick_value| {
                let value = u64::from_le_bytes(tick_value.try_into().unwrap());
                tick_vec.push(value)
            });

            vec.push(tick_vec.try_into().unwrap());
        });

        vec.try_into().unwrap()
    }
}

pub fn max_tick_in_tickarray_bitmap(tick_spacing: u16) -> i32 {
    i32::from(tick_spacing) * TICK_ARRAY_SIZE * TICK_ARRAY_BITMAP_SIZE
}

pub fn check_current_tick_array_is_initialized(
    bit_map: U1024,
    tick_current: i32,
    tick_spacing: u16,
) -> Result<(bool, i32), &'static str> {
    if TickState::check_is_out_of_boundary(tick_current) {
        return Err("invalid tick index");
    }
    let multiplier = i32::from(tick_spacing) * TICK_ARRAY_SIZE;
    let mut compressed = tick_current / multiplier + 512;
    if tick_current < 0 && tick_current % multiplier != 0 {
        compressed -= 1;
    }
    let bit_pos = compressed.abs();
    let mask = U1024::one() << bit_pos;
    let masked = bit_map & mask;
    let initialized = masked != U1024::default();
    if initialized {
        return Ok((true, (compressed - 512) * multiplier));
    }
    return Ok((false, (compressed - 512) * multiplier));
}

pub fn tick_array_offset_in_bitmap(tick_array_start_index: i32, tick_spacing: u16) -> i32 {
    let m = tick_array_start_index.abs() % max_tick_in_tick_array_bitmap(tick_spacing);
    let mut tick_array_offset_in_bitmap = m / TickArrayState::tick_count(tick_spacing);
    if tick_array_start_index < 0 && m != 0 {
        tick_array_offset_in_bitmap = TICK_ARRAY_BITMAP_SIZE - tick_array_offset_in_bitmap;
    }
    tick_array_offset_in_bitmap
}

pub fn get_bitmap_offset(tick_index: i32, tick_spacing: u16) -> Result<usize, &'static str> {
    check_extension_boundary(tick_index, tick_spacing)?;
    let ticks_in_one_bitmap = max_tick_in_tick_array_bitmap(tick_spacing); // 30720
    let mut offset = tick_index.abs() / ticks_in_one_bitmap - 1; // 20520 / 30720 - 1
    if tick_index < 0 && tick_index.abs() % ticks_in_one_bitmap == 0 {
        offset -= 1;
    }
    Ok(offset as usize)
}

pub fn max_tick_in_tick_array_bitmap(tick_spacing: u16) -> i32 {
    i32::from(tick_spacing) * TICK_ARRAY_SIZE * TICK_ARRAY_BITMAP_SIZE
}

pub fn check_extension_boundary(tick_index: i32, tick_spacing: u16) -> Result<(), &'static str> {
    let positive_tick_boundary = max_tick_in_tick_array_bitmap(tick_spacing);
    let negative_tick_boundary = -positive_tick_boundary;
    if tick_index >= negative_tick_boundary && tick_index < positive_tick_boundary {
        return Err("invalid tick array boundary");
    }
    Ok(())
}

pub fn next_initialized_tick_array_start_index(
    bit_map: U1024,
    last_tick_array_start_index: i32,
    tick_spacing: u16,
    zero_for_one: bool,
) -> (bool, i32) {
    let tick_boundary = max_tick_in_tick_array_bitmap(tick_spacing);
    let next_tick_array_start_index = if zero_for_one {
        last_tick_array_start_index - TickArrayState::tick_count(tick_spacing)
    } else {
        last_tick_array_start_index + TickArrayState::tick_count(tick_spacing)
    };

    if next_tick_array_start_index < -tick_boundary || next_tick_array_start_index >= tick_boundary
    {
        return (false, last_tick_array_start_index);
    }

    let multiplier = i32::from(tick_spacing) * TICK_ARRAY_SIZE;
    let mut compressed = next_tick_array_start_index / multiplier + 512;
    if next_tick_array_start_index < 0 && next_tick_array_start_index % multiplier != 0 {
        // round towards negative infinity
        compressed -= 1;
    }
    let bit_pos = compressed.abs();

    if zero_for_one {
        let offset_bit_map = bit_map << (1024 - bit_pos - 1);
        let next_bit = most_significant_bit(offset_bit_map);
        if next_bit.is_some() {
            let next_array_start_index =
                (bit_pos - i32::from(next_bit.unwrap()) - 512) * multiplier;
            (true, next_array_start_index)
        } else {
            (false, -tick_boundary)
        }
    } else {
        let offset_bit_map = bit_map >> bit_pos;
        let next_bit = least_significant_bit(offset_bit_map);
        if next_bit.is_some() {
            let next_array_start_index =
                (bit_pos + i32::from(next_bit.unwrap()) - 512) * multiplier;
            (true, next_array_start_index)
        } else {
            (
                false,
                tick_boundary - TickArrayState::tick_count(tick_spacing),
            )
        }
    }
}

pub fn most_significant_bit(x: U1024) -> Option<u16> {
    if x.is_zero() {
        None
    } else {
        Some(u16::try_from(x.leading_zeros()).unwrap())
    }
}

pub fn least_significant_bit(x: U1024) -> Option<u16> {
    if x.is_zero() {
        None
    } else {
        Some(u16::try_from(x.trailing_zeros()).unwrap())
    }
}

pub fn get_bitmap_tick_boundary(tick_array_start_index: i32, tick_spacing: u16) -> (i32, i32) {
    let ticks_in_one_bitmap: i32 = max_tick_in_tick_array_bitmap(tick_spacing);
    let mut m = tick_array_start_index.abs() / ticks_in_one_bitmap;
    if tick_array_start_index < 0 && tick_array_start_index.abs() % ticks_in_one_bitmap != 0 {
        m += 1;
    }
    let min_value: i32 = ticks_in_one_bitmap * m;
    if tick_array_start_index < 0 {
        (-min_value, -min_value + ticks_in_one_bitmap)
    } else {
        (min_value, min_value + ticks_in_one_bitmap)
    }
}

#[cfg(test)]
pub mod tick_array_bitmap_extension_test {
    use std::str::FromStr;
    use solana_sdk::account_info::AccountInfo;
    use super::*;

    pub struct BuildExtensionAccountInfo {
        pub key: Pubkey,
        pub lamports: u64,
        pub owner: Pubkey,
        pub data: Vec<u8>,
    }

    impl Default for BuildExtensionAccountInfo {
        #[inline]
        fn default() -> BuildExtensionAccountInfo {
            BuildExtensionAccountInfo {
                key: Pubkey::new_unique(),
                lamports: 0,
                owner: Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK").unwrap(),
                data: vec![0; 1832],
            }
        }
    }

    pub fn build_tick_array_bitmap_extension_info<'info>(
        param: &mut BuildExtensionAccountInfo,
    ) -> AccountInfo {
        let disc_bytes = [60, 150, 36, 219, 97, 128, 139, 153];
        for i in 0..8 {
            param.data[i] = disc_bytes[i];
        }
        AccountInfo::new(
            &param.key,
            false,
            true,
            &mut param.lamports,
            param.data.as_mut_slice(),
            &param.owner,
            false,
            0,
        )
    }
}