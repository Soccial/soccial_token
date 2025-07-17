// ===========================================================================
// Market Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles all SCTK **transfer mechanics involving the liquidity and user layer**,  
// including token purchases, peer-to-peer transfers, and fee distribution logic.
//
// ---------------------------------------------------------------------------
// ## Core Features:
// - Token purchase from liquidity and offchain reserve vaults
// - On-chain user-to-user transfers (e.g. tips, donations)
// - Dynamic fee calculation and automated distribution to vaults
//
// ---------------------------------------------------------------------------
// ## Fee Logic:
// - All transfers may include a fee in BPS (basis points)
// - Fees are split between rewards, airdrop, and revenue vaults
// - Vault distribution uses `TokenState.fee` config
//
// ---------------------------------------------------------------------------
// ## Components:
// - `buy_tokens()`: Mints tokens into user wallet after fiat purchase
// - `deposit_tokens()`: Transfers tokens from offchain vault with optional fee
// - `transfer_tokens()`: Allows direct user-to-user SCTK transfers with fee
// - `distribute_fees()`: Handles vault routing of fees
// - `calculate_fee()`: Computes net and fee portions using overflow-safe math
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::{
    economy::fee::{FEE_BPS_BASE, MAX_FEE_BPS}, market::{context::*, error::MarketError}, utils::math::format_sctk, vaults::{error::VaultError, resolve_vault_seeds, VaultAction}
};

#[event]
pub struct TokensPurchased {
    pub buyer: Pubkey,
    pub amount: u64,
    pub net_received: u64,
    pub fee_charged: u64,
    pub to_rewards: u64,
    pub to_airdrop: u64,
    pub to_revenue: u64,
}

#[event]
pub struct TokensWithdrawn {
    pub recipient: Pubkey,
    pub amount: u64,
    pub net_received: u64,
    pub fee_charged: u64,
    pub to_rewards: u64,
    pub to_airdrop: u64,
    pub to_revenue: u64,
}

#[event]
pub struct TokensTransferred {
    pub sender: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub net_received: u64,
    pub fee_charged: u64,
    pub to_rewards: u64,
    pub to_airdrop: u64,
    pub to_revenue: u64,
}


/// ===========================================================================
/// Purchases tokens from the liquidity vault and transfers them to the buyer,
/// deducting a fee (in BPS) and distributing it across vaults.
///
/// ## Source:
/// - Liquidity Vault (PDA)
///
/// ## Accounts:
/// - buyer_token_account: Destination for net amount
/// - liquidity_vault_token_account: Source of funds
/// - rewards/airdrop/revenue vaults: Fee receivers
///
/// ## Fee Behavior:
/// - Calculated using `calculate_fee`
/// - Distributed using `distribute_fees`
///
/// ## Errors:
/// - `VaultError::InsufficientVaultBalance` if vault has insufficient funds
/// - `MarketError::FeeTooHigh` if fee exceeds limit
/// ===========================================================================
pub(crate) fn buy_tokens(
    ctx: Context<BuyTokensContext>, 
    amount: u64, 
    fee_bps: u16,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidVaultAmount);
    require!(fee_bps <= MAX_FEE_BPS, MarketError::FeeTooHigh);
    require!(
        ctx.accounts.liquidity_vault_token_account.amount >= amount,
        VaultError::InsufficientVaultBalance
    );

    // Calculate net and fee portions
    let (net_amount, fee_amount) = calculate_fee(amount, fee_bps)?;

    // Prepare signer seeds for PDA authority
    let signer_seeds: &[&[u8]] = &[b"liquidity_vault", &[ctx.bumps.liquidity_vault]];
    let signer_seeds_nested = &[signer_seeds];

    // Transfer net amount to buyer
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.liquidity_vault.to_account_info(),
        },
        signer_seeds_nested,
    );
    token::transfer(transfer_ctx, net_amount)?;

    // Distribute fee to vaults
    let (to_rewards, to_airdrop, to_revenue) = distribute_fees(
        &FeeDistributionContext {
            token_state: ctx.accounts.token_state.clone(),
            token_program: ctx.accounts.token_program.to_account_info(),
            source_token_account: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            rewards_vault_token_account: ctx.accounts.rewards_vault_token_account.to_account_info(),
            airdrop_vault_token_account: ctx.accounts.airdrop_vault_token_account.to_account_info(),
            revenue_vault_token_account: ctx.accounts.revenue_vault_token_account.to_account_info(),
            authority: ctx.accounts.liquidity_vault.to_account_info(),
        },
        fee_amount,
        Some(signer_seeds_nested),
    )?;

    // Log transaction
    msg!(
        "🛒 User purchased {} SCTK ({} units) from liquidity vault → {} | 📈 Fee: {} SCTK ({} units → {} to revenue, {} to rewards, {} to airdrop)",
        format_sctk(net_amount),
        net_amount,
        ctx.accounts.buyer_token_account.key(),
        format_sctk(fee_amount),
        fee_amount,
        to_revenue,
        to_rewards,
        to_airdrop,
    );

    emit!(TokensPurchased {
        buyer: ctx.accounts.buyer_token_account.owner,
        amount,
        net_received: net_amount,
        fee_charged: fee_amount,
        to_rewards,
        to_airdrop,
        to_revenue,
    });


    Ok(())
}


