use std::any::Any;
use std::collections::VecDeque;
use std::ops::BitXor;
use arrayref::{array_ref, array_refs};
use serde::Deserialize;
use solana_sdk::account::Account;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::pubkey::Pubkey;

use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedConfigAccount, DeserializedPoolAccount, DeserializedTokenAccount};
use crate::account::account::DeserializedConfigAccount::RaydiumClmmConfigAccount;
use crate::constants::*;
use crate::formula::base::Formula;
use crate::formula::base::Formula::{ConcentratedLiquidity, ConstantProduct};
use crate::formula::clmm::constant::{MAX_TICK, MIN_TICK, POOL_SEED, REWARD_NUM, TICK_ARRAY_BITMAP_SIZE};
use crate::formula::clmm::raydium_tick_array::{check_current_tick_array_is_initialized, max_tick_in_tick_array_bitmap, next_initialized_tick_array_start_index, TickArrayBitmapExtension, TickArrayBitmapExtensionAccount, TickArrayState, TickArrayStateAccount};
use crate::formula::clmm::u256_math::U1024;
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

    pub fn get_tick_array_offset(&self, tick_array_start_index: i32) -> Result<usize, &'static str> {
        // require!(
        //     TickArrayState::check_is_valid_start_index(tick_array_start_index, self.tick_spacing),
        //     ErrorCode::InvaildTickIndex
        // );
        let tick_array_offset_in_bitmap = tick_array_start_index
            / TickArrayState::tick_count(self.tick_spacing)
            + TICK_ARRAY_BITMAP_SIZE;
        Ok(tick_array_offset_in_bitmap as usize)
    }

    // todo
    fn flip_tick_array_bit_internal(&mut self, tick_array_start_index: i32) -> Result<(), &'static str> {
        let tick_array_offset_in_bitmap = self.get_tick_array_offset(tick_array_start_index)?;

        let tick_array_bitmap = U1024(self.tick_array_bitmap);
        let mask = U1024::one() << tick_array_offset_in_bitmap;
        self.tick_array_bitmap = tick_array_bitmap.bitxor(mask).0;
        Ok(())
    }

    pub fn flip_tick_array_bit(
        &mut self,
        tick_array_bitmap_extension: Option<&AccountInfo>,
        tick_array_start_index: i32,
    ) -> Result<(), &'static str> {
        if self.is_overflow_default_tickarray_bitmap(vec![tick_array_start_index]) {
            // require_keys_eq!(
            //     tickarray_bitmap_extension.unwrap().key(),
            //     TickArrayBitmapExtension::key(self.key())
            // );
            // AccountLoader::<TickArrayBitmapExtension>::try_from(
            //     tick_array_bitmap_extension.unwrap(),
            // )?
            //     .load_mut()?
            //     .flip_tick_array_bit(tick_array_start_index, self.tick_spacing)
            let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension.unwrap().data.borrow().to_vec());
            extension.flip_tick_array_bit(tick_array_start_index, self.tick_spacing)
        } else {
            self.flip_tick_array_bit_internal(tick_array_start_index)
        }

        // self.flip_tick_array_bit_internal(tick_array_start_index)
    }

    pub fn is_overflow_default_tickarray_bitmap(&self, tick_indexs: Vec<i32>) -> bool {
        let (min_tick_array_start_index_boundary, max_tick_array_index_boundary) =
            self.tick_array_start_index_range();
        for tick_index in tick_indexs {
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

    pub fn get_reward_growths(reward_infos: &[RaydiumRewardInfo; REWARD_NUM]) -> [u128; REWARD_NUM] {
        let mut reward_growths = [0u128; REWARD_NUM];
        for i in 0..REWARD_NUM {
            reward_growths[i] = reward_infos[i].reward_growth_global_x64;
        }
        reward_growths
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

#[cfg(test)]
pub mod pool_test {
    use super::*;
    use std::cell::RefCell;
    use std::str::FromStr;

    const PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";

    pub fn build_pool(
        tick_current: i32,
        tick_spacing: u16,
        sqrt_price_x64: u128,
        liquidity: u128,
    ) -> RefCell<RaydiumClmmMarket> {
        let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();
        let mut new_pool = RaydiumClmmMarket::default();
        new_pool.tick_current = tick_current;
        new_pool.tick_spacing = tick_spacing;
        new_pool.sqrt_price_x64 = sqrt_price_x64;
        new_pool.liquidity = liquidity;
        new_pool.token_mint_0 = Pubkey::new_unique();
        new_pool.token_mint_1 = Pubkey::new_unique();
        new_pool.amm_config = Pubkey::new_unique();
        // let mut random = rand::random<u128>();
        new_pool.fee_growth_global_0_x64 = rand::random::<u128>();
        new_pool.fee_growth_global_1_x64 = rand::random::<u128>();
        new_pool.bump = [Pubkey::find_program_address(
            &[
                &POOL_SEED.as_bytes(),
                new_pool.amm_config.as_ref(),
                new_pool.token_mint_0.as_ref(),
                new_pool.token_mint_1.as_ref(),
            ],
            &program_pubkey,
        )
            .1];
        RefCell::new(new_pool)
    }

    mod tick_array_bitmap_test {

        use super::*;

        #[test]
        fn get_arrary_start_index_negative() {
            let mut pool_state = RaydiumClmmMarket::default();
            pool_state.tick_spacing = 10;
            pool_state.flip_tick_array_bit(None, -600).unwrap();
            assert!(U1024(pool_state.tick_array_bitmap).bit(511) == true);

            pool_state.flip_tick_array_bit(None, -1200).unwrap();
            assert!(U1024(pool_state.tick_array_bitmap).bit(510) == true);

            pool_state.flip_tick_array_bit(None, -1800).unwrap();
            assert!(U1024(pool_state.tick_array_bitmap).bit(509) == true);

            pool_state.flip_tick_array_bit(None, -38400).unwrap();
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(-38400).unwrap())
                    == true
            );
            pool_state.flip_tick_array_bit(None, -39000).unwrap();
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(-39000).unwrap())
                    == true
            );
            pool_state.flip_tick_array_bit(None, -307200).unwrap();
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(-307200).unwrap())
                    == true
            );
        }

        #[test]
        fn get_arrary_start_index_positive() {
            let mut pool_state = RaydiumClmmMarket::default();
            pool_state.tick_spacing = 10;
            pool_state.flip_tick_array_bit(None, 0).unwrap();
            assert!(pool_state.get_tick_array_offset(0).unwrap() == 512);
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(0).unwrap())
                    == true
            );

            pool_state.flip_tick_array_bit(None, 600).unwrap();
            assert!(pool_state.get_tick_array_offset(600).unwrap() == 513);
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(600).unwrap())
                    == true
            );

            pool_state.flip_tick_array_bit(None, 1200).unwrap();
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(1200).unwrap())
                    == true
            );

            pool_state.flip_tick_array_bit(None, 38400).unwrap();
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(38400).unwrap())
                    == true
            );

            pool_state.flip_tick_array_bit(None, 306600).unwrap();
            assert!(pool_state.get_tick_array_offset(306600).unwrap() == 1023);
            assert!(
                U1024(pool_state.tick_array_bitmap)
                    .bit(pool_state.get_tick_array_offset(306600).unwrap())
                    == true
            );
        }

        #[test]
        fn default_tick_array_start_index_range_test() {
            let mut pool_state = RaydiumClmmMarket::default();
            pool_state.tick_spacing = 60;
            // -443580 is the min tick can use to open a position when tick_spacing is 60 due to MIN_TICK is -443636
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![-443580]) == false);
            // 443580 is the min tick can use to open a position when tick_spacing is 60 due to MAX_TICK is 443636
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![443580]) == false);

            pool_state.tick_spacing = 10;
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![-307200]) == false);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![-307201]) == true);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![307200]) == true);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![307199]) == false);

            pool_state.tick_spacing = 1;
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![-30720]) == false);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![-30721]) == true);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![30720]) == true);
            assert!(pool_state.is_overflow_default_tickarray_bitmap(vec![30719]) == false);
        }
    }

    mod use_tickarray_bitmap_extension_test {

        use std::ops::Deref;
        use crate::formula::clmm::constant::{POOL_TICK_ARRAY_BITMAP_SEED, TICK_ARRAY_SIZE};
        use crate::formula::clmm::raydium_tick_array::tick_array_bitmap_extension_test::{build_tick_array_bitmap_extension_info, BuildExtensionAccountInfo};
        use crate::formula::clmm::raydium_tick_math::get_sqrt_price_at_tick;
        use super::*;

        // todo: this code does not work
        pub fn pool_flip_tick_array_bit_helper(
            pool_state: &mut RaydiumClmmMarket,
            tickarray_bitmap_extension: Option<&AccountInfo>,
            init_tick_array_start_indexs: Vec<i32>,
        ) {
            for start_index in init_tick_array_start_indexs {
                pool_state
                    .flip_tick_array_bit(tickarray_bitmap_extension, start_index)
                    .unwrap();
            }
        }

        #[test]
        fn get_first_initialized_tick_array_test() {
            let tick_spacing = 1;
            let tick_current = tick_spacing * TICK_ARRAY_SIZE * 511 - 1;
            let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

            let pool_state_refcel = build_pool(
                tick_current,
                tick_spacing.try_into().unwrap(),
                get_sqrt_price_at_tick(tick_current).unwrap(),
                0,
            );

            let mut pool_state = pool_state_refcel.borrow_mut();

            let param: &mut BuildExtensionAccountInfo = &mut BuildExtensionAccountInfo::default();
            param.key = Pubkey::find_program_address(
                &[
                    POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                    pool_state.key(&program_pubkey).as_ref(),
                ],
                &program_pubkey,
            ).0;
            let tick_array_bitmap_extension_info: AccountInfo<'_> =
                build_tick_array_bitmap_extension_info(param);

            pool_flip_tick_array_bit_helper(
                &mut pool_state,
                Some(&tick_array_bitmap_extension_info),
                vec![
                    -tick_spacing * TICK_ARRAY_SIZE * 513, // tick in extension
                    tick_spacing * TICK_ARRAY_SIZE * 511,
                    tick_spacing * TICK_ARRAY_SIZE * 512, // tick in extension
                ],
            );

            let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

            /////// instead of using pool_flip_tick_array_bit_helper, flip manually
            let init_tick_array_start_index = vec![
                -tick_spacing * TICK_ARRAY_SIZE * 513, // tick in extension
                tick_spacing * TICK_ARRAY_SIZE * 511,
                tick_spacing * TICK_ARRAY_SIZE * 512, // tick in extension
            ];
            for start_index in init_tick_array_start_index {
                extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
            }
            ///////

            let tick_array_bitmap_extension = Some(&extension);

            let (is_first_initilzied, start_index) = pool_state
                .get_first_initialized_tick_array(&tick_array_bitmap_extension, true)
                .unwrap();
            assert!(is_first_initilzied == false);
            assert!(start_index == -tick_spacing * TICK_ARRAY_SIZE * 513);

            let (is_first_initilzied, start_index) = pool_state
                .get_first_initialized_tick_array(&tick_array_bitmap_extension, false)
                .unwrap();
            assert!(is_first_initilzied == false);
            assert!(start_index == tick_spacing * TICK_ARRAY_SIZE * 511);

            pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 511;
            let (is_first_initilzied, start_index) = pool_state
                .get_first_initialized_tick_array(&tick_array_bitmap_extension, true)
                .unwrap();
            assert!(is_first_initilzied == true);
            assert!(start_index == tick_spacing * TICK_ARRAY_SIZE * 511);

            pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 512;
            let (is_first_initilzied, start_index) = pool_state
                .get_first_initialized_tick_array(&tick_array_bitmap_extension, true)
                .unwrap();
            assert!(is_first_initilzied == true);
            assert!(start_index == tick_spacing * TICK_ARRAY_SIZE * 512);
        }

        mod next_initialized_tick_array_start_index_test {

            use super::*;
            #[test]
            fn from_pool_bitmap_to_extension_negative_bitmap() {
                let tick_spacing = 1;
                let tick_current = tick_spacing * TICK_ARRAY_SIZE * 511;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let pool_state_refcel = build_pool(
                    tick_current,
                    tick_spacing.try_into().unwrap(),
                    get_sqrt_price_at_tick(tick_current).unwrap(),
                    0,
                );

                let mut pool_state = pool_state_refcel.borrow_mut();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                param.key = Pubkey::find_program_address(
                    &[
                        POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                        pool_state.key(&program_pubkey).as_ref(),
                    ],
                    &program_pubkey,
                )
                    .0;

                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                // let tick_array_bitmap_extension = Some(
                //     *AccountLoader::<TickArrayBitmapExtension>::try_from(
                //         &tick_array_bitmap_extension_info,
                //     )
                //         .unwrap()
                //         .load()
                //         .unwrap()
                //         .deref(),
                // );
                let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

                /////// instead of using pool_flip_tick_array_bit_helper, flip manually
                let init_tick_array_start_index = vec![
                    -tick_spacing * TICK_ARRAY_SIZE * 7394, // max negative tick array start index boundary in extension
                    -tick_spacing * TICK_ARRAY_SIZE * 1000, // tick in extension
                    -tick_spacing * TICK_ARRAY_SIZE * 513,  // tick in extension
                    tick_spacing * TICK_ARRAY_SIZE * 510,   // tick in pool bitmap
                ];
                for start_index in init_tick_array_start_index {
                    extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
                }
                ///////

                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert_eq!(start_index.unwrap(), tick_spacing * TICK_ARRAY_SIZE * 510);

                pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 510;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == -tick_spacing * TICK_ARRAY_SIZE * 513);

                pool_state.tick_current = -tick_spacing * TICK_ARRAY_SIZE * 513;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == -tick_spacing * TICK_ARRAY_SIZE * 1000);

                pool_state.tick_current = -tick_spacing * TICK_ARRAY_SIZE * 7393;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == -tick_spacing * TICK_ARRAY_SIZE * 7394);

                pool_state.tick_current = -tick_spacing * TICK_ARRAY_SIZE * 7394;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.is_none() == true);
            }

            #[test]
            fn from_pool_bitmap_to_extension_positive_bitmap() {
                let tick_spacing = 1;
                let tick_current = 0;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let pool_state_refcel = build_pool(
                    tick_current,
                    tick_spacing.try_into().unwrap(),
                    get_sqrt_price_at_tick(tick_current).unwrap(),
                    0,
                );

                let mut pool_state = pool_state_refcel.borrow_mut();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                param.key = Pubkey::find_program_address(
                    &[
                        POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                        pool_state.key(&program_pubkey).as_ref(),
                    ],
                    &program_pubkey,
                )
                    .0;
                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                // let tick_array_bitmap_extension = Some(
                //     *AccountLoader::<TickArrayBitmapExtension>::try_from(
                //         &tick_array_bitmap_extension_info,
                //     )
                //         .unwrap()
                //         .load()
                //         .unwrap()
                //         .deref(),
                // );
                let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

                /////// instead of using pool_flip_tick_array_bit_helper, flip manually
                let init_tick_array_start_index = vec![
                    tick_spacing * TICK_ARRAY_SIZE * 510,  // tick in pool bitmap
                    tick_spacing * TICK_ARRAY_SIZE * 511,  // tick in pool bitmap
                    tick_spacing * TICK_ARRAY_SIZE * 512,  // tick in extension boundary
                    tick_spacing * TICK_ARRAY_SIZE * 7393, // max positvie tick array start index boundary in extension
                ];
                for start_index in init_tick_array_start_index {
                    extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
                }
                ///////

                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == tick_spacing * TICK_ARRAY_SIZE * 510);

                pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 510;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == tick_spacing * TICK_ARRAY_SIZE * 511);

                pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 511;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == tick_spacing * TICK_ARRAY_SIZE * 512);

                pool_state.tick_current = tick_spacing * TICK_ARRAY_SIZE * 7393;
                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.is_none() == true);
            }

            #[test]
            fn from_extension_negative_bitmap_to_extension_positive_bitmap() {
                let tick_spacing = 1;
                let tick_current = -tick_spacing * TICK_ARRAY_SIZE * 999;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let pool_state_refcel = build_pool(
                    tick_current,
                    tick_spacing.try_into().unwrap(),
                    get_sqrt_price_at_tick(tick_current).unwrap(),
                    0,
                );

                let mut pool_state = pool_state_refcel.borrow_mut();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                param.key = Pubkey::find_program_address(
                    &[
                        POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                        pool_state.key(&program_pubkey).as_ref(),
                    ],
                    &program_pubkey,
                )
                    .0;

                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                // let tick_array_bitmap_extension = Some(
                //     *AccountLoader::<TickArrayBitmapExtension>::try_from(
                //         &tick_array_bitmap_extension_info,
                //     )
                //         .unwrap()
                //         .load()
                //         .unwrap()
                //         .deref(),
                // );
                let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

                /////// instead of using pool_flip_tick_array_bit_helper, flip manually
                let init_tick_array_start_index = vec![
                    -tick_spacing * TICK_ARRAY_SIZE * 1000, // tick in extension
                    tick_spacing * TICK_ARRAY_SIZE * 512,   // tick in extension boundary
                    tick_spacing * TICK_ARRAY_SIZE * 1000,  // tick in extension
                ];
                for start_index in init_tick_array_start_index {
                    extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
                }
                ///////

                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == tick_spacing * TICK_ARRAY_SIZE * 512);
            }

            #[test]
            fn from_extension_positive_bitmap_to_extension_negative_bitmap() {
                let tick_spacing = 1;
                let tick_current = tick_spacing * TICK_ARRAY_SIZE * 999;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let pool_state_refcel = build_pool(
                    tick_current,
                    tick_spacing.try_into().unwrap(),
                    get_sqrt_price_at_tick(tick_current).unwrap(),
                    0,
                );

                let mut pool_state = pool_state_refcel.borrow_mut();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                param.key = Pubkey::find_program_address(
                    &[
                        POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                        pool_state.key(&program_pubkey).as_ref(),
                    ],
                    &program_pubkey,
                )
                    .0;
                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

                /////// instead of using pool_flip_tick_array_bit_helper, flip manually
                let init_tick_array_start_index = vec![
                    -tick_spacing * TICK_ARRAY_SIZE * 1000, // tick in extension
                    -tick_spacing * TICK_ARRAY_SIZE * 513,  // tick in extension
                    tick_spacing * TICK_ARRAY_SIZE * 1000,  // tick in extension
                ];
                for start_index in init_tick_array_start_index {
                    extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
                }
                ///////

                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == -tick_spacing * TICK_ARRAY_SIZE * 513);
            }

            #[test]
            fn no_initialized_tick_array() {
                let mut pool_state = RaydiumClmmMarket::default();
                pool_state.tick_spacing = 1;
                pool_state.tick_current = 0;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                pool_flip_tick_array_bit_helper(
                    &mut pool_state,
                    Some(&tick_array_bitmap_extension_info),
                    vec![],
                );

                // let tick_array_bitmap_extension = Some(
                //     *AccountLoader::<TickArrayBitmapExtension>::try_from(
                //         &tick_array_bitmap_extension_info,
                //     )
                //         .unwrap()
                //         .load()
                //         .unwrap()
                //         .deref(),
                // );
                let extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());
                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        true,
                    )
                    .unwrap();
                assert!(start_index.is_none());

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        pool_state.tick_current,
                        false,
                    )
                    .unwrap();
                assert!(start_index.is_none());
            }

            #[test]
            fn min_tick_max_tick_initialized_test() {
                let tick_spacing = 1;
                let tick_current = 0;
                let program_pubkey: Pubkey = Pubkey::from_str(PROGRAM_ID).unwrap();

                let pool_state_refcel = build_pool(
                    tick_current,
                    tick_spacing.try_into().unwrap(),
                    get_sqrt_price_at_tick(tick_current).unwrap(),
                    0,
                );

                let mut pool_state = pool_state_refcel.borrow_mut();

                let param: &mut BuildExtensionAccountInfo =
                    &mut BuildExtensionAccountInfo::default();
                param.key = Pubkey::find_program_address(
                    &[
                        POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
                        pool_state.key(&program_pubkey).as_ref(),
                    ],
                    &program_pubkey,
                )
                    .0;
                let tick_array_bitmap_extension_info: AccountInfo<'_> =
                    build_tick_array_bitmap_extension_info(param);

                // let tick_array_bitmap_extension = Some(
                //     *AccountLoader::<TickArrayBitmapExtension>::try_from(
                //         &tick_array_bitmap_extension_info,
                //     )
                //         .unwrap()
                //         .load()
                //         .unwrap()
                //         .deref(),
                // );
                let mut extension = TickArrayBitmapExtension::unpack_data(&tick_array_bitmap_extension_info.data.borrow().to_vec());

                /////// instead of using pool_flip_tick_array_bit_helper, flip manually
                let init_tick_array_start_index = vec![
                    -tick_spacing * TICK_ARRAY_SIZE * 7394, // The tickarray where min_tick(-443636) is located
                    tick_spacing * TICK_ARRAY_SIZE * 7393, // The tickarray where max_tick(443636) is located
                ];
                for start_index in init_tick_array_start_index {
                    extension.flip_tick_array_bit(start_index, pool_state.tick_spacing);
                }
                ///////

                let tick_array_bitmap_extension = Some(&extension);

                let start_index = pool_state
                    .next_initialized_tick_array_start_index(
                        &tick_array_bitmap_extension,
                        -tick_spacing * TICK_ARRAY_SIZE * 7394,
                        false,
                    )
                    .unwrap();
                assert!(start_index.unwrap() == tick_spacing * TICK_ARRAY_SIZE * 7393);
            }
        }
    }
}
