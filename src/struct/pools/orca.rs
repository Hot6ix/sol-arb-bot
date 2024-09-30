use std::any::Any;
use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;
use crate::account::account::{AccountDataSerializer, DeserializedAccount, DeserializedConfigAccount, DeserializedDataAccount, DeserializedTokenAccount};
use crate::r#struct::market::{Market, PoolOperation};
use crate::formula::base::Formula;
use crate::formula::base::Formula::ConcentratedLiquidity;
use crate::formula::orca_clmm::swap_internal;
use crate::utils::PubkeyPair;

#[derive(Copy, Clone, Debug, Default)]
pub struct OrcaClmmMarket {
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1], // 1
    pub tick_spacing: u16, // 2
    pub tick_spacing_seed: [u8; 2], // 2
    pub fee_rate: u16, // 2
    pub protocol_fee_rate: u16, // 2
    pub liquidity: u128, // 16
    pub sqrt_price: u128, // 16
    pub tick_current_index: i32, // 4
    pub protocol_fee_owed_a: u64, // 8
    pub protocol_fee_owed_b: u64, // 8
    pub token_mint_a: Pubkey, // 32
    pub token_vault_a: Pubkey, // 32
    pub fee_growth_global_a: u128, // 16
    pub token_mint_b: Pubkey, // 32
    pub token_vault_b: Pubkey, // 32
    pub fee_growth_global_b: u128, // 16
    pub reward_last_updated_timestamp: u64, // 8
    pub reward_infos: [WhirlpoolRewardInfo; 3] // 128 * 3; 384
}

impl AccountDataSerializer for OrcaClmmMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 653]; // 653
        let (discriminator, whirlpools_config, whirlpool_bump, tick_spacing, tick_spacing_seed, fee_rate, protocol_fee_rate, liquidity, sqrt_price, tick_current_index, protocol_fee_owed_a, protocol_fee_owed_b, token_mint_a, token_vault_a, fee_growth_global_a, token_mint_b, token_vault_b, fee_growth_global_b, reward_last_updated_timestamp, reward_infos) =
            array_refs![src, 8, 32, 1, 2, 2, 2, 2, 16, 16, 4, 8, 8, 32, 32, 16, 32, 32, 16, 8, 384];

        OrcaClmmMarket {
            whirlpools_config: Pubkey::new_from_array(*whirlpools_config),
            whirlpool_bump: *whirlpool_bump,
            tick_spacing: u16::from_le_bytes(*tick_spacing),
            tick_spacing_seed: *tick_spacing_seed,
            fee_rate: u16::from_le_bytes(*fee_rate),
            protocol_fee_rate: u16::from_le_bytes(*protocol_fee_rate),
            liquidity: u128::from_le_bytes(*liquidity),
            sqrt_price: u128::from_le_bytes(*sqrt_price),
            tick_current_index: i32::from_le_bytes(*tick_current_index),
            protocol_fee_owed_a: u64::from_le_bytes(*protocol_fee_owed_a),
            protocol_fee_owed_b: u64::from_le_bytes(*protocol_fee_owed_b),
            token_mint_a: Pubkey::new_from_array(*token_mint_a),
            token_vault_a: Pubkey::new_from_array(*token_vault_a),
            fee_growth_global_a: u128::from_le_bytes(*fee_growth_global_a),
            token_mint_b: Pubkey::new_from_array(*token_mint_b),
            token_vault_b: Pubkey::new_from_array(*token_vault_b),
            fee_growth_global_b: u128::from_le_bytes(*fee_growth_global_b),
            reward_last_updated_timestamp: u64::from_le_bytes(*reward_last_updated_timestamp),
            reward_infos: WhirlpoolRewardInfo::unpack_data_set(*reward_infos)
        }
    }
}

impl PoolOperation for OrcaClmmMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_mint_a,
            pubkey_b: self.token_mint_b,
        }
    }

    fn get_pool_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_vault_a,
            pubkey_b: self.token_vault_b
        }
    }

    fn get_swap_related_pubkeys(&self) -> Vec<(DeserializedAccount, Pubkey)> {
        vec![
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_vault_a),
            (DeserializedAccount::TokenAccount(DeserializedTokenAccount::default()), self.token_vault_b),
            (DeserializedAccount::ConfigAccount(DeserializedConfigAccount::default()), self.whirlpools_config),
        ]
    }

    fn get_formula(&self) -> Formula {
        ConcentratedLiquidity
    }

    fn swap(&self, accounts: &Vec<DeserializedAccount>) {
        todo!()
        // swap_internal()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey, // 32
    pub vault: Pubkey, // 32
    pub authority: Pubkey, // 32
    pub emissions_per_second_x64: u128, // 16
    pub growth_global_x64: u128 // 16
}

impl AccountDataSerializer for WhirlpoolRewardInfo {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 128];
        let (mint, vault, authority, emissions_per_second_x64, growth_global_x64) =
            array_refs![src, 32, 32, 32, 16, 16];

        WhirlpoolRewardInfo {
            mint: Pubkey::new_from_array(*mint),
            vault: Pubkey::new_from_array(*vault),
            authority: Pubkey::new_from_array(*authority),
            emissions_per_second_x64: u128::from_le_bytes(*emissions_per_second_x64),
            growth_global_x64: u128::from_le_bytes(*growth_global_x64),
        }
    }
}

