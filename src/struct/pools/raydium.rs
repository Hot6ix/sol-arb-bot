use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;
use crate::formula::base::Formula;
use crate::formula::base::Formula::{ConcentratedLiquidity, ConstantProduct};
use crate::account::account::{AccountDataSerializer};
use crate::r#struct::market::{ConfigAccount, PoolOperation};
use crate::utils::PubkeyPair;

#[derive(Copy, Clone, Debug)]
pub struct RaydiumClmmMarket {
    pub bump: [u8; 1], // 1
    pub amm_config: Pubkey, // 32
    pub owner: Pubkey, // 32
    pub token_mint_0: Pubkey, // 32
    pub token_mint_1: Pubkey, // 32
    pub token_vault_0: Pubkey, // 32
    pub token_vault_1: Pubkey, // 32
    pub observation_key: Pubkey, // 32
    pub mint_decimals_0: u8, // 1
    pub mint_decimals_1: u8, // 1
    pub tick_spacing: u16, // 2
    pub liquidity: u128, // 16
    pub sqrt_price_x64: u128, // 16
    pub tick_current: i32, // 4
    pub padding3: u16, // 2
    pub padding4: u16, // 2
    pub fee_growth_global_0_x64: u128, // 16
    pub fee_growth_global_1_x64: u128, // 16
    pub protocol_fees_token_0: u64, // 8
    pub protocol_fees_token_1: u64, // 8
    pub swap_in_amount_token_0: u128, // 16
    pub swap_out_amount_token_1: u128, // 16
    pub swap_in_amount_token_1: u128, // 16
    pub swap_out_amount_token_0: u128, // 16
    pub status: u8, // 1
    pub padding: [u8; 7], // 7
    pub reward_info: [RewardInfo; 3], // 169 * 3; 507
    pub tick_array_bitmap: [u64; 16], // 128
    pub total_fees_token_0: u64, // 8
    pub total_fees_claimed_token_0: u64, // 8
    pub total_fees_token_1: u64, // 8
    pub total_fees_claimed_token_1: u64, // 8
    pub fund_fees_token_0: u64, // 8
    pub fund_fees_token_1: u64, // 8
    pub open_time: u64, // 8
    pub recent_epoch: u64, // 8
    pub padding1: [u64; 24], // 192
    pub padding2: [u64; 32] // 256
}

impl AccountDataSerializer for RaydiumClmmMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 1544];
        let (discriminator, bump, amm_config, owner, token_mint_0, token_mint_1, token_vault_0, token_vault_1, observation_key, mint_decimals_0, mint_decimals_1, tick_spacing, liquidity, sqrt_price_x64, tick_current, padding3, padding4, fee_growth_global_0_x64, fee_growth_global_1_x64, protocol_fees_token_0, protocol_fees_token_1, swap_in_amount_token_0, swap_out_amount_token_1, swap_in_amount_token_1, swap_out_amount_token_0, status, padding, reward_infos, tick_array_bitmap, total_fees_token_0, total_fees_claimed_token_0, total_fees_token_1, total_fees_claimed_token_1, fund_fees_token_0, fund_fees_token_1, open_time, recent_epoch, padding1, padding2) =
            array_refs![src, 8, 1, 32, 32, 32, 32, 32, 32, 32, 1, 1, 2, 16, 16, 4, 2, 2, 16, 16, 8, 8, 16, 16, 16, 16, 1, 7, 507, 128, 8, 8, 8, 8, 8, 8, 8, 8, 192, 256];

        RaydiumClmmMarket {
            bump: *bump,
            amm_config: Pubkey::new_from_array(*amm_config),
            owner: Pubkey::new_from_array(*owner),
            token_mint_0: Pubkey::new_from_array(*token_mint_0),
            token_mint_1: Pubkey::new_from_array(*token_mint_1),
            token_vault_0: Pubkey::new_from_array(*token_vault_0),
            token_vault_1: Pubkey::new_from_array(*token_vault_1),
            observation_key: Pubkey::new_from_array(*observation_key),
            mint_decimals_0: u8::from_le_bytes(*mint_decimals_0),
            mint_decimals_1: u8::from_le_bytes(*mint_decimals_1),
            tick_spacing: u16::from_le_bytes(*tick_spacing),
            liquidity: u128::from_le_bytes(*liquidity),
            sqrt_price_x64: u128::from_le_bytes(*sqrt_price_x64),
            tick_current: i32::from_le_bytes(*tick_current),
            padding3: u16::from_le_bytes(*padding3),
            padding4: u16::from_le_bytes(*padding4),
            fee_growth_global_0_x64: u128::from_le_bytes(*fee_growth_global_0_x64),
            fee_growth_global_1_x64: u128::from_le_bytes(*fee_growth_global_1_x64),
            protocol_fees_token_0: u64::from_le_bytes(*protocol_fees_token_0),
            protocol_fees_token_1: u64::from_le_bytes(*protocol_fees_token_1),
            swap_in_amount_token_0: u128::from_le_bytes(*swap_in_amount_token_0),
            swap_out_amount_token_1: u128::from_le_bytes(*swap_out_amount_token_1),
            swap_in_amount_token_1: u128::from_le_bytes(*swap_in_amount_token_1),
            swap_out_amount_token_0: u128::from_le_bytes(*swap_out_amount_token_0),
            status: u8::from_le_bytes(*status),
            padding: *padding,
            reward_info: RewardInfo::unpack_data_set(*reward_infos),
            tick_array_bitmap: [0u64; 16], // temp
            total_fees_token_0: u64::from_le_bytes(*total_fees_token_0),
            total_fees_claimed_token_0: u64::from_le_bytes(*total_fees_claimed_token_0),
            total_fees_token_1: u64::from_le_bytes(*total_fees_token_1),
            total_fees_claimed_token_1: u64::from_le_bytes(*total_fees_claimed_token_1),
            fund_fees_token_0: u64::from_le_bytes(*fund_fees_token_0),
            fund_fees_token_1: u64::from_le_bytes(*fund_fees_token_1),
            open_time: u64::from_le_bytes(*open_time),
            recent_epoch: u64::from_le_bytes(*recent_epoch),
            padding1: [0u64; 24], // temp
            padding2: [0u64; 32], // temp
        }
    }
}

