use std::collections::HashMap;
use std::str::FromStr;
use anyhow::Context;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_sdk::pubkey::Pubkey;
use crate::constants::METEORA_DLMM_PROGRAM_PUBKEY;
use crate::formula::dlmm::bin::{Bin, BinArray, SwapResult};
use crate::formula::dlmm::bin_array_bitmap_extension::BinArrayBitmapExtension;
use crate::formula::dlmm::constant::BIN_ARRAY;
use crate::r#struct::pools::MeteoraDlmmMarket;

pub fn quote_exact_out(
    lb_pair_pubkey: Pubkey,
    lb_pair: &MeteoraDlmmMarket,
    mut amount_out: u64,
    swap_for_y: bool,
    bin_arrays: HashMap<Pubkey, BinArray>,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    current_timestamp: u64,
    current_slot: u64,
) -> Result<SwapExactOutQuote, &'static str> {
    validate_swap_activation(lb_pair, current_timestamp, current_slot)?;

    let mut lb_pair = *lb_pair;
    lb_pair.update_references(current_timestamp as i64)?;

    let mut total_amount_in: u64 = 0;
    let mut total_fee: u64 = 0;

    while amount_out > 0 {
        let active_bin_array_pubkey = get_bin_array_pubkeys_for_swap(
            lb_pair_pubkey,
            &lb_pair,
            bitmap_extension,
            swap_for_y,
            1,
        )?
            .pop()
            .context("Pool out of liquidity").expect("Pool out of liquidity");

        let mut active_bin_array = bin_arrays
            .get(&active_bin_array_pubkey)
            .cloned()
            .context("Active bin array not found").expect("Active bin array not found");

        loop {
            if active_bin_array
                .is_bin_id_within_range(lb_pair.active_id)
                .is_err()
                || amount_out == 0
            {
                break;
            }

            lb_pair.update_volatility_accumulator()?;

            let active_bin = active_bin_array.get_bin_mut(lb_pair.active_id)?;
            let price = active_bin.get_or_store_bin_price(lb_pair.active_id, lb_pair.bin_step)?;

            if !active_bin.is_empty(!swap_for_y) {
                let bin_max_amount_out = active_bin.get_max_amount_out(swap_for_y);
                if amount_out >= bin_max_amount_out {
                    let max_amount_in = active_bin.get_max_amount_in(price, swap_for_y)?;
                    let max_fee = lb_pair.compute_fee(max_amount_in)?;

                    total_amount_in = total_amount_in
                        .checked_add(max_amount_in)
                        .context("MathOverflow").expect("MathOverflow");

                    total_fee = total_fee.checked_add(max_fee).context("MathOverflow").expect("MathOverflow");

                    amount_out = amount_out
                        .checked_sub(bin_max_amount_out)
                        .context("MathOverflow").expect("MathOverflow");
                } else {
                    let amount_in = Bin::get_amount_in(amount_out, price, swap_for_y)?;
                    let fee = lb_pair.compute_fee(amount_in)?;

                    total_amount_in = total_amount_in
                        .checked_add(amount_in)
                        .context("MathOverflow").expect("MathOverflow");

                    total_fee = total_fee.checked_add(fee).context("MathOverflow").expect("MathOverflow");

                    amount_out = 0;
                }
            }

            if amount_out > 0 {
                lb_pair.advance_active_bin(swap_for_y)?;
            }
        }
    }

    Ok(SwapExactOutQuote {
        amount_in: total_amount_in,
        fee: total_fee,
    })
}

pub fn quote_exact_in(
    lb_pair_pubkey: Pubkey,
    lb_pair: &MeteoraDlmmMarket,
    mut amount_in: u64,
    swap_for_y: bool,
    bin_arrays: HashMap<Pubkey, BinArray>,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    current_timestamp: u64,
    current_slot: u64,
) -> Result<SwapExactInQuote, &'static str> {
    validate_swap_activation(lb_pair, current_timestamp, current_slot)?;

    let mut lb_pair = *lb_pair;
    lb_pair.update_references(current_timestamp as i64)?;

    let mut total_amount_out: u64 = 0;
    let mut total_fee: u64 = 0;

    while amount_in > 0 {
        let active_bin_array_pubkey = get_bin_array_pubkeys_for_swap(
            lb_pair_pubkey,
            &lb_pair,
            bitmap_extension,
            swap_for_y,
            1,
        )?
            .pop()
            .context("Pool out of liquidity").expect("Pool out of liquidity");

        let mut active_bin_array = bin_arrays
            .get(&active_bin_array_pubkey)
            .cloned()
            .context("Active bin array not found").expect("Active bin array not found");

        loop {
            if active_bin_array
                .is_bin_id_within_range(lb_pair.active_id)
                .is_err()
                || amount_in == 0
            {
                break;
            }

            lb_pair.update_volatility_accumulator()?;

            let active_bin = active_bin_array.get_bin_mut(lb_pair.active_id)?;
            let price = active_bin.get_or_store_bin_price(lb_pair.active_id, lb_pair.bin_step)?;

            if !active_bin.is_empty(!swap_for_y) {
                let SwapResult {
                    amount_in_with_fees,
                    amount_out,
                    fee,
                    ..
                } = active_bin.swap(amount_in, price, swap_for_y, &lb_pair, None)?;

                amount_in = amount_in
                    .checked_sub(amount_in_with_fees)
                    .context("MathOverflow").expect("MathOverflow");

                total_amount_out = total_amount_out
                    .checked_add(amount_out)
                    .context("MathOverflow").expect("MathOverflow");
                total_fee = total_fee.checked_add(fee).context("MathOverflow").expect("MathOverflow");
            }

            if amount_in > 0 {
                lb_pair.advance_active_bin(swap_for_y)?;
            }
        }
    }

    Ok(SwapExactInQuote {
        amount_out: total_amount_out,
        fee: total_fee,
    })
}

