use std::ops::Sub;
use arrayref::{array_ref, array_refs};
use num_bigfloat::BigFloat;
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketSerializer};
use crate::pools::math::calculate_fee;

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

impl MarketSerializer for RaydiumCpmmMarket {
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

// this trait will be merged into MarketOperation
pub trait Accountant {
    fn get_fee(&self);
    fn apply_fee(&self, amount: &u64) -> BigFloat;
}

impl Accountant for RaydiumCpmmMarket {
    fn get_fee(&self) {
        todo!()
    }

    fn apply_fee(&self, amount: &u64) -> BigFloat {
        let fee = calculate_fee(
            self.trade_fee_numerator,
            self.trade_fee_denominator,
            amount
        );

        BigFloat::from(*amount).sub(fee)
    }
}

pub struct RaydiumConfig {
    pub bump: u8,
    pub index: u16,
    pub owner: Pubkey,
    pub protocol_fee_rate: u32,
    pub trade_fee_rate: u32,
    pub tick_spacing: u16,
    pub fund_fee_rate: u32,
    pub padding_u32: u32,
    pub fund_owner: Pubkey,
    pub padding: [u64; 3]
}

impl MarketSerializer for RaydiumConfig {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 108];
        let (bump, index, owner, protocol_fee_rate, trade_fee_rate, tick_spacing, fund_fee_rate, padding_u32, fund_owner, padding) =
            array_refs![src, 1, 2, 32, 4, 4, 2, 4, 4, 32, 24];

        RaydiumConfig {
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