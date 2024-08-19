use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketOperation, MarketSerializer, PubkeyPair};

pub struct OrcaMarket {
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

impl MarketSerializer for OrcaMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 653]; // 653
        let (discriminator, whirlpools_config, whirlpool_bump, tick_spacing, tick_spacing_seed, fee_rate, protocol_fee_rate, liquidity, sqrt_price, tick_current_index, protocol_fee_owed_a, protocol_fee_owed_b, token_mint_a, token_vault_a, fee_growth_global_a, token_mint_b, token_vault_b, fee_growth_global_b, reward_last_updated_timestamp, reward_infos) =
            array_refs![src, 8, 32, 1, 2, 2, 2, 2, 16, 16, 4, 8, 8, 32, 32, 16, 32, 32, 16, 8, 384];

        OrcaMarket {
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

impl MarketOperation for OrcaMarket {
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
}

pub struct WhirlpoolRewardInfo {
    pub mint: Pubkey, // 32
    pub vault: Pubkey, // 32
    pub authority: Pubkey, // 32
    pub emissions_per_second_x64: u128, // 16
    pub growth_global_x64: u128 // 16
}

impl WhirlpoolRewardInfo {
    pub fn unpack_data(data: &Vec<u8>) -> WhirlpoolRewardInfo {
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
}