/// ===========================================================================
/// Withdraws tokens from the offchain reserve vault into a user wallet,
/// with optional fee logic and vault-based fee distribution.
///
/// ## Source:
/// - Offchain Reserve Vault (PDA)
///
/// ## Behavior:
/// - Checks vault balance
/// - Applies fee via `calculate_fee`
/// - Sends net to user, routes fee to vaults
///
/// ## Use Case:
/// - Fiat ramp bridge to SPL token via custodial backend
///
/// ## Errors:
/// - `VaultError::InsufficientVaultBalance`
/// - `MarketError::*` for fee or amount issues
/// ===========================================================================
pub(crate) fn deposit_tokens(
    ctx: Context<DepositTokensContext>,
    amount: u64,
    fee_bps: u16,
) -> Result<()> {
    require!(amount > 0, MarketError::InvalidAmount);
    require!(fee_bps <= MAX_FEE_BPS, MarketError::FeeTooHigh);

   

    require!(
        ctx.accounts.offchain_reserve_vault_token_account.amount >= amount,
        VaultError::InsufficientVaultBalance
    );

    let (net_amount, fee_amount) = calculate_fee(amount, fee_bps)?;

    let (seed, bump) =
        resolve_vault_seeds(&ctx.accounts.offchain_reserve_vault, VaultAction::Operation)?;
    let signer_seeds: &[&[u8]] = &[seed, &[bump]];
    let signer_seeds_nested = &[signer_seeds];

    // Transfer net amount to user
    let transfer_to_user = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.offchain_reserve_vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.offchain_reserve_vault.to_account_info(),
        },
        signer_seeds_nested,
    );
    
    token::transfer(transfer_to_user, net_amount)?;

   
    // Distribute fee to vaults
    let (to_rewards, to_airdrop, to_revenue) = distribute_fees(
        &FeeDistributionContext {
            token_state: ctx.accounts.token_state.clone(),
            token_program: ctx.accounts.token_program.to_account_info(),
            source_token_account: ctx.accounts.offchain_reserve_vault_token_account.to_account_info(),
            rewards_vault_token_account: ctx.accounts.rewards_vault_token_account.to_account_info(),
            airdrop_vault_token_account: ctx.accounts.airdrop_vault_token_account.to_account_info(),
            revenue_vault_token_account: ctx.accounts.revenue_vault_token_account.to_account_info(),
            authority: ctx.accounts.offchain_reserve_vault.to_account_info(),
        },
        fee_amount,
        Some(signer_seeds_nested),
    )?;

   msg!(
        "💸 User withdrew {} SCTK ({} units) from offchain_reserve_vault to SPL wallet: {} | 📈 Fee: {} SCTK ({} units → {} to revenue, {} to rewards, {} to airdrop)",
        format_sctk(net_amount),
        net_amount,
        ctx.accounts.destination_token_account.key(),
        format_sctk(fee_amount),
        fee_amount,
        to_revenue,
        to_rewards,
        to_airdrop,
    );

    emit!(TokensWithdrawn {
        recipient: ctx.accounts.destination_token_account.owner,
        amount,
        net_received: net_amount,
        fee_charged: fee_amount,
        to_rewards,
        to_airdrop,
        to_revenue,
    });


    Ok(())
}


