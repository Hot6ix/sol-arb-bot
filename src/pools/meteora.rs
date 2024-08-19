use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketOperation, MarketSerializer, PubkeyPair};

pub struct MeteoraMarket {
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

impl MarketSerializer for MeteoraMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 904];
        let (discriminator, parameters, v_parameters, bump_seed, bin_step_seed, pair_type, active_id, bin_step, status, require_base_factor_seed, base_factor_seed, padding1, token_x_mint, token_y_mint, reserve_x, reserve_y, protocol_fee, fee_owner, reward_infos, oracle, bit_array_bitmap, last_updated_at, whitelisted_wallet, pre_activation_swap_address, base_key, activation_slot, pre_activation_slot_duration, padding2, lock_durations_in_slot, creator, reserved) =
            array_refs![src, 8, 32, 32, 1, 2, 1, 4, 2, 1, 1, 2, 2, 32, 32, 32, 32, 16, 32, 288, 32, 128, 8, 32, 32, 32, 8, 8, 8, 8, 32, 24];

        MeteoraMarket {
            parameters: StaticParameters::unpack_data(*parameters),
            v_parameters: VariableParameters::unpack_data(*v_parameters),
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
            protocol_fee: ProtocolFee::unpack_data(*protocol_fee),
            fee_owner: Pubkey::new_from_array(*fee_owner),
            reward_infos: RewardInfo::unpack_data_set(*reward_infos),
            oracle: Pubkey::new_from_array(*oracle),
            bin_array_bitmap: [0u64; 16], // temp
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

impl MarketOperation for MeteoraMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_x_mint,
            pubkey_b: self.token_y_mint
        }
    }
}

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

impl StaticParameters {
    fn unpack_data(data: [u8; 32]) -> Self {
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

pub struct VariableParameters {
    pub volatility_accumulator: u32, // 4
    pub volatility_reference: u32, // 4
    pub index_reference: i32, // 4
    pub padding: [u8; 4], // 4
    pub last_update_timestamp: i64, // 8
    pub padding1: [u8; 8] // 8
}

impl VariableParameters {
    fn unpack_data(data: [u8; 32]) -> Self {
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

pub struct ProtocolFee {
    pub amount_x: u64, // 8
    pub amount_y: u64 // 8
}

impl ProtocolFee {
    pub fn unpack_data(data: [u8; 16]) -> ProtocolFee {
        let src = array_ref![data, 0, 16];
        let (amount_x, amount_y) =
            array_refs![src, 8, 8];

        ProtocolFee {
            amount_x: u64::from_le_bytes(*amount_x),
            amount_y: u64::from_le_bytes(*amount_y)
        }
    }
}

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

impl RewardInfo {
    pub fn unpack_data(data: Vec<u8>) -> RewardInfo {
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
    pub fn unpack_data_set(data: [u8; 288]) -> [RewardInfo; 2] {
        let src = array_ref![data, 0, 288];
        let (first, second) = data.split_at_checked(data.len() / 2).unwrap();

        [
            Self::unpack_data(Vec::from(first)),
            Self::unpack_data(Vec::from(second))
        ]
    }
}

pub fn parse_bin_array_bitmap(data: [u8; 128]) {
}