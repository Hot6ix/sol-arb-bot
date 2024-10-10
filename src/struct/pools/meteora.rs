use std::any::Any;
use std::ops::{BitXor, Shl, Shr};
use arrayref::{array_ref, array_refs};
use ruint::aliases::U1024;
use solana_sdk::pubkey::Pubkey;
use crate::formula::base::Formula;
use crate::formula::base::Formula::DynamicLiquidity;
use crate::formula::dlmm::bin::{BinArray};
use crate::formula::dlmm::bin_array_bitmap_extension::BinArrayBitmapExtension;
use crate::formula::dlmm::constant::{BASIS_POINT_MAX, BIN_ARRAY_BITMAP_SIZE, FEE_PRECISION, MAX_BIN_ID, MAX_FEE_RATE, MIN_BIN_ID};
use crate::formula::dlmm::safe_math::SafeMath;
use crate::formula::dlmm::u128x128_math::Rounding;
use crate::formula::dlmm::utils_math::{one, safe_mul_div_cast};
use crate::formula::meteora_dlmm::{PairStatus, PairType};
use crate::r#struct::account::{AccountDataSerializer, DeserializedAccount, DeserializedConfigAccount, DeserializedTokenAccount};
use crate::r#struct::market::{PoolOperation};
use crate::utils::{PubkeyPair};

#[derive(Copy, Clone, Debug, Default)]
pub struct MeteoraClmmMarket {
    pub parameters: StaticParameters, // 32
    pub v_parameters: VariableParameters, // 32
    pub bump_seed: [u8; 1], // 1
    pub bin_step_seed: [u8; 2], // 2
    pub pair_type: u8, // 1
    pub active_id: i32, // 4
    pub bin_step: u16, // 2
    pub status: u8, // 1
    pub require_base_factor_seed: u8, // 1
    pub base_factor_seed: [u8; 2], // 2
    pub _padding1: [u8; 2], // 2
    pub token_x_mint: Pubkey, // 32
    pub token_y_mint: Pubkey, // 32
    pub reserve_x: Pubkey, // 32
    pub reserve_y: Pubkey, // 32
    pub protocol_fee: ProtocolFee, // 16
    pub fee_owner: Pubkey, // 32
    pub reward_infos: [RewardInfo; 2], // 288
    pub oracle: Pubkey, // 32
    pub bin_array_bitmap: [u64; 16], // 128
    pub last_updated_at: i64, // 8
    pub whitelisted_wallet: Pubkey, // 32
    pub pre_activation_swap_address: Pubkey, // 32
    pub base_key: Pubkey, // 32
    pub activation_slot: u64, // 8
    pub pre_activation_slot_duration: u64, // 8
    pub _padding2: [u8; 8], // 8
    pub lock_durations_in_slot: u64, // 8
    pub creator: Pubkey, // 32
    pub _reserved: [u8; 24], // 24
}

