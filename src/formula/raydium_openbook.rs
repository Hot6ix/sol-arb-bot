use solana_sdk::pubkey::Pubkey;
use crate::formula::clmm::u256_math::U128;
use crate::formula::openbook::math::Calculator;
use crate::formula::openbook::openbook_processor::{LeafNode, SwapDirection};

pub fn process_swap_base_in(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    swap: SwapInstructionBaseIn,
) -> Result<(), &'static str> {
    const ACCOUNT_LEN: usize = 17;
    let input_account_len = accounts.len();
    if input_account_len != ACCOUNT_LEN && input_account_len != ACCOUNT_LEN + 1 {
        return Err("AmmError::WrongAccountsNumber");
    }
    let account_info_iter = &mut accounts.iter();
    let token_program_info = next_account_info(account_info_iter)?;

    let amm_info = next_account_info(account_info_iter)?;
    let amm_authority_info = next_account_info(account_info_iter)?;
    let amm_open_orders_info = next_account_info(account_info_iter)?;
    if input_account_len == ACCOUNT_LEN + 1 {
        let _amm_target_orders_info = next_account_info(account_info_iter)?;
    }
    let amm_coin_vault_info = next_account_info(account_info_iter)?;
    let amm_pc_vault_info = next_account_info(account_info_iter)?;

    let market_program_info = next_account_info(account_info_iter)?;

    let mut amm = AmmInfo::load_mut_checked(&amm_info, program_id)?;
    let enable_orderbook;
    if AmmStatus::from_u64(amm.status).orderbook_permission() {
        enable_orderbook = true;
    } else {
        enable_orderbook = false;
    }
    let market_info = next_account_info(account_info_iter)?;
    let market_bids_info = next_account_info(account_info_iter)?;
    let market_asks_info = next_account_info(account_info_iter)?;
    let market_event_queue_info = next_account_info(account_info_iter)?;
    let market_coin_vault_info = next_account_info(account_info_iter)?;
    let market_pc_vault_info = next_account_info(account_info_iter)?;
    let market_vault_signer = next_account_info(account_info_iter)?;

    let user_source_info = next_account_info(account_info_iter)?;
    let user_destination_info = next_account_info(account_info_iter)?;
    let user_source_owner = next_account_info(account_info_iter)?;
    if !user_source_owner.is_signer {
        return Err(AmmError::InvalidSignAccount.into());
    }
    check_assert_eq!(
            *token_program_info.key,
            spl_token::id(),
            "spl_token_program",
            AmmError::InvalidSplTokenProgram
        );
    let spl_token_program_id = token_program_info.key;
    if *amm_authority_info.key
        != Self::authority_id(program_id, AUTHORITY_AMM, amm.nonce as u8)?
    {
        return Err("AmmError::InvalidProgramAddress");
    }
    check_assert_eq!(
            *amm_coin_vault_info.key,
            amm.coin_vault,
            "coin_vault",
            AmmError::InvalidCoinVault
        );
    check_assert_eq!(
            *amm_pc_vault_info.key,
            amm.pc_vault,
            "pc_vault",
            AmmError::InvalidPCVault
        );

    if *user_source_info.key == amm.pc_vault || *user_source_info.key == amm.coin_vault {
        return Err("AmmError::InvalidUserToken");
    }
    if *user_destination_info.key == amm.pc_vault
        || *user_destination_info.key == amm.coin_vault
    {
        return Err("AmmError::InvalidUserToken");
    }

    let amm_coin_vault =
        Self::unpack_token_account(&amm_coin_vault_info, spl_token_program_id)?;
    let amm_pc_vault = Self::unpack_token_account(&amm_pc_vault_info, spl_token_program_id)?;

    let user_source = Self::unpack_token_account(&user_source_info, spl_token_program_id)?;
    let user_destination =
        Self::unpack_token_account(&user_destination_info, spl_token_program_id)?;

    if !AmmStatus::from_u64(amm.status).swap_permission() {
        msg!(&format!("swap_base_in: status {}", amm.status));
        let clock = Clock::get()?;
        if amm.status == AmmStatus::OrderBookOnly.into_u64()
            && (clock.unix_timestamp as u64) >= amm.state_data.orderbook_to_init_time
        {
            amm.status = AmmStatus::Initialized.into_u64();
            msg!("swap_base_in: OrderBook to Initialized");
        } else {
            return Err("AmmError::InvalidStatus");
        }
    } else if amm.status == AmmStatus::WaitingTrade.into_u64() {
        let clock = Clock::get()?;
        if (clock.unix_timestamp as u64) < amm.state_data.pool_open_time {
            return Err("AmmError::InvalidStatus");
        } else {
            amm.status = AmmStatus::SwapOnly.into_u64();
            println!("swap_base_in: WaitingTrade to SwapOnly");
        }
    }

    let total_pc_without_take_pnl;
    let total_coin_without_take_pnl;
    let mut bids: Vec<LeafNode> = Vec::new();
    let mut asks: Vec<LeafNode> = Vec::new();
    if enable_orderbook {
        check_assert_eq!(
                *amm_open_orders_info.key,
                amm.open_orders,
                "open_orders",
                AmmError::InvalidOpenOrders
            );
        check_assert_eq!(
                *market_program_info.key,
                amm.market_program,
                "market_program",
                AmmError::InvalidMarketProgram
            );
        check_assert_eq!(
                *market_info.key,
                amm.market,
                "market",
                AmmError::InvalidMarket
            );
        let (market_state, open_orders) = Processor::load_serum_market_order(
            market_info,
            amm_open_orders_info,
            amm_authority_info,
            &amm,
            false,
        )?;
        let bids_orders = market_state.load_bids_mut(&market_bids_info)?;
        let asks_orders = market_state.load_asks_mut(&market_asks_info)?;
        (bids, asks) = Self::get_amm_orders(&open_orders, bids_orders, asks_orders)?;
        (total_pc_without_take_pnl, total_coin_without_take_pnl) =
            Calculator::calc_total_without_take_pnl(
                amm_pc_vault.amount,
                amm_coin_vault.amount,
                &open_orders,
                &amm,
                &market_state,
                &market_event_queue_info,
                &amm_open_orders_info,
            )?;
    } else {
        (total_pc_without_take_pnl, total_coin_without_take_pnl) =
            Calculator::calc_total_without_take_pnl_no_orderbook(
                amm_pc_vault.amount,
                amm_coin_vault.amount,
                &amm,
            )?;
    }

    let swap_direction;
    if user_source.mint == amm_coin_vault.mint && user_destination.mint == amm_pc_vault.mint {
        swap_direction = SwapDirection::Coin2PC
    } else if user_source.mint == amm_pc_vault.mint
        && user_destination.mint == amm_coin_vault.mint
    {
        swap_direction = SwapDirection::PC2Coin
    } else {
        return Err("AmmError::InvalidUserToken");
    }
    if user_source.amount < swap.amount_in {
        // encode_ray_log(SwapBaseInLog {
        //     log_type: LogType::SwapBaseIn.into_u8(),
        //     amount_in: swap.amount_in,
        //     minimum_out: swap.minimum_amount_out,
        //     direction: swap_direction as u64,
        //     user_source: user_source.amount,
        //     pool_coin: total_coin_without_take_pnl,
        //     pool_pc: total_pc_without_take_pnl,
        //     out_amount: 0,
        // });
        return Err("AmmError::InsufficientFunds");
    }
    let swap_fee = U128::from(swap.amount_in)
        .checked_mul(amm.fees.swap_fee_numerator.into())
        .unwrap()
        .checked_ceil_div(amm.fees.swap_fee_denominator.into())
        .unwrap()
        .0;
    let swap_in_after_deduct_fee = U128::from(swap.amount_in).checked_sub(swap_fee).unwrap();
    let swap_amount_out = Calculator::swap_token_amount_base_in(
        swap_in_after_deduct_fee,
        total_pc_without_take_pnl.into(),
        total_coin_without_take_pnl.into(),
        swap_direction,
    )
        .as_u64();
    // encode_ray_log(SwapBaseInLog {
    //     log_type: LogType::SwapBaseIn.into_u8(),
    //     amount_in: swap.amount_in,
    //     minimum_out: swap.minimum_amount_out,
    //     direction: swap_direction as u64,
    //     user_source: user_source.amount,
    //     pool_coin: total_coin_without_take_pnl,
    //     pool_pc: total_pc_without_take_pnl,
    //     out_amount: swap_amount_out,
    // });
    if swap_amount_out < swap.minimum_amount_out {
        return Err("AmmError::ExceededSlippage");
    }
    if swap_amount_out == 0 || swap.amount_in == 0 {
        return Err("AmmError::InvalidInput");
    }
}