impl PoolOperation for RaydiumClmmMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_mint_0,
            pubkey_b: self.token_mint_1
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_vault_0,
            pubkey_b: self.token_vault_1
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(String, Pubkey)> {
        vec![
            (stringify!(self.amm_config).to_string(), self.amm_config),
            (stringify!(self.token_vault_0).to_string(), self.token_vault_0),
            (stringify!(self.token_vault_1).to_string(), self.token_vault_1),
            (stringify!(self.observation_key).to_string(), self.observation_key)
        ]
    }

    fn get_formula(&self) -> Formula {
        ConcentratedLiquidity
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RewardInfo { // 169
    pub reward_state: u8, // 1
    pub open_time: u64, // 8
    pub end_time: u64, // 8
    pub last_update_time: u64, // 8
    pub emissions_per_second_x64: u128, // 16
    pub reward_total_emissioned: u64, // 8
    pub reward_claimed: u64, // 8
    pub token_mint: Pubkey, // 32
    pub token_vault: Pubkey, // 32
    pub authority: Pubkey, // 32
    pub reward_growth_global_x64: u128 // 16
}

impl AccountDataSerializer for RewardInfo {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 169];
        let (reward_state, open_time, end_time, last_update_time, emissions_per_second_x64, reward_total_emissioned, reward_claimed, token_mint, token_vault, authority, reward_growth_global_x64) =
            array_refs![src, 1, 8, 8, 8, 16, 8, 8, 32, 32, 32, 16];

        RewardInfo {
            reward_state: u8::from_le_bytes(*reward_state),
            open_time: u64::from_le_bytes(*open_time),
            end_time: u64::from_le_bytes(*end_time),
            last_update_time: u64::from_le_bytes(*last_update_time),
            emissions_per_second_x64: u128::from_le_bytes(*emissions_per_second_x64),
            reward_total_emissioned: u64::from_le_bytes(*reward_total_emissioned),
            reward_claimed: u64::from_le_bytes(*reward_claimed),
            token_mint: Pubkey::new_from_array(*token_mint),
            token_vault: Pubkey::new_from_array(*token_vault),
            authority: Pubkey::new_from_array(*authority),
            reward_growth_global_x64: u128::from_le_bytes(*reward_growth_global_x64),
        }
    }
}