impl AccountDataSerializer for MeteoraClmmMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 904];
        let (discriminator, parameters, v_parameters, bump_seed, bin_step_seed, pair_type, active_id, bin_step, status, require_base_factor_seed, base_factor_seed, padding1, token_x_mint, token_y_mint, reserve_x, reserve_y, protocol_fee, fee_owner, reward_infos, oracle, bin_array_bitmap, last_updated_at, whitelisted_wallet, pre_activation_swap_address, base_key, activation_slot, pre_activation_slot_duration, padding2, lock_durations_in_slot, creator, reserved) =
            array_refs![src, 8, 32, 32, 1, 2, 1, 4, 2, 1, 1, 2, 2, 32, 32, 32, 32, 16, 32, 288, 32, 128, 8, 32, 32, 32, 8, 8, 8, 8, 32, 24];

        MeteoraClmmMarket {
            parameters: StaticParameters::unpack_data(&Vec::from(parameters)),
            v_parameters: VariableParameters::unpack_data(&Vec::from(v_parameters)),
            bump_seed: *bump_seed,
            bin_step_seed: *bin_step_seed,
            pair_type: u8::from_le_bytes(*pair_type),
            active_id: i32::from_le_bytes(*active_id),
            bin_step: u16::from_le_bytes(*bin_step),
            status: u8::from_le_bytes(*status),
            require_base_factor_seed: u8::from_le_bytes(*require_base_factor_seed),
            base_factor_seed: *base_factor_seed,
            _padding1: *padding1,
            token_x_mint: Pubkey::new_from_array(*token_x_mint),
            token_y_mint: Pubkey::new_from_array(*token_y_mint),
            reserve_x: Pubkey::new_from_array(*reserve_x),
            reserve_y: Pubkey::new_from_array(*reserve_y),
            protocol_fee: ProtocolFee::unpack_data(&Vec::from(protocol_fee)),
            fee_owner: Pubkey::new_from_array(*fee_owner),
            reward_infos: RewardInfo::unpack_data_set(*reward_infos),
            oracle: Pubkey::new_from_array(*oracle),
            bin_array_bitmap: Self::unpack_data_set(*bin_array_bitmap),
            last_updated_at: 0,
            whitelisted_wallet: Pubkey::new_from_array(*whitelisted_wallet),
            pre_activation_swap_address: Pubkey::new_from_array(*pre_activation_swap_address),
            base_key: Pubkey::new_from_array(*base_key),
            activation_slot: u64::from_le_bytes(*activation_slot),
            pre_activation_slot_duration: u64::from_le_bytes(*pre_activation_slot_duration),
            _padding2: *padding2,
            lock_durations_in_slot: u64::from_le_bytes(*lock_durations_in_slot),
            creator: Pubkey::new_from_array(*creator),
            _reserved: *reserved,
        }
    }
}