/// ===========================================================================
/// Transfers tokens from one user to another, deducting a fee (if any),
/// and routing it to the reward, airdrop, and revenue vaults.
///
/// ## Use Cases:
/// - Peer-to-peer tips and microtransactions
/// - Custom fee can be set by dApp layer
///
/// ## Fee Distribution:
/// - Sender pays fee
/// - Fee routed to vaults via `distribute_fees`
///
/// ## Security:
/// - Uses user's authority as signer
///
/// ## Errors:
/// - Invalid transfer amount or excessive fee
/// ===========================================================================
pub(crate) fn transfer_tokens(
    ctx: Context<TransferTokensContext>,
    amount: u64,
    fee_bps: u16,
) -> Result<()> {
    require!(amount > 0, MarketError::InvalidAmount);
    require!(fee_bps <= MAX_FEE_BPS, MarketError::FeeTooHigh);

    let (net_amount, fee_amount) = calculate_fee(amount, fee_bps)?;

    let cpi_ctx_net = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.sender_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        },
    );

    token::transfer(cpi_ctx_net, net_amount)?;

    let (to_rewards, to_airdrop, to_revenue) = distribute_fees(
        &FeeDistributionContext {
            token_state: ctx.accounts.token_state.clone(),
            token_program: ctx.accounts.token_program.to_account_info(),
            source_token_account: ctx.accounts.sender_token_account.to_account_info(),
            rewards_vault_token_account: ctx.accounts.rewards_vault_token_account.to_account_info(),
            airdrop_vault_token_account: ctx.accounts.airdrop_vault_token_account.to_account_info(),
            revenue_vault_token_account: ctx.accounts.revenue_vault_token_account.to_account_info(),
            authority: ctx.accounts.sender.to_account_info(),
        },
        fee_amount,
        None,
    )?;

    msg!(
        "💸 User transferred {} SCTK ({} units) to {} | 📈 Fee: {} SCTK ({} units → {} to revenue, {} to rewards, {} to airdrop)",
        format_sctk(net_amount),
        net_amount,
        ctx.accounts.recipient_token_account.key(),
        format_sctk(fee_amount),
        fee_amount,
        to_revenue,
        to_rewards,
        to_airdrop,
    );

    emit!(TokensTransferred {
        sender: ctx.accounts.sender.key(),
        recipient: ctx.accounts.recipient_token_account.owner,
        amount,
        net_received: net_amount,
        fee_charged: fee_amount,
        to_rewards,
        to_airdrop,
        to_revenue,
    });


    Ok(())
}


/// ===========================================================================
/// Distributes a total fee amount across vaults using configured weights,
/// performing CPI token transfers for each vault target.
///
/// ## Allocation:
/// - Rewards vault: Incentives, loyalty programs
/// - Airdrop vault: Community rewards
/// - Revenue vault: Platform income
///
/// ## Inputs:
/// - FeeDistributionContext: Includes vault accounts and signer
/// - total_fee: Amount to split
/// - signer_seeds: Optional PDA signer seeds
///
/// ## Behavior:
/// - Early return if fee == 0
/// - Transfers executed only for non-zero shares
///
/// ## Errors:
/// - Any transfer failure results in early exit
/// ===========================================================================
pub(crate) fn distribute_fees<'info>(
    ctx: &FeeDistributionContext<'info>,
    total_fee: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<(u64, u64, u64)> {
    if total_fee == 0 {
        return Ok((0, 0, 0));
    }

    let (to_rewards, to_airdrop, to_revenue) = ctx.token_state.fee.split_fee(total_fee)?;

    if to_rewards > 0 {
        let transfer_ctx = match signer_seeds {
            Some(seeds) => CpiContext::new_with_signer(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.rewards_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
                seeds,
            ),
            None => CpiContext::new(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.rewards_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
            ),
        };
        token::transfer(transfer_ctx, to_rewards)?;
    }

    if to_airdrop > 0 {
        let transfer_ctx = match signer_seeds {
            Some(seeds) => CpiContext::new_with_signer(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.airdrop_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
                seeds,
            ),
            None => CpiContext::new(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.airdrop_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
            ),
        };
        token::transfer(transfer_ctx, to_airdrop)?;
    }

    if to_revenue > 0 {
        let transfer_ctx = match signer_seeds {
            Some(seeds) => CpiContext::new_with_signer(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.revenue_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
                seeds,
            ),
            None => CpiContext::new(
                ctx.token_program.to_account_info(),
                Transfer {
                    from: ctx.source_token_account.to_account_info(),
                    to: ctx.revenue_vault_token_account.to_account_info(),
                    authority: ctx.authority.clone(),
                },
            ),
        };
        token::transfer(transfer_ctx, to_revenue)?;
    }

    Ok((to_rewards, to_airdrop, to_revenue))
}


/// ===========================================================================
/// Calculates the fee amount and net amount from a total using BPS (basis points),
/// applying overflow-safe arithmetic throughout.
///
/// ## Inputs:
/// - `amount`: Total token amount (in base units)
/// - `fee_bps`: Fee in basis points (1% = 100)
///
/// ## Returns:
/// - (net_amount, fee_amount)
///
/// ## Errors:
/// - `FeeTooHigh` if fee exceeds cap
/// - `Overflow` or `Underflow` for unsafe arithmetic
///
/// ## Example:

/// let (net, fee) = calculate_fee(1_000_000, 300)?; // 3% fee
/// assert_eq!(net, 970_000);
/// assert_eq!(fee, 30_000);

/// ===========================================================================
pub(crate) fn calculate_fee(
    amount: u64, 
    fee_bps: u16
) -> Result<(u64, u64)> {
    if fee_bps > MAX_FEE_BPS {
        return Err(MarketError::FeeTooHigh.into());
    }

    let fee = amount
        .checked_mul(fee_bps as u64)
        .ok_or(MarketError::Overflow)?
        .checked_div(FEE_BPS_BASE as u64)
        .ok_or(MarketError::Overflow)?;

    let net = amount.checked_sub(fee).ok_or(MarketError::Underflow)?;
    Ok((net, fee))
} 