impl RewardInfo {
    fn unpack_data_set(data: [u8; 507]) -> [RewardInfo; 3] {
        let index = data.len() / 3;
        let (first, rest) = data.split_at_checked(index).unwrap();
        let (second, third) = rest.split_at_checked(index).unwrap();

        [
            Self::unpack_data(&Vec::from(first)),
            Self::unpack_data(&Vec::from(second)),
            Self::unpack_data(&Vec::from(third))
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AmmConfig { // 117
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub padding_u32: u32,
    pub fund_owner: Pubkey,
    pub padding: [u64; 3],
}

impl AccountDataSerializer for AmmConfig {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 117];
        let (discriminator, bump, index, owner, protocol_fee_rate, trade_fee_rate, tick_spacing, fund_fee_rate, padding_u32, fund_owner, padding) =
            array_refs![src, 8, 1, 2, 32, 4, 4, 2, 4, 4, 32, 24];

        AmmConfig {
            bump: u8::from_le_bytes(*bump),
            index: u16::from_le_bytes(*index),
            owner: Pubkey::new_from_array(*owner),
            protocol_fee_rate: u32::from_le_bytes(*protocol_fee_rate),
            trade_fee_rate: u32::from_le_bytes(*trade_fee_rate),
            tick_spacing: u16::from_le_bytes(*tick_spacing),
            fund_fee_rate: u32::from_le_bytes(*fund_fee_rate),
            padding_u32: u32::from_le_bytes(*padding_u32),
            fund_owner: Pubkey::new_from_array(*fund_owner),
            padding: [0u64; 3], // temp
        }
    }
}

pub enum RaydiumClmmAccount {
    AmmConfig(AmmConfig)
}

/////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone, Debug)]
pub struct RaydiumCpmmMarket {
    pub status: u64,
    pub nonce: u64,
    pub max_order: u64,
    pub depth: u64,
    pub base_decimal: u64,
    pub quote_decimal: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,
    pub amount_wave_ratio: u64,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub mint_price_multiplier: u64,
    pub max_price_multiplier: u64,
    pub system_decimal_value: u64,
    pub min_separate_numerator: u64,
    pub min_separate_denominator: u64,
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub pnl_numerator: u64,
    pub pnl_denominator: u64,
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
    pub base_need_take_pnl: u64,
    pub quote_need_take_pnl: u64,
    pub quote_total_pnl: u64,
    pub base_total_pnl: u64,
    pub pool_open_time: u64,
    pub punish_pc_amount: u64,
    pub punish_coin_amount: u64,
    pub orderbook_to_init_time: u64,
    pub swap_base_in_amount: u128,
    pub swap_quote_out_amount: u128,
    pub swap_base2_quote_fee: u64,
    pub swap_quote_in_amount: u128,
    pub swap_base_out_amount: u128,
    pub swap_quote2_base_fee: u64,
    pub base_vault: Pubkey, // 32
    pub quote_vault: Pubkey, // 32
    pub base_mint: Pubkey, // 32
    pub quote_mint: Pubkey, // 32
    pub lp_mint: Pubkey, // 32
    pub open_orders: Pubkey, // 32
    pub market_id: Pubkey, // 32
    pub market_program_id: Pubkey, // 32
    pub target_orders: Pubkey, // 32
    pub withdraw_queue: Pubkey, // 32
    pub lp_vault: Pubkey, // 32
    pub owner: Pubkey, // 32
    pub lp_reserve: u64,
    pub padding: [u64; 3]
}

impl AccountDataSerializer for RaydiumCpmmMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        // no discriminator for native solana program
        let src = array_ref![data, 0, 752];
        let (status, nonce, max_order, depth, base_decimal, quote_decimal, state, reset_flag, min_size, vol_max_cut_ratio, amount_wave_ratio, base_lot_size, quote_lot_size, mint_price_multiplier, max_price_multiplier, system_decimal_value, min_separate_numerator, min_separate_denominator, trade_fee_numerator, trade_fee_denominator, pnl_numerator, pnl_denominator, swap_fee_numerator, swap_fee_denominator, base_need_take_pnl, quote_need_take_pnl, quote_total_pnl, base_total_pnl, pool_open_time, punish_pc_amount, punish_coin_amount, orderbook_to_init_time, swap_base_in_amount, swap_quote_out_amount, swap_base2_quote_fee, swap_quote_in_amount, swap_base_out_amount, swap_quote2_base_fee, base_vault, quote_vault, base_mint, quote_mint, lp_mint, open_orders, market_id, market_program_id, target_orders, withdraw_queue, lp_vault, owner, lp_reserve, padding) =
            array_refs![src, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 8, 16, 16, 8, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 8, 24];