impl PoolOperation for MeteoraClmmMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_x_mint,
            pubkey_b: self.token_y_mint
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.reserve_x,
            pubkey_b: self.reserve_y
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        vec![
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.reserve_x),
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.reserve_y),
        ]
    }

    fn get_formula(&self) -> Formula {
        DynamicLiquidity
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl MeteoraClmmMarket {
    pub fn unpack_data_set(data: [u8; 128]) -> [u64; 16] {
        let mut vec: Vec<u64> = Vec::new();

        data.chunks_exact(8).for_each(|array| {
            vec.push(u64::from_le_bytes(array.try_into().unwrap()))
        });

        vec.try_into().unwrap()
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct StaticParameters {
    pub base_factor: u16, // 2
    pub filter_period: u16, // 2
    pub decay_period: u16, // 2
    pub reduction_factor: u16, // 2
    pub variable_fee_control: u32, // 4
    pub max_volatility_accumulator: u32, // 4
    pub min_bin_id: i32, // 4
    pub max_bin_id: i32, // 4
    pub protocol_share: u16, // 2
    pub padding: [u8; 6] // 6
}

impl AccountDataSerializer for StaticParameters {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 32];
        let (base_factor, filter_period, decay_period, reduction_factor, variable_fee_control, max_volatility_accumulator, min_bin_id, max_bin_id, protocol_share, padding) =
            array_refs![src, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6];

        StaticParameters {
            base_factor: u16::from_le_bytes(*base_factor),
            filter_period: u16::from_le_bytes(*filter_period),
            decay_period: u16::from_le_bytes(*decay_period),
            reduction_factor: u16::from_le_bytes(*reduction_factor),
            variable_fee_control: u32::from_le_bytes(*variable_fee_control),
            max_volatility_accumulator: u32::from_le_bytes(*max_volatility_accumulator),
            min_bin_id: i32::from_le_bytes(*min_bin_id),
            max_bin_id: i32::from_le_bytes(*max_bin_id),
            protocol_share: u16::from_le_bytes(*protocol_share),
            padding: *padding,
        }
    }
}

impl StaticParameters {
    pub fn get_filter_period(&self) -> u16 {
        self.filter_period
    }

    pub fn get_decay_period(&self) -> u16 {
        self.decay_period
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VariableParameters {
    pub volatility_accumulator: u32, // 4
    pub volatility_reference: u32, // 4
    pub index_reference: i32, // 4
    pub padding: [u8; 4], // 4
    pub last_update_timestamp: i64, // 8
    pub padding1: [u8; 8] // 8
}

impl AccountDataSerializer for VariableParameters {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 32];
        let (volatility_accumulator, volatility_reference, index_reference, padding, last_update_timestamp, padding1) =
            array_refs![src, 4, 4, 4, 4, 8, 8];

        VariableParameters {
            volatility_accumulator: u32::from_le_bytes(*volatility_accumulator),
            volatility_reference: u32::from_le_bytes(*volatility_reference),
            index_reference: i32::from_le_bytes(*index_reference),
            padding: *padding,
            last_update_timestamp: i64::from_le_bytes(*last_update_timestamp),
            padding1: *padding1,
        }
    }
}

impl VariableParameters {
    /// volatility_accumulator = min(volatility_reference + num_of_bin_crossed, max_volatility_accumulator)
    pub fn update_volatility_accumulator(
        &mut self,
        active_id: i32,
        static_params: &StaticParameters,
    ) -> Result<(), &'static str> {
        // Upscale to prevent overflow caused by swapping from left most bin to right most bin.
        let delta_id = i64::from(self.index_reference)
            .safe_sub(active_id.into())?
            .unsigned_abs();

        let volatility_accumulator = u64::from(self.volatility_reference)
            .safe_add(delta_id.safe_mul(BASIS_POINT_MAX as u64)?)?;

        self.volatility_accumulator = std::cmp::min(
            volatility_accumulator,
            static_params.max_volatility_accumulator.into(),
        )
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?;

        Ok(())
    }

    /// Update id, and volatility reference
    pub fn update_references(
        &mut self,
        active_id: i32,
        current_timestamp: i64,
        static_params: &StaticParameters,
    ) -> Result<(), &'static str> {
        let elapsed = current_timestamp.safe_sub(self.last_update_timestamp)?;

        // Not high frequency trade
        if elapsed >= static_params.get_filter_period() as i64 {
            // Update active id of last transaction
            self.index_reference = active_id;
            // filter period < t < decay_period. Decay time window.
            if elapsed < static_params.get_decay_period() as i64 {
                let volatility_reference = self
                    .volatility_accumulator
                    .safe_mul(static_params.reduction_factor as u32)?
                    .safe_div(BASIS_POINT_MAX as u32)?;

                self.volatility_reference = volatility_reference;
            }
            // Out of decay time window
            else {
                self.volatility_reference = 0;
            }
        }

        // self.last_update_timestamp = current_timestamp;

        Ok(())
    }

    pub fn update_volatility_parameter(
        &mut self,
        active_id: i32,
        current_timestamp: i64,
        static_params: &StaticParameters,
    ) -> Result<(), &'static str> {
        self.update_references(active_id, current_timestamp, static_params)?;
        self.update_volatility_accumulator(active_id, static_params)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ProtocolFee {
    pub amount_x: u64, // 8
    pub amount_y: u64 // 8
}

impl AccountDataSerializer for ProtocolFee {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 16];
        let (amount_x, amount_y) =
            array_refs![src, 8, 8];

        ProtocolFee {
            amount_x: u64::from_le_bytes(*amount_x),
            amount_y: u64::from_le_bytes(*amount_y)
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RewardInfo {
    pub mint: Pubkey, // 32
    pub vault: Pubkey, // 32
    pub funder: Pubkey, // 32
    pub reward_duration: u64, // 8
    pub reward_duration_end: u64, // 8
    pub reward_rate: u128, // 16
    pub last_update_time: i64, // 8
    pub cumulative_seconds_with_empty_liquidity_reward: u64 // 8
}

impl AccountDataSerializer for RewardInfo {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 144];
        let (mint, vault, funder, reward_duration, reward_duration_end, reward_rate, last_update_time, cumulative_seconds_with_empty_liquidity_reward) =
            array_refs![src, 32, 32, 32, 8, 8, 16, 8, 8];

        RewardInfo {
            mint: Pubkey::new_from_array(*mint),
            vault: Pubkey::new_from_array(*vault),
            funder: Pubkey::new_from_array(*funder),
            reward_duration: u64::from_le_bytes(*reward_duration),
            reward_duration_end: u64::from_le_bytes(*reward_duration_end),
            reward_rate: u128::from_le_bytes(*reward_rate),
            last_update_time: i64::from_le_bytes(*last_update_time),
            cumulative_seconds_with_empty_liquidity_reward: u64::from_le_bytes(*cumulative_seconds_with_empty_liquidity_reward),
        }
    }
}

