use std::any::Any;
use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;

use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedTokenAccount};
use crate::formula::base::Formula;
use crate::formula::base::Formula::ConcentratedLiquidity;
use crate::r#struct::market::PoolOperation;
use crate::utils::PubkeyPair;

#[derive(Copy, Clone, Debug, Default)]
pub struct LifinityMarket { // 895
    pub initializer_key: Pubkey, // 32
    pub initializer_deposit_token_account: Pubkey, // 32
    pub initializer_receiver_token_account: Pubkey, // 32
    pub initializer_amount: u64, // 8
    pub taker_amount: u64, // 8
    pub is_initialized: bool, // 1
    pub bump_seed: u8, // 1
    pub freeze_trade: u8, // 1
    pub freeze_deposit: u8, // 1
    pub freeze_withdraw: u8, // 1
    pub base_decimals: u8, // 1
    pub token_program_id: Pubkey, // 32
    pub token_a_account: Pubkey, // 32
    pub token_b_account: Pubkey, // 32
    pub pool_mint: Pubkey, // 32
    pub token_a_mint: Pubkey, // 32
    pub token_b_mint: Pubkey, // 32
    pub fee_account: Pubkey, // 32
    pub oracle_main_account: Pubkey, // 32
    pub oracle_sub_account: Pubkey, // 32
    pub oracle_pc_account: Pubkey, // 32
    pub fees: AmmFees, // 64
    pub curve: AmmCurve, // 9
    pub config: AmmConfig, // 224
    pub amm_p_temp1: Pubkey, // 32
    pub amm_p_temp2: Pubkey, // 32
    pub amm_p_temp3: Pubkey, // 32
    pub amm_p_temp4: Pubkey, // 32
    pub amm_p_temp5: Pubkey, // 32
}

impl AccountDataSerializer for LifinityMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 903];
        let (discriminator, initializer_key, initializer_deposit_token_account, initializer_receiver_token_account, initializer_amount, taker_amount, is_initialized, bump_seed, freeze_trade, freeze_deposit, freeze_withdraw, base_decimals, token_program_id, token_a_account, token_b_account, pool_mint, token_a_mint, token_b_mint, fee_account, oracle_main_account, oracle_sub_account, oracle_pc_account, fees, curve, config, amm_p_temp1, amm_p_temp2, amm_p_temp3, amm_p_temp4, amm_p_temp5) =
            array_refs![src, 8, 32, 32, 32, 8, 8, 1, 1, 1, 1, 1, 1, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 64, 9, 224, 32, 32, 32, 32, 32];

        LifinityMarket {
            initializer_key: Pubkey::new_from_array(*initializer_key),
            initializer_deposit_token_account: Pubkey::new_from_array(*initializer_deposit_token_account),
            initializer_receiver_token_account: Pubkey::new_from_array(*initializer_receiver_token_account),
            initializer_amount: u64::from_le_bytes(*initializer_amount),
            taker_amount: u64::from_le_bytes(*taker_amount),
            is_initialized: bool::from(true),
            bump_seed: u8::from_le_bytes(*bump_seed),
            freeze_trade: u8::from_le_bytes(*freeze_trade),
            freeze_deposit: u8::from_le_bytes(*freeze_deposit),
            freeze_withdraw: u8::from_le_bytes(*freeze_withdraw),
            base_decimals: u8::from_le_bytes(*base_decimals),
            token_program_id: Pubkey::new_from_array(*token_program_id),
            token_a_account: Pubkey::new_from_array(*token_a_account),
            token_b_account: Pubkey::new_from_array(*token_b_account),
            pool_mint: Pubkey::new_from_array(*pool_mint),
            token_a_mint: Pubkey::new_from_array(*token_a_mint),
            token_b_mint: Pubkey::new_from_array(*token_b_mint),
            fee_account: Pubkey::new_from_array(*fee_account),
            oracle_main_account: Pubkey::new_from_array(*oracle_main_account),
            oracle_sub_account: Pubkey::new_from_array(*oracle_sub_account),
            oracle_pc_account: Pubkey::new_from_array(*oracle_pc_account),
            fees: AmmFees::unpack_data(&Vec::from(fees)),
            curve: AmmCurve::unpack_data(*curve),
            config: AmmConfig::unpack_data(*config),
            amm_p_temp1: Pubkey::new_from_array(*amm_p_temp1),
            amm_p_temp2: Pubkey::new_from_array(*amm_p_temp2),
            amm_p_temp3: Pubkey::new_from_array(*amm_p_temp3),
            amm_p_temp4: Pubkey::new_from_array(*amm_p_temp4),
            amm_p_temp5: Pubkey::new_from_array(*amm_p_temp5),
        }
    }
}