        RaydiumCpmmMarket {
            status: u64::from_le_bytes(*status),
            nonce: u64::from_le_bytes(*nonce),
            max_order: u64::from_le_bytes(*max_order),
            depth: u64::from_le_bytes(*depth),
            base_decimal: u64::from_le_bytes(*base_decimal),
            quote_decimal: u64::from_le_bytes(*quote_decimal),
            state: u64::from_le_bytes(*state),
            reset_flag: u64::from_le_bytes(*reset_flag),
            min_size: u64::from_le_bytes(*min_size),
            vol_max_cut_ratio: u64::from_le_bytes(*vol_max_cut_ratio),
            amount_wave_ratio: u64::from_le_bytes(*amount_wave_ratio),
            base_lot_size: u64::from_le_bytes(*base_lot_size),
            quote_lot_size: u64::from_le_bytes(*quote_lot_size),
            mint_price_multiplier: u64::from_le_bytes(*mint_price_multiplier),
            max_price_multiplier: u64::from_le_bytes(*max_price_multiplier),
            system_decimal_value: u64::from_le_bytes(*system_decimal_value),
            min_separate_numerator: u64::from_le_bytes(*min_separate_numerator),
            min_separate_denominator: u64::from_le_bytes(*min_separate_denominator),
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            pnl_numerator: u64::from_le_bytes(*pnl_numerator),
            pnl_denominator: u64::from_le_bytes(*pnl_denominator),
            swap_fee_numerator: u64::from_le_bytes(*swap_fee_numerator),
            swap_fee_denominator: u64::from_le_bytes(*swap_fee_denominator),
            base_need_take_pnl: u64::from_le_bytes(*base_need_take_pnl),
            quote_need_take_pnl: u64::from_le_bytes(*quote_need_take_pnl),
            quote_total_pnl: u64::from_le_bytes(*quote_total_pnl),
            base_total_pnl: u64::from_le_bytes(*base_total_pnl),
            pool_open_time: u64::from_le_bytes(*pool_open_time),
            punish_pc_amount: u64::from_le_bytes(*punish_pc_amount),
            punish_coin_amount: u64::from_le_bytes(*punish_coin_amount),
            orderbook_to_init_time: u64::from_le_bytes(*orderbook_to_init_time),
            swap_base_in_amount: u128::from_le_bytes(*swap_base_in_amount),
            swap_quote_out_amount: u128::from_le_bytes(*swap_quote_out_amount),
            swap_base2_quote_fee: u64::from_le_bytes(*swap_base2_quote_fee),
            swap_quote_in_amount: u128::from_le_bytes(*swap_quote_in_amount),
            swap_base_out_amount: u128::from_le_bytes(*swap_base_out_amount),
            swap_quote2_base_fee: u64::from_le_bytes(*swap_quote2_base_fee),
            base_vault: Pubkey::new_from_array(*base_vault),
            quote_vault: Pubkey::new_from_array(*quote_vault),
            base_mint: Pubkey::new_from_array(*base_mint),
            quote_mint: Pubkey::new_from_array(*quote_mint),
            lp_mint: Pubkey::new_from_array(*lp_mint),
            open_orders: Pubkey::new_from_array(*open_orders),
            market_id: Pubkey::new_from_array(*market_id),
            market_program_id: Pubkey::new_from_array(*market_program_id),
            target_orders: Pubkey::new_from_array(*target_orders),
            withdraw_queue: Pubkey::new_from_array(*withdraw_queue),
            lp_vault: Pubkey::new_from_array(*lp_vault),
            owner: Pubkey::new_from_array(*owner),
            lp_reserve: u64::from_le_bytes(*lp_reserve),
            padding: [0,0,0], // temp
        }
    }
}

impl PoolOperation for RaydiumCpmmMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.base_mint,
            pubkey_b: self.quote_mint
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.base_vault,
            pubkey_b: self.quote_vault
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(String, Pubkey)> {
        vec![
            (stringify!(self.base_vault).to_string(), self.base_vault),
            (stringify!(self.quote_vault).to_string(), self.quote_vault),
        ]
    }

    fn get_formula(&self) -> Formula {
        ConstantProduct
    }
}