impl RewardInfo {
    pub fn unpack_data_set(data: [u8; 288]) -> [RewardInfo; 2] {
        let src = array_ref![data, 0, 288];
        let (first, second) = data.split_at_checked(src.len() / 2).unwrap();

        [
            Self::unpack_data(&Vec::from(first)),
            Self::unpack_data(&Vec::from(second))
        ]
    }

    pub fn initialized(&self) -> bool {
        self.mint.ne(&Pubkey::default())
    }

    pub fn calculate_reward_per_token_stored_since_last_update(
        &self,
        current_time: u64,
        liquidity_supply: u64,
    ) -> Result<u128, &'static str> {
        let time_period = self.get_seconds_elapsed_since_last_update(current_time)?;

        safe_mul_div_cast(
            time_period.into(),
            self.reward_rate,
            liquidity_supply.into(),
            Rounding::Down,
        )
    }

    pub fn update_last_update_time(&mut self, current_time: u64) {
        self.last_update_time = std::cmp::min(current_time as i64, self.reward_duration_end as i64);
    }

    pub fn get_seconds_elapsed_since_last_update(&self, current_time: u64) -> Result<u64, &'static str> {
        let last_time_reward_applicable = std::cmp::min(current_time, self.reward_duration_end);
        let time_period = last_time_reward_applicable.safe_sub(self.last_update_time as u64)?;

        Ok(time_period)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MeteoraDlmmMarket {
    pub parameters: StaticParameters,
    pub v_parameters: VariableParameters,
    pub bump_seed: [u8; 1],
    pub bin_step_seed: [u8; 2],
    pub pair_type: u8,
    pub active_id: i32,
    pub bin_step: u16,
    pub status: u8,
    pub require_base_factor_seed: u8,
    pub base_factor_seed: [u8; 2],
    pub activation_type: u8,
    pub _padding_0: u8,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub protocol_fee: ProtocolFee,
    pub _padding_1: [u8; 32],
    pub reward_infos: [RewardInfo; 2], // TODO: Bug in anchor IDL parser when using InitSpace macro. Temp hardcode it. https://github.com/coral-xyz/anchor/issues/2556
    pub oracle: Pubkey,
    pub bin_array_bitmap: [u64; 16], // store default bin id from -512 to 511 (bin id from -35840 to 35840, price from 2.7e-16 to 3.6e15)
    pub last_updated_at: i64,
    pub whitelisted_wallet: Pubkey,
    pub pre_activation_swap_address: Pubkey,
    pub base_key: Pubkey,
    pub activation_point: u64,
    pub pre_activation_duration: u64,
    pub _padding_2: [u8; 8],
    pub lock_duration: u64,
    pub creator: Pubkey,
    pub _reserved: [u8; 24],
}

