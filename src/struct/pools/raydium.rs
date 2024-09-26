use std::any::Any;
use std::collections::VecDeque;

use arrayref::{array_ref, array_refs};
use num_enum::TryFromPrimitive;
use numext_fixed_uint::U1024;
use serde::Deserialize;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedConfigAccount, DeserializedPoolAccount, DeserializedTokenAccount};
use crate::account::account::DeserializedConfigAccount::RaydiumClmmConfigAccount;
use crate::constants::*;
use crate::formula::base::Formula;
use crate::formula::base::Formula::{ConcentratedLiquidity, ConstantProduct};
use crate::formula::clmm::constant::{MAX_TICK, MIN_TICK, POOL_SEED};
use crate::formula::clmm::tick_array::{check_current_tick_array_is_initialized, max_tick_in_tick_array_bitmap, next_initialized_tick_array_start_index, TickArrayBitmapExtension, TickArrayBitmapExtensionAccount, TickArrayState, TickArrayStateAccount};
use crate::formula::constant_product::{ConstantProductBase, DefaultConstantProduct};
use crate::formula::raydium_clmm::swap_internal;
use crate::r#struct::market::{Market, PoolOperation};
use crate::utils::PubkeyPair;

#[derive(Copy, Clone, Debug, Default)]
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
    pub reward_info: [RaydiumRewardInfo; 3], // 169 * 3; 507
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

        let padding1_array: Vec<u64> = padding1.chunks_exact(8).map(|array| {
            u64::from_le_bytes((*array).try_into().unwrap())
        }).collect::<Vec<u64>>();
        let padding2_array: Vec<u64> = padding2.chunks_exact(8).map(|array| {
            u64::from_le_bytes((*array).try_into().unwrap())
        }).collect::<Vec<u64>>();

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
            reward_info: RaydiumRewardInfo::unpack_data_set(*reward_infos),
            tick_array_bitmap: bytemuck::cast(*tick_array_bitmap),
            total_fees_token_0: u64::from_le_bytes(*total_fees_token_0),
            total_fees_claimed_token_0: u64::from_le_bytes(*total_fees_claimed_token_0),
            total_fees_token_1: u64::from_le_bytes(*total_fees_token_1),
            total_fees_claimed_token_1: u64::from_le_bytes(*total_fees_claimed_token_1),
            fund_fees_token_0: u64::from_le_bytes(*fund_fees_token_0),
            fund_fees_token_1: u64::from_le_bytes(*fund_fees_token_1),
            open_time: u64::from_le_bytes(*open_time),
            recent_epoch: u64::from_le_bytes(*recent_epoch),
            padding1: padding1_array.try_into().unwrap(),
            padding2: padding2_array.try_into().unwrap()
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

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        vec![
            (DeserializedAccount::ConfigAccount(DeserializedConfigAccount::default()), self.amm_config),
            // (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_vault_0),
            // (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_vault_1),
            // (DeserializedAccount::ConfigAccount(DeserializedConfigAccount::default()), self.observation_key),
        ]
    }

    fn get_formula(&self) -> Formula {
        ConcentratedLiquidity
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        let mut market = RaydiumClmmMarket::default();
        let mut amm_config = AmmConfig::default();
        let mut tick_array_states: VecDeque<TickArrayState> = VecDeque::new();
        let mut tick_array_bitmap_extension = TickArrayBitmapExtension::default();

        accounts.iter().for_each(|account| {
            match account {
                DeserializedAccount::PoolAccount(pool) => {
                    if let Some(raydium_clmm_market) = pool.operation.as_any().downcast_ref::<RaydiumClmmMarket>() {
                        market = *raydium_clmm_market;
                    }
                }
                DeserializedAccount::ConfigAccount(config) => {
                    match config {
                        RaydiumClmmConfigAccount(raydium_config) => {
                            match raydium_config {
                                RaydiumClmmAccount::AmmConfig(amm) => {
                                    amm_config = amm.config
                                }
                                RaydiumClmmAccount::ObservationKey => {}
                                RaydiumClmmAccount::TickArrayState(state) => {
                                    tick_array_states.push_back(state.tick_array_state.clone())
                                }
                                RaydiumClmmAccount::TickArrayBitmapExtension(extension) => {
                                    tick_array_bitmap_extension = extension.tick_array_bitmap_extension.clone()
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                DeserializedAccount::Account(_) => {}
                DeserializedAccount::TokenAccount(_) => {}
            }
        });

        let sqrt_price_x64 = market.sqrt_price_x64;

        swap_internal(
            &amm_config,
            &mut market,
            &mut tick_array_states,
            &Some(&tick_array_bitmap_extension),
            0u128,
            sqrt_price_x64,
            false,
            false
        ).expect("swap failed");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl RaydiumClmmMarket {
    pub fn key(&self, program_id: &Pubkey) -> Pubkey {
        Pubkey::create_program_address(
            &[
                &POOL_SEED.as_bytes(),
                self.amm_config.as_ref(),
                self.token_mint_0.as_ref(),
                self.token_mint_1.as_ref(),
                self.bump.as_ref(),
            ],
            program_id
        ).unwrap()
    }

    pub fn resolve(pubkey: Pubkey, account: Account) -> RaydiumClmmAccount {
        RaydiumClmmAccount::AmmConfig(
            AmmConfigAccount {
                pubkey,
                config: account.deserialize_data::<AmmConfig>().unwrap(),
                market: Market::RAYDIUM
            }
        )
    }

    pub fn is_overflow_default_tick_array_bitmap(&self, tick_index_array: Vec<i32>) -> bool {
        let (min_tick_array_start_index_boundary, max_tick_array_index_boundary) =
            self.tick_array_start_index_range();
        for tick_index in tick_index_array {
            let tick_array_start_index =
                TickArrayState::get_array_start_index(tick_index, self.tick_spacing);
            if tick_array_start_index >= max_tick_array_index_boundary
                || tick_array_start_index < min_tick_array_start_index_boundary
            {
                return true;
            }
        }
        false
    }

    pub fn tick_array_start_index_range(&self) -> (i32, i32) {
        let mut max_tick_boundary =
            max_tick_in_tick_array_bitmap(self.tick_spacing);
        let mut min_tick_boundary = -max_tick_boundary;
        if max_tick_boundary > MAX_TICK {
            max_tick_boundary =
                TickArrayState::get_array_start_index(MAX_TICK, self.tick_spacing);
            // find the next tick array start index
            max_tick_boundary = max_tick_boundary + TickArrayState::tick_count(self.tick_spacing);
        }
        if min_tick_boundary < MIN_TICK {
            min_tick_boundary =
                TickArrayState::get_array_start_index(MIN_TICK, self.tick_spacing);
        }
        (min_tick_boundary, max_tick_boundary)
    }

    pub fn get_first_initialized_tick_array(
        &self,
        tick_array_bitmap_extension: &Option<&TickArrayBitmapExtension>,
        zero_for_one: bool,
    ) -> Result<(bool, i32), &'static str> {
        let (is_initialized, start_index) =
            if self.is_overflow_default_tick_array_bitmap(vec![self.tick_current]) {
                tick_array_bitmap_extension
                    .unwrap()
                    .check_tick_array_is_initialized(
                        TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing), //-20520
                        self.tick_spacing,
                    )?
            } else {
                check_current_tick_array_is_initialized(
                    U1024(self.tick_array_bitmap),
                    self.tick_current,
                    self.tick_spacing.into(),
                )?
            };
        if is_initialized {
            return Ok((true, start_index));
        }
        let next_start_index = self.next_initialized_tick_array_start_index(
            tick_array_bitmap_extension,
            TickArrayState::get_array_start_index(self.tick_current, self.tick_spacing),
            zero_for_one,
        )?;
        return Ok((false, next_start_index.unwrap()))
    }

    pub fn next_initialized_tick_array_start_index(
        &self,
        tick_array_bitmap_extension: &Option<&TickArrayBitmapExtension>,
        mut last_tick_array_start_index: i32,
        zero_for_one: bool,
    ) -> Result<Option<i32>, &'static str> {
        last_tick_array_start_index =
            TickArrayState::get_array_start_index(last_tick_array_start_index, self.tick_spacing);

        loop {
            let (is_found, start_index) =
                next_initialized_tick_array_start_index(
                    U1024(self.tick_array_bitmap),
                    last_tick_array_start_index,
                    self.tick_spacing,
                    zero_for_one,
                );
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if tick_array_bitmap_extension.is_none() {
                return Err("missing tick array bitmap extension account");
            }

            let (is_found, start_index) = tick_array_bitmap_extension
                .unwrap()
                .next_initialized_tick_array_from_one_bitmap(
                    last_tick_array_start_index,
                    self.tick_spacing,
                    zero_for_one,
                )?;
            if is_found {
                return Ok(Some(start_index));
            }
            last_tick_array_start_index = start_index;

            if last_tick_array_start_index < MIN_TICK
                || last_tick_array_start_index > MAX_TICK
            {
                return Ok(None);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct RaydiumRewardInfo { // 169
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

impl RaydiumRewardInfo {
    pub fn initialized(&self) -> bool {
        self.token_mint.ne(&Pubkey::default())
    }
}

impl AccountDataSerializer for RaydiumRewardInfo {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 169];
        let (reward_state, open_time, end_time, last_update_time, emissions_per_second_x64, reward_total_emissioned, reward_claimed, token_mint, token_vault, authority, reward_growth_global_x64) =
            array_refs![src, 1, 8, 8, 8, 16, 8, 8, 32, 32, 32, 16];

        RaydiumRewardInfo {
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

impl RaydiumRewardInfo {
    fn unpack_data_set(data: [u8; 507]) -> [RaydiumRewardInfo; 3] {
        let mut vec: Vec<RaydiumRewardInfo> = Vec::new();

        data.chunks_exact(169).for_each(|array| {
            vec.push(RaydiumRewardInfo::unpack_data(&array.to_vec()))
        });

        vec.try_into().unwrap()
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Default)]
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

#[derive(Clone, Deserialize, PartialEq)]
pub struct AmmConfigAccount {
    pub pubkey: Pubkey,
    pub config: AmmConfig,
    pub market: Market,
}

#[derive(Clone, PartialEq)]
pub enum RaydiumClmmAccount {
    AmmConfig(AmmConfigAccount),
    ObservationKey,
    TickArrayState(TickArrayStateAccount),
    TickArrayBitmapExtension(TickArrayBitmapExtensionAccount)
}

impl RaydiumClmmAccount {
    pub fn get_pubkey(&self) -> Pubkey {
        match self {
            RaydiumClmmAccount::AmmConfig(account) => {
                account.pubkey
            }
            _ => {
                Pubkey::default()
            }
        }
    }

    pub fn get_market(&self) -> Market {
        Market::RAYDIUM
    }

    pub fn resolve_account(pubkey: Pubkey, data: &Vec<u8>) -> RaydiumClmmAccount {
        match data.len() {
            RAYDIUM_CLMM_AMM_CONFIG => {
                RaydiumClmmAccount::AmmConfig(AmmConfigAccount {
                    pubkey,
                    config: AmmConfig::unpack_data(data),
                    market: Market::RAYDIUM
                })
            }
            RAYDIUM_CLMM_OBSERVATION_KEY => {
                // todo
                RaydiumClmmAccount::ObservationKey
            }
            RAYDIUM_CLMM_TICK_ARRAY_BITMAP_EXTENSION => {
                RaydiumClmmAccount::TickArrayBitmapExtension(TickArrayBitmapExtensionAccount {
                    pubkey,
                    market: Market::RAYDIUM,
                    tick_array_bitmap_extension: TickArrayBitmapExtension::unpack_data(data),
                })
            }
            RAYDIUM_CLMM_TICK_ARRAY_STATE => {
                RaydiumClmmAccount::TickArrayState(TickArrayStateAccount {
                    pubkey,
                    market: Market::RAYDIUM,
                    tick_array_state: TickArrayState::unpack_data(data)
                })
            }
            _ => {
                panic!("could not resolve account from data: pubkey({})", pubkey)
            }
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone, Debug, Default)]
pub struct RaydiumOpenBookMarket {
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

impl AccountDataSerializer for RaydiumOpenBookMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        // no discriminator for native solana program
        let src = array_ref![data, 0, 752];
        let (status, nonce, max_order, depth, base_decimal, quote_decimal, state, reset_flag, min_size, vol_max_cut_ratio, amount_wave_ratio, base_lot_size, quote_lot_size, mint_price_multiplier, max_price_multiplier, system_decimal_value, min_separate_numerator, min_separate_denominator, trade_fee_numerator, trade_fee_denominator, pnl_numerator, pnl_denominator, swap_fee_numerator, swap_fee_denominator, base_need_take_pnl, quote_need_take_pnl, quote_total_pnl, base_total_pnl, pool_open_time, punish_pc_amount, punish_coin_amount, orderbook_to_init_time, swap_base_in_amount, swap_quote_out_amount, swap_base2_quote_fee, swap_quote_in_amount, swap_base_out_amount, swap_quote2_base_fee, base_vault, quote_vault, base_mint, quote_mint, lp_mint, open_orders, market_id, market_program_id, target_orders, withdraw_queue, lp_vault, owner, lp_reserve, padding) =
            array_refs![src, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 8, 16, 16, 8, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 8, 24];

        let padding_array: Vec<u64> = padding.chunks_exact(8).map(|array| {
            u64::from_le_bytes(array.try_into().unwrap())
        }).collect::<Vec<u64>>();

        RaydiumOpenBookMarket {
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
            padding: padding_array.try_into().unwrap()
        }
    }
}

impl PoolOperation for RaydiumOpenBookMarket {
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

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        vec![
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.base_vault),
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.quote_vault),
        ]
    }

    fn get_formula(&self) -> Formula {
        Formula::OpenBook
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        todo!()
        // if !accounts.is_empty() {
        //     let mut raydium_pool= &DeserializedPoolAccount::default();
        //     let mut base_vault= &DeserializedTokenAccount::default();
        //     let mut quote_vault= &DeserializedTokenAccount::default();
        //
        //     accounts.iter().for_each(|account| {
        //         match account {
        //             DeserializedAccount::Account(_) => {}
        //             DeserializedAccount::PoolAccount(pool) => {
        //                 raydium_pool = &pool
        //             }
        //             DeserializedAccount::TokenAccount(token) => {
        //                 if token.pubkey == self.base_vault {
        //                     base_vault = &token
        //                 }
        //                 else if token.pubkey == self.quote_vault {
        //                     quote_vault = &token
        //                 }
        //             }
        //             DeserializedAccount::ConfigAccount(_) => {}
        //         }
        //     });
        //
        //     let cpmm = DefaultConstantProduct {
        //         token_a_amount: base_vault.get_amount(),
        //         token_b_amount: quote_vault.get_amount(),
        //         decimal_diff: (self.base_decimal - self.quote_decimal) as i32,
        //         swap_fee_numerator: self.swap_fee_numerator,
        //         swap_fee_denominator: self.swap_fee_denominator
        //     };
        //
        //     let res = cpmm.swap(1000000000u64, true);
        //     println!("{}", res);
        // }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Deserialize)]
pub enum RaydiumOpenBookAccount {
    Unknown
}