pub fn get_bin_array_pubkeys_for_swap(
    lb_pair_pubkey: Pubkey,
    lb_pair: &MeteoraDlmmMarket,
    bitmap_extension: Option<&BinArrayBitmapExtension>,
    swap_for_y: bool,
    take_count: u8,
) -> Result<Vec<Pubkey>, &'static str> {
    let mut start_bin_array_idx = BinArray::bin_id_to_bin_array_index(lb_pair.active_id)?;
    let mut bin_array_idx = vec![];
    let increment = if swap_for_y { -1 } else { 1 };

    loop {
        if bin_array_idx.len() == take_count as usize {
            break;
        }

        if lb_pair.is_overflow_default_bin_array_bitmap(start_bin_array_idx) {
            let Some(bitmap_extension) = bitmap_extension else {
                break;
            };
            let Ok((next_bin_array_idx, has_liquidity)) = bitmap_extension
                .next_bin_array_index_with_liquidity(swap_for_y, start_bin_array_idx)
            else {
                // Out of search range. No liquidity.
                break;
            };
            if has_liquidity {
                bin_array_idx.push(next_bin_array_idx);
                start_bin_array_idx = next_bin_array_idx + increment;
            } else {
                // Switch to internal bitmap
                start_bin_array_idx = next_bin_array_idx;
            }
        } else {
            let Ok((next_bin_array_idx, has_liquidity)) = lb_pair
                .next_bin_array_index_with_liquidity_internal(swap_for_y, start_bin_array_idx)
            else {
                break;
            };
            if has_liquidity {
                bin_array_idx.push(next_bin_array_idx);
                start_bin_array_idx = next_bin_array_idx + increment;
            } else {
                // Switch to external bitmap
                start_bin_array_idx = next_bin_array_idx;
            }
        }
    }

    let bin_array_pubkeys = bin_array_idx
        .into_iter()
        .map(|idx| derive_bin_array_pda(lb_pair_pubkey, idx.into()).0)
        .collect();

    Ok(bin_array_pubkeys)
}

fn validate_swap_activation(
    lb_pair: &MeteoraDlmmMarket,
    current_timestamp: u64,
    current_slot: u64,
) -> Result<(), &'static str> {
    assert!(lb_pair.status()?.eq(&PairStatus::Enabled));

    let pair_type = lb_pair.pair_type()?;
    if pair_type.eq(&PairType::Permission) {
        let activation_type = ActivationType::try_from(lb_pair.activation_type).expect("unknown activation_type");
        let current_point = match activation_type {
            ActivationType::Slot => current_slot,
            ActivationType::Timestamp => current_timestamp,
        };

        assert!(current_point >= lb_pair.activation_point)
    }

    Ok(())
}

pub fn derive_bin_array_pda(lb_pair: Pubkey, bin_array_index: i64) -> (Pubkey, u8) {
    let program_id = Pubkey::from_str(METEORA_DLMM_PROGRAM_PUBKEY).unwrap();
    Pubkey::find_program_address(
        &[BIN_ARRAY, lb_pair.as_ref(), &bin_array_index.to_le_bytes()],
        &program_id,
    )
}

#[derive(Debug)]
pub struct SwapExactOutQuote {
    pub amount_in: u64,
    pub fee: u64,
}

#[derive(Debug)]
pub struct SwapExactInQuote {
    pub amount_out: u64,
    pub fee: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the Pair. 0 = Permissionless, 1 = Permission. Putting 0 as permissionless for backward compatibility.
pub enum PairType {
    Permissionless,
    Permission,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}

#[derive(Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
/// Pair status. 0 = Enabled, 1 = Disabled. Putting 0 as enabled for backward compatibility.
pub enum PairStatus {
    // Fully enabled.
    // Condition:
    // Permissionless: PairStatus::Enabled
    // Permission: PairStatus::Enabled and current_point > activation_point
    Enabled,
    // Similar as emergency mode. User can only withdraw (Only outflow). Except whitelisted wallet still have full privileges.
    Disabled,
}