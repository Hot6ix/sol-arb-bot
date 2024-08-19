use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketOperation, MarketSerializer, PubkeyPair};

pub struct LifinityMarket {
    pub initializer_key: Pubkey,
    pub initializer_deposit_token_account: Pubkey,
    pub initializer_receiver_token_account: Pubkey,
    pub initializer_amount: u64,
    pub taker_amount: u64,
    pub is_initialized: bool,
    pub bump_seed: u8,
    pub freeze_trade: u8,
    pub freeze_deposit: u8,
    pub freeze_withdraw: u8,
    pub base_decimals: u8,
    pub token_program_id: Pubkey,
    pub token_a_account: Pubkey,
    pub token_b_account: Pubkey,
    pub pool_mint: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub fee_account: Pubkey,
    pub oracle_main_account: Pubkey,
    pub oracle_sub_account: Pubkey,
    pub oracle_pc_account: Pubkey,
    pub fees: AmmFees,
    pub curve: AmmCurve,
    pub config: AmmConfig,
    pub amm_p_temp1: Pubkey,
    pub amm_p_temp2: Pubkey,
    pub amm_p_temp3: Pubkey,
    pub amm_p_temp4: Pubkey,
    pub amm_p_temp5: Pubkey,
}

impl MarketSerializer for LifinityMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        todo!()
    }
}

impl MarketOperation for LifinityMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        todo!()
    }
}

pub struct AmmFees {
    pub trade_fee_numerator: u64,
    pub trade_fee_denominator: u64,
    pub owner_trade_fee_numerator: u64,
    pub owner_trade_fee_denominator: u64,
    pub owner_withdraw_fee_numerator: u64,
    pub owner_withdraw_fee_denominator: u64,
    pub host_fee_numerator: u64,
    pub host_fee_denominator: u64,
}

pub struct AmmCurve {
    pub curve_type: u8,
    pub curve_parameters: u64
}

pub struct AmmConfig {
    pub last_price: u64,
    pub last_balance_price: u64,
    pub config_denominator: u64,
    pub volume_x: u64,
    pub volume_y: u64,
    pub volume_x_in_y: u64,
    pub deposit_cap: u64,
    pub regression_target:u64,
    pub oracle_type: u64,
    pub oracle_status: u64,
    pub oracle_main_slot_limit: u64,
    pub oracle_sub_confidence_limit: u64,
    pub oracle_sub_slot_limit: u64,
    pub oracle_pc_confidence_limit: u64,
    pub std_spread: u64,
    pub std_spread_buffer: u64,
    pub spread_coefficient: u64,
    pub price_buffer_coin: i64,
    pub price_buffer_pc: i64,
    pub rebalance_ratio: u64,
    pub fee_trade: u64,
    pub fee_platform: u64,
    pub config_temp3: u64,
    pub config_temp4: u64,
    pub config_temp5: u64,
    pub config_temp6: u64,
    pub config_temp7: u64,
    pub config_temp8: u64,
}