impl AccountDataSerializer for MeteoraDlmmMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 904];
        let (discriminator, parameters, v_parameters, bump_seed, bin_step_seed, pair_type, active_id, bin_step, status, require_base_factor_seed, base_factor_seed, activation_type, _padding_0, token_x_mint, token_y_mint, reserve_x, reserve_y, _protocol_fee, _padding_1, reward_infos, oracle, bin_array_bitmap, last_updated_at, whitelisted_wallet, pre_activation_swap_address, base_key, activation_point, pre_activation_duration, _padding_2, lock_duration, creator, _reserved) =
            array_refs![src, 8, 32, 32, 1, 2, 1, 4, 2, 1, 1, 2, 1, 1, 32, 32, 32, 32, 16, 32, 288, 32, 128, 8, 32, 32, 32, 8, 8, 8, 8, 32, 24];

        MeteoraDlmmMarket {
            parameters: StaticParameters::unpack_data(&Vec::from(parameters)),
            v_parameters: VariableParameters::unpack_data(&Vec::from(v_parameters)),
            bump_seed: *bump_seed,
            bin_step_seed: *bin_step_seed,
            pair_type: u8::from_le_bytes(*pair_type),
            active_id: i32::from_le_bytes(*active_id),
            bin_step: u16::from_le_bytes(*bin_step),
            status: u8::from_le_bytes(*status),
            require_base_factor_seed: u8::from_le_bytes(*require_base_factor_seed),
            base_factor_seed: *base_factor_seed,
            activation_type: u8::from_le_bytes(*activation_type),
            _padding_0: u8::from_le_bytes(*_padding_0),
            token_x_mint: Pubkey::new_from_array(*token_x_mint),
            token_y_mint: Pubkey::new_from_array(*token_y_mint),
            reserve_x: Pubkey::new_from_array(*reserve_x),
            reserve_y: Pubkey::new_from_array(*reserve_y),
            protocol_fee: ProtocolFee::unpack_data(&_protocol_fee.to_vec()),
            _padding_1: *_padding_1,
            reward_infos: RewardInfo::unpack_data_set(*reward_infos),
            oracle: Pubkey::new_from_array(*oracle),
            bin_array_bitmap: Self::unpack_data_set(*bin_array_bitmap),
            last_updated_at: i64::from_le_bytes(*last_updated_at),
            whitelisted_wallet: Pubkey::new_from_array(*whitelisted_wallet),
            pre_activation_swap_address: Pubkey::new_from_array(*pre_activation_swap_address),
            base_key: Pubkey::new_from_array(*base_key),
            activation_point: u64::from_le_bytes(*activation_point),
            pre_activation_duration: u64::from_le_bytes(*pre_activation_duration),
            _padding_2: *_padding_2,
            lock_duration: u64::from_le_bytes(*lock_duration),
            creator: Pubkey::new_from_array(*creator),
            _reserved: *_reserved,
        }
    }
}