impl PoolOperation for LifinityMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_a_mint,
            pubkey_b: self.token_b_mint
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_a_account,
            pubkey_b: self.token_b_account
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        vec![
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_a_account),
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_b_account),
        ]
    }

    fn get_formula(&self) -> Formula {
        ConcentratedLiquidity
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct AmmFees { // 64
    pub trade_fee_numerator: u64, // 8
    pub trade_fee_denominator: u64, // 8
    pub owner_trade_fee_numerator: u64, // 8
    pub owner_trade_fee_denominator: u64, // 8
    pub owner_withdraw_fee_numerator: u64, // 8
    pub owner_withdraw_fee_denominator: u64, // 8
    pub host_fee_numerator: u64, // 8
    pub host_fee_denominator: u64, // 8
}

impl AccountDataSerializer for AmmFees {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 64];
        let (trade_fee_numerator, trade_fee_denominator, owner_trade_fee_numerator, owner_trade_fee_denominator, owner_withdraw_fee_numerator, owner_withdraw_fee_denominator, host_fee_numerator, host_fee_denominator) =
            array_refs![src, 8, 8, 8, 8, 8, 8, 8, 8];

        AmmFees {
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            owner_trade_fee_numerator: u64::from_le_bytes(*owner_trade_fee_numerator),
            owner_trade_fee_denominator: u64::from_le_bytes(*owner_trade_fee_denominator),
            owner_withdraw_fee_numerator: u64::from_le_bytes(*owner_withdraw_fee_numerator),
            owner_withdraw_fee_denominator: u64::from_le_bytes(*owner_withdraw_fee_denominator),
            host_fee_numerator: u64::from_le_bytes(*host_fee_numerator),
            host_fee_denominator: u64::from_le_bytes(*host_fee_denominator),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct AmmCurve { // 9
    pub curve_type: u8, // 1
    pub curve_parameters: u64 // 8
}

impl AmmCurve {
    pub fn unpack_data(data: [u8; 9]) -> AmmCurve {
        let src = array_ref![data, 0, 9];
        let (curve_type, curve_parameters) =
            array_refs![src, 1, 8];

        AmmCurve {
            curve_type: u8::from_le_bytes(*curve_type),
            curve_parameters: u64::from_le_bytes(*curve_parameters),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct AmmConfig { // 224
    pub last_price: u64, // 8
    pub last_balance_price: u64, // 8
    pub config_denominator: u64, // 8
    pub volume_x: u64, // 8
    pub volume_y: u64, // 8
    pub volume_x_in_y: u64, // 8
    pub deposit_cap: u64, // 8
    pub regression_target: u64, // 8
    pub oracle_type: u64, // 8
    pub oracle_status: u64, // 8
    pub oracle_main_slot_limit: u64, // 8
    pub oracle_sub_confidence_limit: u64, // 8
    pub oracle_sub_slot_limit: u64, // 8
    pub oracle_pc_confidence_limit: u64, // 8
    pub std_spread: u64, // 8
    pub std_spread_buffer: u64, // 8
    pub spread_coefficient: u64, // 8
    pub price_buffer_coin: i64, // 8
    pub price_buffer_pc: i64, // 8
    pub rebalance_ratio: u64, // 8
    pub fee_trade: u64, // 8
    pub fee_platform: u64, // 8
    pub config_temp3: u64, // 8
    pub config_temp4: u64, // 8
    pub config_temp5: u64, // 8
    pub config_temp6: u64, // 8
    pub config_temp7: u64, // 8
    pub config_temp8: u64, // 8
}

impl AmmConfig {
    pub fn unpack_data(data: [u8; 224]) -> AmmConfig {
        let src = array_ref![data, 0, 224];
        let (last_price, last_balance_price, config_denominator, volume_x, volume_y, volume_x_in_y, deposit_cap, regression_target, oracle_type, oracle_status, oracle_main_slot_limit, oracle_sub_confidence_limit, oracle_sub_slot_limit, oracle_pc_confidence_limit, std_spread, std_spread_buffer, spread_coefficient, price_buffer_coin, price_buffer_pc, rebalance_ratio, fee_trade, fee_platform, config_temp3, config_temp4, config_temp5, config_temp6, config_temp7, config_temp8) =
            array_refs![src, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];

        AmmConfig {
            last_price: u64::from_le_bytes(*last_price),
            last_balance_price: u64::from_le_bytes(*last_balance_price),
            config_denominator: u64::from_le_bytes(*config_denominator),
            volume_x: u64::from_le_bytes(*volume_x),
            volume_y: u64::from_le_bytes(*volume_y),
            volume_x_in_y: u64::from_le_bytes(*volume_x_in_y),
            deposit_cap: u64::from_le_bytes(*deposit_cap),
            regression_target: u64::from_le_bytes(*regression_target),
            oracle_type: u64::from_le_bytes(*oracle_type),
            oracle_status: u64::from_le_bytes(*oracle_status),
            oracle_main_slot_limit: u64::from_le_bytes(*oracle_main_slot_limit),
            oracle_sub_confidence_limit: u64::from_le_bytes(*oracle_sub_confidence_limit),
            oracle_sub_slot_limit: u64::from_le_bytes(*oracle_sub_slot_limit),
            oracle_pc_confidence_limit: u64::from_le_bytes(*oracle_pc_confidence_limit),
            std_spread: u64::from_le_bytes(*std_spread),
            std_spread_buffer: u64::from_le_bytes(*std_spread_buffer),
            spread_coefficient: u64::from_le_bytes(*spread_coefficient),
            price_buffer_coin: i64::from_le_bytes(*price_buffer_coin),
            price_buffer_pc: i64::from_le_bytes(*price_buffer_pc),
            rebalance_ratio: u64::from_le_bytes(*rebalance_ratio),
            fee_trade: u64::from_le_bytes(*fee_trade),
            fee_platform: u64::from_le_bytes(*fee_platform),
            config_temp3: u64::from_le_bytes(*config_temp3),
            config_temp4: u64::from_le_bytes(*config_temp4),
            config_temp5: u64::from_le_bytes(*config_temp5),
            config_temp6: u64::from_le_bytes(*config_temp6),
            config_temp7: u64::from_le_bytes(*config_temp7),
            config_temp8: u64::from_le_bytes(*config_temp8),
        }
    }
}