impl WhirlpoolRewardInfo {
    pub fn unpack_data_set(data: [u8; 384]) -> [WhirlpoolRewardInfo; 3] {
        let index = data.len() / 3;
        let (first, rest) = data.split_at_checked(index).unwrap();
        let (second, third) = rest.split_at_checked(index).unwrap();

        [
            Self::unpack_data(&Vec::from(first)),
            Self::unpack_data(&Vec::from(second)),
            Self::unpack_data(&Vec::from(third))
        ]
    }

    pub fn initialized(&self) -> bool {
        self.mint.ne(&Pubkey::default())
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WhirlpoolsConfigAccount {
    pub pubkey: Pubkey,
    pub config: WhirlpoolsConfig,
    pub market: Market,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WhirlpoolsConfig {
    pub fee_authority: Pubkey, // 32
    pub collect_protocol_fees_authority: Pubkey, // 32
    pub reward_emissions_super_authority: Pubkey, // 32
    pub default_protocol_fee_rate: u16, // 2
}

impl AccountDataSerializer for WhirlpoolsConfig {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 98];
        let (fee_authority, collect_protocol_fees_authority, reward_emissions_super_authority, default_protocol_fee_rate) =
            array_refs![src, 32, 32, 32, 2];

        WhirlpoolsConfig {
            fee_authority: Pubkey::new_from_array(*fee_authority),
            collect_protocol_fees_authority: Pubkey::new_from_array(*collect_protocol_fees_authority),
            reward_emissions_super_authority: Pubkey::new_from_array(*reward_emissions_super_authority),
            default_protocol_fee_rate: u16::from_le_bytes(*default_protocol_fee_rate),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum OrcaClmmAccount {
    WhirlpoolsConfig(WhirlpoolsConfigAccount)
}

impl OrcaClmmAccount {
    pub fn get_pubkey(&self) -> Pubkey {
        match self {
            OrcaClmmAccount::WhirlpoolsConfig(account) => {
                account.pubkey
            }
        }
    }

    pub fn get_market(&self) -> Market {
        match self {
            OrcaClmmAccount::WhirlpoolsConfig(account) => {
                account.market
            }
        }
    }
}

// todo
#[cfg(test)]
pub mod whirlpool_builder {
    use crate::formula::clmm::orca_swap_state::NUM_REWARDS;
    use super::{OrcaClmmMarket, WhirlpoolRewardInfo};

    #[derive(Default)]
    pub struct WhirlpoolBuilder {
        liquidity: u128,
        tick_spacing: u16,
        tick_current_index: i32,
        sqrt_price: u128,
        fee_rate: u16,
        protocol_fee_rate: u16,
        fee_growth_global_a: u128,
        fee_growth_global_b: u128,
        reward_last_updated_timestamp: u64,
        reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS],
    }

    impl WhirlpoolBuilder {
        pub fn new() -> Self {
            Self {
                reward_infos: [WhirlpoolRewardInfo::default(); NUM_REWARDS],
                ..Default::default()
            }
        }

        pub fn liquidity(mut self, liquidity: u128) -> Self {
            self.liquidity = liquidity;
            self
        }

        pub fn reward_last_updated_timestamp(mut self, reward_last_updated_timestamp: u64) -> Self {
            self.reward_last_updated_timestamp = reward_last_updated_timestamp;
            self
        }

        pub fn reward_info(mut self, index: usize, reward_info: WhirlpoolRewardInfo) -> Self {
            self.reward_infos[index] = reward_info;
            self
        }

        pub fn reward_infos(mut self, reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS]) -> Self {
            self.reward_infos = reward_infos;
            self
        }

        pub fn tick_spacing(mut self, tick_spacing: u16) -> Self {
            self.tick_spacing = tick_spacing;
            self
        }

        pub fn tick_current_index(mut self, tick_current_index: i32) -> Self {
            self.tick_current_index = tick_current_index;
            self
        }

        pub fn sqrt_price(mut self, sqrt_price: u128) -> Self {
            self.sqrt_price = sqrt_price;
            self
        }

        pub fn fee_growth_global_a(mut self, fee_growth_global_a: u128) -> Self {
            self.fee_growth_global_a = fee_growth_global_a;
            self
        }

        pub fn fee_growth_global_b(mut self, fee_growth_global_b: u128) -> Self {
            self.fee_growth_global_b = fee_growth_global_b;
            self
        }

        pub fn fee_rate(mut self, fee_rate: u16) -> Self {
            self.fee_rate = fee_rate;
            self
        }

        pub fn protocol_fee_rate(mut self, protocol_fee_rate: u16) -> Self {
            self.protocol_fee_rate = protocol_fee_rate;
            self
        }

        pub fn build(self) -> OrcaClmmMarket {
            OrcaClmmMarket {
                liquidity: self.liquidity,
                reward_last_updated_timestamp: self.reward_last_updated_timestamp,
                reward_infos: self.reward_infos,
                tick_current_index: self.tick_current_index,
                sqrt_price: self.sqrt_price,
                tick_spacing: self.tick_spacing,
                fee_growth_global_a: self.fee_growth_global_a,
                fee_growth_global_b: self.fee_growth_global_b,
                fee_rate: self.fee_rate,
                protocol_fee_rate: self.protocol_fee_rate,
                ..Default::default()
            }
        }
    }
}