impl PoolOperation for MeteoraDlmmMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_x_mint,
            pubkey_b: self.token_y_mint,
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.reserve_x,
            pubkey_b: self.reserve_y,
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        todo!()
    }

    fn get_formula(&self) -> Formula {
        DynamicLiquidity
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl MeteoraDlmmMarket {
    pub fn unpack_data_set(data: [u8; 128]) -> [u64; 16] {
        let mut vec: Vec<u64> = Vec::new();

        data.chunks_exact(8).for_each(|array| {
            vec.push(u64::from_le_bytes(array.try_into().unwrap()))
        });

        vec.try_into().unwrap()
    }

    pub fn status(&self) -> Result<PairStatus, &'static str> {
        let pair_status: PairStatus = self
            .status
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?;

        Ok(pair_status)
    }

    pub fn pair_type(&self) -> Result<PairType, &'static str> {
        let pair_type: PairType = self
            .pair_type
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?;

        Ok(pair_type)
    }

    pub fn swap_for_y(&self, out_token_mint: Pubkey) -> bool {
        out_token_mint.eq(&self.token_y_mint)
    }

    /// Plus / Minus 1 to the active bin based on the swap direction
    pub fn advance_active_bin(&mut self, swap_for_y: bool) -> Result<(), &'static str> {
        let next_active_bin_id = if swap_for_y {
            self.active_id.safe_sub(1)?
        } else {
            self.active_id.safe_add(1)?
        };

        // todo
        // require!(
        //     next_active_bin_id >= MIN_BIN_ID && next_active_bin_id <= MAX_BIN_ID,
        //     LBError::PairInsufficientLiquidity
        // );
        assert!(next_active_bin_id >= MIN_BIN_ID && next_active_bin_id <= MAX_BIN_ID);

        self.active_id = next_active_bin_id;

        Ok(())
    }

    pub fn compute_fee(&self, amount: u64) -> Result<u64, &'static str> {
        let total_fee_rate = self.get_total_fee()?;
        let denominator = u128::from(FEE_PRECISION).safe_sub(total_fee_rate)?;

        // Ceil division
        let fee = u128::from(amount)
            .safe_mul(total_fee_rate)?
            .safe_add(denominator)?
            .safe_sub(1)?;

        let scaled_down_fee = fee.safe_div(denominator)?;

        Ok(scaled_down_fee
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?)
    }

    pub fn compute_protocol_fee(&self, fee_amount: u64) -> Result<u64, &'static str> {
        let protocol_fee = u128::from(fee_amount)
            .safe_mul(self.parameters.protocol_share.into())?
            .safe_div(BASIS_POINT_MAX as u128)?;

        Ok(protocol_fee
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?)
    }

    pub fn compute_fee_from_amount(&self, amount_with_fees: u64) -> Result<u64, &'static str> {
        // total_fee_rate 1e9 unit
        let total_fee_rate = self.get_total_fee()?;
        // Ceil division
        let fee_amount = u128::from(amount_with_fees)
            .safe_mul(total_fee_rate)?
            .safe_add((FEE_PRECISION - 1).into())?;
        let scaled_down_fee = fee_amount.safe_div(FEE_PRECISION.into())?;

        Ok(scaled_down_fee
            .try_into()
            .map_err(|_| "LBError::TypeCastFailed")?)
    }

    pub fn get_total_fee(&self) -> Result<u128, &'static str> {
        let total_fee_rate = self.get_base_fee()?.safe_add(self.get_variable_fee()?)?;
        let total_fee_rate_cap = std::cmp::min(total_fee_rate, MAX_FEE_RATE.into());
        Ok(total_fee_rate_cap)
    }

    pub fn get_base_fee(&self) -> Result<u128, &'static str> {
        Ok(u128::from(self.parameters.base_factor)
            .safe_mul(self.bin_step.into())?
            // Make it to be the same as FEE_PRECISION defined for ceil_div later on.
            .safe_mul(10u128)?)
    }

    pub fn get_variable_fee(&self) -> Result<u128, &'static str> {
        self.compute_variable_fee(self.v_parameters.volatility_accumulator)
    }

    pub fn compute_variable_fee(&self, volatility_accumulator: u32) -> Result<u128, &'static str> {
        if self.parameters.variable_fee_control > 0 {
            let volatility_accumulator: u128 = volatility_accumulator.into();
            let bin_step: u128 = self.bin_step.into();
            let variable_fee_control: u128 = self.parameters.variable_fee_control.into();

            let square_vfa_bin = volatility_accumulator
                .safe_mul(bin_step)?
                .checked_pow(2)
                .ok_or("LBError::MathOverflow")?;

            // Variable fee control, volatility accumulator, bin step are in basis point unit (10_000)
            // This is 1e20. Which > 1e9. Scale down it to 1e9 unit and ceiling the remaining.
            let v_fee = variable_fee_control.safe_mul(square_vfa_bin)?;

            let scaled_v_fee = v_fee.safe_add(99_999_999_999)?.safe_div(100_000_000_000)?;
            return Ok(scaled_v_fee);
        }

        Ok(0)
    }

    /// Accumulate protocol fee
    pub fn accumulate_protocol_fees(&mut self, fee_amount_x: u64, fee_amount_y: u64) -> Result<(), &'static str> {
        self.protocol_fee.amount_x = self.protocol_fee.amount_x.safe_add(fee_amount_x)?;
        self.protocol_fee.amount_y = self.protocol_fee.amount_y.safe_add(fee_amount_y)?;

        Ok(())
    }

    /// Update volatility reference and accumulator
    pub fn update_volatility_parameters(&mut self, current_timestamp: i64) -> Result<(), &'static str> {
        self.v_parameters.update_volatility_parameter(
            self.active_id,
            current_timestamp,
            &self.parameters,
        )
    }

    pub fn update_references(&mut self, current_timestamp: i64) -> Result<(), &'static str> {
        self.v_parameters
            .update_references(self.active_id, current_timestamp, &self.parameters)
    }

    pub fn update_volatility_accumulator(&mut self) -> Result<(), &'static str> {
        self.v_parameters
            .update_volatility_accumulator(self.active_id, &self.parameters)
    }

    pub fn withdraw_protocol_fee(&mut self, amount_x: u64, amount_y: u64) -> Result<(), &'static str> {
        self.protocol_fee.amount_x = self.protocol_fee.amount_x.safe_sub(amount_x)?;
        self.protocol_fee.amount_y = self.protocol_fee.amount_y.safe_sub(amount_y)?;

        Ok(())
    }

    pub fn oracle_initialized(&self) -> bool {
        self.oracle != Pubkey::default()
    }

    // todo
    pub fn flip_bin_array_bit(
        &mut self,
        bin_array_bitmap_extension: &Option<BinArrayBitmapExtension>,
        bin_array_index: i32,
    ) -> Result<(), &'static str> {
        // if self.is_overflow_default_bin_array_bitmap(bin_array_index) {
        //     match bin_array_bitmap_extension {
        //         Some(mut bitmap_ext) => {
        //             bitmap_ext.flip_bin_array_bit(bin_array_index)?;
        //         }
        //         None => return Err("LBError::BitmapExtensionAccountIsNotProvided"),
        //     }
        // } else {
        //     self.flip_bin_array_bit_internal(bin_array_index)?;
        // }

        Ok(())
    }

    pub fn is_overflow_default_bin_array_bitmap(&self, bin_array_index: i32) -> bool {
        let (min_bitmap_id, max_bitmap_id) = MeteoraDlmmMarket::bitmap_range();
        bin_array_index > max_bitmap_id || bin_array_index < min_bitmap_id
    }

    pub fn bitmap_range() -> (i32, i32) {
        (-BIN_ARRAY_BITMAP_SIZE, BIN_ARRAY_BITMAP_SIZE - 1)
    }

    fn get_bin_array_offset(bin_array_index: i32) -> usize {
        (bin_array_index + BIN_ARRAY_BITMAP_SIZE) as usize
    }

    fn flip_bin_array_bit_internal(&mut self, bin_array_index: i32) -> Result<(), &'static str> {
        let bin_array_offset = Self::get_bin_array_offset(bin_array_index);
        let bin_array_bitmap = U1024::from_limbs(self.bin_array_bitmap);
        let mask = one::<1024, 16>() << bin_array_offset;
        self.bin_array_bitmap = bin_array_bitmap.bitxor(mask).into_limbs();
        Ok(())
    }

    // return bin_array_index that it's liquidity is non-zero
    // if cannot find one, return false
    pub fn next_bin_array_index_with_liquidity_internal(
        &self,
        swap_for_y: bool,
        start_array_index: i32,
    ) -> Result<(i32, bool), &'static str> {
        let bin_array_bitmap = U1024::from_limbs(self.bin_array_bitmap);
        let array_offset: usize = Self::get_bin_array_offset(start_array_index);
        let (min_bitmap_id, max_bitmap_id) = MeteoraDlmmMarket::bitmap_range();
        if swap_for_y {
            let bin_map_range: usize = max_bitmap_id
                .safe_sub(min_bitmap_id)?
                .try_into()
                .map_err(|_| "LBError::TypeCastFailed")?;
            let offset_bit_map = bin_array_bitmap.shl(bin_map_range.safe_sub(array_offset)?);

            if offset_bit_map.eq(&U1024::ZERO) {
                return Ok((min_bitmap_id.safe_sub(1)?, false));
            } else {
                let next_bit = offset_bit_map.leading_zeros();
                return Ok((start_array_index.safe_sub(next_bit as i32)?, true));
            }
        } else {
            let offset_bit_map = bin_array_bitmap.shr(array_offset);
            if offset_bit_map.eq(&U1024::ZERO) {
                return Ok((max_bitmap_id.safe_add(1)?, false));
            } else {
                let next_bit = offset_bit_map.trailing_zeros();
                return Ok((
                    start_array_index.checked_add(next_bit as i32).unwrap(),
                    true,
                ));
            };
        }
    }

    // shift active until non-zero liquidity bin_array_index
    fn shift_active_bin(&mut self, swap_for_y: bool, bin_array_index: i32) -> Result<(), &'static str> {
        // update active id
        let (lower_bin_id, upper_bin_id) =
            BinArray::get_bin_array_lower_upper_bin_id(bin_array_index)?;

        if swap_for_y {
            self.active_id = upper_bin_id;
        } else {
            self.active_id = lower_bin_id;
        }
        Ok(())
    }

    fn next_bin_array_index_with_liquidity_from_extension(
        swap_for_y: bool,
        bin_array_index: i32,
        bin_array_bitmap_extension: &Option<BinArrayBitmapExtension>,
    ) -> Result<(i32, bool), &'static str> {
        match bin_array_bitmap_extension {
            Some(bitmap_ext) => {
                return Ok(bitmap_ext
                    .next_bin_array_index_with_liquidity(swap_for_y, bin_array_index)?);
            }
            None => return Err("LBError::BitmapExtensionAccountIsNotProvided"),
        }
    }

    pub fn next_bin_array_index_from_internal_to_extension(
        &mut self,
        swap_for_y: bool,
        current_array_index: i32,
        start_array_index: i32,
        bin_array_bitmap_extension: &Option<BinArrayBitmapExtension>,
    ) -> Result<(), &'static str> {
        let (bin_array_index, is_non_zero_liquidity_flag) =
            self.next_bin_array_index_with_liquidity_internal(swap_for_y, start_array_index)?;
        if is_non_zero_liquidity_flag {
            if current_array_index != bin_array_index {
                self.shift_active_bin(swap_for_y, bin_array_index)?;
            }
        } else {
            let (bin_array_index, _) = MeteoraDlmmMarket::next_bin_array_index_with_liquidity_from_extension(
                swap_for_y,
                bin_array_index,
                bin_array_bitmap_extension,
            )?;
            // no need to check for flag here, because if we cannot find the non-liquidity bin array id in the extension go from lb_pair state, then extension will return error
            if current_array_index != bin_array_index {
                self.shift_active_bin(swap_for_y, bin_array_index)?;
            }
        }
        Ok(())
    }

    pub fn next_bin_array_index_with_liquidity(
        &mut self,
        swap_for_y: bool,
        bin_array_bitmap_extension: &Option<BinArrayBitmapExtension>,
    ) -> Result<(), &'static str> {
        let start_array_index = BinArray::bin_id_to_bin_array_index(self.active_id)?;

        if self.is_overflow_default_bin_array_bitmap(start_array_index) {
            let (bin_array_index, is_non_zero_liquidity_flag) =
                MeteoraDlmmMarket::next_bin_array_index_with_liquidity_from_extension(
                    swap_for_y,
                    start_array_index,
                    bin_array_bitmap_extension,
                )?;
            if is_non_zero_liquidity_flag {
                if start_array_index != bin_array_index {
                    self.shift_active_bin(swap_for_y, bin_array_index)?;
                }
            } else {
                self.next_bin_array_index_from_internal_to_extension(
                    swap_for_y,
                    start_array_index,
                    bin_array_index,
                    bin_array_bitmap_extension,
                )?;
            }
        } else {
            self.next_bin_array_index_from_internal_to_extension(
                swap_for_y,
                start_array_index,
                start_array_index,
                bin_array_bitmap_extension,
            )?;
        }
        Ok(())
    }
}