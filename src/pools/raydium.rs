use arrayref::{array_ref, array_refs};
use solana_sdk::pubkey::Pubkey;
use crate::pools::{MarketOperation, MarketSerializer, PubkeyPair};

pub struct RaydiumMarket {
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

impl MarketSerializer for RaydiumMarket {
    fn unpack_data(data: &Vec<u8>) -> Self {
        let src = array_ref![data, 0, 1544];
        let (discriminator, bump, amm_config, owner, token_mint_0, token_mint_1, token_vault_0, token_vault_1, observation_key, mint_decimals_0, mint_decimals_1, tick_spacing, liquidity, sqrt_price_x64, tick_current, padding3, padding4, fee_growth_global_0_x64, fee_growth_global_1_x64, protocol_fees_token_0, protocol_fees_token_1, swap_in_amount_token_0, swap_out_amount_token_1, swap_in_amount_token_1, swap_out_amount_token_0, status, padding, reward_infos, tick_array_bitmap, total_fees_token_0, total_fees_claimed_token_0, total_fees_token_1, total_fees_claimed_token_1, fund_fees_token_0, fund_fees_token_1, open_time, recent_epoch, padding1, padding2) =
            array_refs![src, 8, 1, 32, 32, 32, 32, 32, 32, 32, 1, 1, 2, 16, 16, 4, 2, 2, 16, 16, 8, 8, 16, 16, 16, 16, 1, 7, 507, 128, 8, 8, 8, 8, 8, 8, 8, 8, 192, 256];
        todo!()
    }
}

impl MarketOperation for RaydiumMarket {
    fn get_mint_pair(&self) -> PubkeyPair {
        PubkeyPair {
            pubkey_a: self.token_mint_0,
            pubkey_b: self.token_mint_1
        }
    }
}

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

impl RewardInfo {
    fn unpack_data(data: &Vec<u8>) -> Self {
        todo!()
    }

    fn unpack_data_set(data: [u8; 531]) -> [RewardInfo; 3] {
        todo!()
    }
}