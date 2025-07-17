// ===========================================================================
// Staking Logic Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module implements the **core staking logic** for the Soccial Token,
// allowing users to lock tokens under predefined reward plans and earn
// APR-based incentives over renewable cycles.
//
// ---------------------------------------------------------------------------
// ## Core Capabilities:
// - Stake tokens from user wallet or liquidity vault (Buy & Stake)
// - Auto-renewed staking cycles with per-cycle reward claims
// - Reinforcement logic to top-up existing stakes
// - Reward calculation based on APR and lockup duration
//
// ---------------------------------------------------------------------------
// ## System Design:
// - Vault-based architecture for secure token flow
// - Staking entries tracked individually via `StakingAccount` PDAs
// - APR stored in basis points (bps), where 100 bps = 1% APR
// - Cycles are reset after each claim or reinforcement
//
// ---------------------------------------------------------------------------
// ## Supported Instructions:
// - `stake_tokens`: Stake tokens from the user wallet
// - `buy_and_stake_tokens`: Stake tokens directly from liquidity vault
// - `add_to_stake`: Reinforce an existing stake with more tokens
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{Transfer, transfer};

use crate::staking::{context::*, StakingErrorCode};
use solana_program::sysvar::clock::Clock;

#[event]
pub struct StakeReinforced {
    pub participant: Pubkey,
    pub added_tokens: u64,
    pub compounded_rewards: u64,
    pub new_reserved_rewards: u64,
    pub total_transferred: u64,
    pub timestamp: i64,
}

#[event]
pub struct BuyAndStakeEvent {
    pub participant: Pubkey,
    pub plan_id: u8,
    pub staked_tokens: u64,
    pub reserved_rewards: u64,
    pub total_transferred: u64,
    pub timestamp: i64,
}

#[event]
pub struct StakeTokensEvent {
    pub participant: Pubkey,
    pub plan_id: u8,
    pub staked_tokens: u64,
    pub reserved_rewards: u64,
    pub total_transferred: u64,
    pub timestamp: i64,
}


/// ===========================================================================
/// Stakes Tokens via Liquidity Vault Purchase (Buy & Stake)
///
/// This function is used when a user acquires tokens (e.g. via fiat or reward)
/// and wants to **stake them immediately**. It pulls tokens directly from the
/// liquidity vault without involving the user's token account.
///
/// ## Behavior:
/// - Validates the selected staking plan
/// - Estimates the reward based on APR
/// - Transfers `amount + reward_estimate` from the `liquidity_vault` to `staking_vault`
/// - Registers the staking metadata under a new `StakingAccount`
///
/// ## Security:
/// - Fails if the liquidity vault has insufficient balance
/// - Does **not** involve user signature – ideal for programmatic staking flows
///
/// ## Parameters:
/// - `ctx`: Anchor context with liquidity and staking vaults
/// - `amount`: Tokens to be staked
/// - `plan_id`: Identifier of the staking plan
///
/// ## Errors:
/// - `InvalidStakingPlan` if plan ID is unknown
/// - `Overflow` if math fails
/// - `InsufficientVaultBalance` if vault can't cover stake + reward
/// ===========================================================================
/// ===========================================================================
/// Stakes Tokens via Liquidity Vault Purchase (Buy & Stake)
///
/// Used when a user receives tokens (e.g. from fiat or referral) and wants to
/// stake them immediately. Pulls tokens from the liquidity vault directly.
///
/// ## Behavior:
/// - Validates plan
/// - Calculates reward estimate
/// - Transfers `amount + estimated_reward` from liquidity → staking vault
/// - Registers new `StakingAccount` with full tracking for cyclic staking
///
/// ## Notes:
/// - Stake will auto-renew after each lockup if not withdrawn
/// ===========================================================================
pub fn buy_and_stake_tokens(
    ctx: Context<BuyAndStakeTokens>,
    amount: u64,
    plan_id: u8,
) -> Result<()> {
    let clock = Clock::get()?;
    let staking_state = &mut ctx.accounts.staking_state;
    let staking_account = &mut ctx.accounts.staking_account;

    // Step 1: Validate plan
    let plan = staking_state
        .get_plan(plan_id)
        .ok_or(StakingErrorCode::InvalidStakingPlan)?;

    // Step 2: Calculate reward and total required
    let apr_bps = plan.apr_bps as u64;
    let reward_estimate = amount
        .checked_mul(apr_bps).ok_or(StakingErrorCode::Overflow)?
        .checked_div(10_000).ok_or(StakingErrorCode::Overflow)?;

    let total_required = amount
        .checked_add(reward_estimate).ok_or(StakingErrorCode::Overflow)?;

    // Step 3: Ensure liquidity vault has enough
    require!(
        ctx.accounts.liquidity_vault_token_account.amount >= total_required,
        StakingErrorCode::InsufficientVaultBalance
    );

    // Step 4: Transfer stake + reward reserve from liquidity vault to staking vault
    let signer_seeds: &[&[u8]] = &[b"liquidity_vault", &[ctx.bumps.liquidity_vault]];
    let signer = &[signer_seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            to: ctx.accounts.staking_vault_token_account.to_account_info(),
            authority: ctx.accounts.liquidity_vault.to_account_info(),
        },
        signer,
    );

    transfer(cpi_ctx, total_required)?;

    // Step 5: Register staking metadata (new format)
    staking_account.participant = ctx.accounts.participant.key();
    staking_account.stake_id = staking_state.last_id;
    staking_account.start_time = clock.unix_timestamp;
    staking_account.lockup_duration = plan.lockup_duration;
    staking_account.apr_bps = plan.apr_bps;
    staking_account.staked_tokens = amount;
    staking_account.plan_id = plan_id;
    staking_account.total_rewards_claimed = 0;
    staking_account.cycles_completed = 0;

    staking_state.last_id += 1;

    // Step 6: Log
    msg!(
        "✅ Buy & Stake complete → User: {} | Plan: {} | Staked: {} tokens | Reserved Rewards: {} tokens | Total transferred: {} tokens (from liquidity vault → staking vault)",
        staking_account.participant,
        plan_id,
        amount,
        reward_estimate,
        total_required
    );

    emit!(BuyAndStakeEvent {
        participant: staking_account.participant,
        plan_id,
        staked_tokens: amount,
        reserved_rewards: reward_estimate,
        total_transferred: total_required,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// ===========================================================================
/// Stakes Tokens from User’s Wallet (Standard Stake Flow)
///
/// Used when a user wants to stake tokens from their own wallet (ATA).
/// Transfers the tokens to the `staking_vault` and reserves the reward
/// from the liquidity vault, setting up a renewable staking entry.
///
/// ## Behavior:
/// - Validates user's token balance
/// - Estimates reward based on plan APR
/// - Transfers stake from user to vault
/// - Reserves reward from liquidity vault
/// - Registers staking metadata (with support for renewal cycles)
///
/// ## Parameters:
/// - `ctx`: Anchor context including user wallet and vaults
/// - `amount`: Amount to stake
/// - `plan_id`: Identifier of the staking plan
///
/// ## Errors:
/// - `InvalidStakingPlan`, `InsufficientUserBalance`
/// - `Overflow`, `InsufficientVaultBalance`
/// ===========================================================================
pub fn stake_tokens(
    ctx: &mut Context<StakeTokens>, 
    amount: u64,
    plan_id: u8
) -> Result<()> {
    let clock = Clock::get()?;
    let staking_state = &mut ctx.accounts.staking_state;
    let staking_account = &mut ctx.accounts.staking_account;

    // Step 1: Validate staking plan
    let plan = staking_state
        .get_plan(plan_id)
        .ok_or(StakingErrorCode::InvalidStakingPlan)?;

    // Step 2: Validate user's balance
    let user_balance = ctx.accounts.participant_token_account.amount;
    require!(user_balance >= amount, StakingErrorCode::InsufficientUserBalance);

    // Step 3: Calculate estimated reward
    let reward_estimate = amount
        .checked_mul(plan.apr_bps as u64).ok_or(StakingErrorCode::Overflow)?
        .checked_div(10_000).ok_or(StakingErrorCode::Overflow)?;

    // Step 4: Validate liquidity vault has reward funds
    let liquidity_balance = ctx.accounts.liquidity_vault_token_account.amount;
    require!(
        liquidity_balance >= reward_estimate,
        StakingErrorCode::InsufficientVaultBalance
    );

    // Step 5: Reserve reward from liquidity vault
    let seeds: &[&[u8]] = &[b"liquidity_vault", &[ctx.bumps.liquidity_vault]];
    let signer = &[seeds];

    let reward_transfer = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            to: ctx.accounts.staking_vault_token_account.to_account_info(),
            authority: ctx.accounts.liquidity_vault.to_account_info(),
        },
        signer,
    );

    transfer(reward_transfer, reward_estimate)?;
    msg!(
        "📥 Reserved {} tokens from liquidity vault to staking vault (future rewards for participant {} in plan {}).",
        reward_estimate,
        ctx.accounts.participant.key(),
        plan_id
    );

    // Step 6: Transfer stake from user wallet to staking vault
    let user_to_staking = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.participant_token_account.to_account_info(),
            to: ctx.accounts.staking_vault_token_account.to_account_info(),
            authority: ctx.accounts.participant.to_account_info(),
        },
    );
    transfer(user_to_staking, amount)?;

    // Step 7: Register staking metadata
    staking_account.participant = ctx.accounts.participant.key();
    staking_account.stake_id = staking_state.last_id;
    staking_account.start_time = clock.unix_timestamp;
    staking_account.lockup_duration = plan.lockup_duration;
    staking_account.apr_bps = plan.apr_bps;
    staking_account.staked_tokens = amount;
    staking_account.plan_id = plan_id;
    staking_account.cycles_completed = 0;
    staking_account.total_rewards_claimed = 0;

    staking_state.last_id += 1;

    let total_transferred = amount + reward_estimate;

    msg!(
        "✅ Stake complete → User: {} | Plan: {} | Staked: {} tokens | Reserved Rewards: {} tokens | Total transferred: {} tokens (from user wallet + liquidity vault → staking vault)",
        staking_account.participant,
        plan_id,
        amount,
        reward_estimate,
        total_transferred
    );

    emit!(StakeTokensEvent {
        participant: staking_account.participant,
        plan_id,
        staked_tokens: amount,
        reserved_rewards: reward_estimate,
        total_transferred,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}


/// ===========================================================================
/// Reinforces or Refreshes an Existing Stake (Add to Stake / Restart Cycle)
///
/// This function supports two primary use cases:
///
/// 1. **Stake Reinforcement (amount > 0):**
///    - The participant adds more tokens to an existing stake.
///    - The total staked amount increases.
///    - The `start_time` is reset to begin a new reward cycle.
///    - Rewards are recalculated based on the updated stake.
///    - If the previous lockup period has ended, pending rewards are
///      automatically compounded (added to stake) before reinforcement.
///
/// 2. **Cycle Refresh Only (amount == 0):**
///    - No additional tokens are added.
///    - If the previous cycle has completed, pending rewards are auto-compounded.
///    - The cycle is restarted (`start_time` is updated).
///    - Additional rewards are reserved based on the updated staked amount.
///
/// ## Notes:
/// - Pending rewards are always applied first before any further calculations.
/// - Only the **additional** reward (based on the new amount or updated stake)
///   is pulled from the liquidity vault.
/// - `total_rewards_claimed` and `cycles_completed` are updated accordingly.
///
/// ## Parameters:
/// - `ctx`: Anchor context including participant, vaults, token accounts, etc.
/// - `amount`: Number of additional tokens to stake, or 0 to only refresh cycle.
///
/// ## Examples:
/// - `add_to_stake(ctx, 5000)` → adds 5000 tokens and restarts the cycle.
/// - `add_to_stake(ctx, 0)` → restarts the cycle using current stake + pending rewards.
///
/// ## Errors:
/// - `StakingPeriodNotOver` if the lockup hasn’t ended but compound is attempted
/// - `InsufficientUserBalance` if participant lacks funds
/// - `InsufficientVaultBalance` if the vault cannot cover rewards
/// - `Overflow`, `RewardOverflow` if arithmetic overflows
/// ===========================================================================
pub fn add_to_stake(
    ctx: &mut Context<ReinforceStake>,
    amount: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    let staking_account = &mut ctx.accounts.staking_account;

    let mut old_rewards_compounded = 0;

    // Step 0: Auto-compound if previous cycle ended
    let lockup_end = staking_account.start_time + staking_account.lockup_duration;
    if clock.unix_timestamp >= lockup_end {
        let apr = staking_account.apr_bps as u128;
        let base = staking_account.staked_tokens as u128;

        let pending_reward = base
            .checked_mul(apr)
            .ok_or(StakingErrorCode::RewardOverflow)?
            .checked_div(10_000)
            .ok_or(StakingErrorCode::RewardOverflow)? as u64;

        require!(
            ctx.accounts.staking_vault_token_account.amount >= pending_reward,
            StakingErrorCode::InsufficientVaultBalance
        );

        staking_account.staked_tokens = staking_account
            .staked_tokens
            .checked_add(pending_reward)
            .ok_or(StakingErrorCode::Overflow)?;

        staking_account.total_rewards_claimed = staking_account
            .total_rewards_claimed
            .checked_add(pending_reward)
            .ok_or(StakingErrorCode::Overflow)?;

        staking_account.cycles_completed += 1;

        old_rewards_compounded = pending_reward;
    }

    // Step 1: Transfer user tokens if any
    if amount > 0 {
        let user_balance = ctx.accounts.participant_token_account.amount;
        require!(
            user_balance >= amount,
            StakingErrorCode::InsufficientUserBalance
        );

        let transfer_user = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.participant_token_account.to_account_info(),
                to: ctx.accounts.staking_vault_token_account.to_account_info(),
                authority: ctx.accounts.participant.to_account_info(),
            },
        );
        transfer(transfer_user, amount)?;
    }

    // Step 2: Calculate new reward based on full updated stake
    let apr = staking_account.apr_bps as u64;
    let updated_stake = staking_account
        .staked_tokens
        .checked_add(amount)
        .ok_or(StakingErrorCode::Overflow)?;

    let new_reward = updated_stake
        .checked_mul(apr)
        .ok_or(StakingErrorCode::RewardOverflow)?
        .checked_div(10_000)
        .ok_or(StakingErrorCode::RewardOverflow)?;

    // Step 3: Determine how much is already covered
    let already_reserved = amount
        .checked_mul(apr)
        .ok_or(StakingErrorCode::RewardOverflow)?
        .checked_div(10_000)
        .ok_or(StakingErrorCode::RewardOverflow)?;

    let reward_delta = new_reward.saturating_sub(already_reserved);

    // Step 4: Transfer delta of reward if needed
    if reward_delta > 0 {
        require!(
            ctx.accounts.liquidity_vault_token_account.amount >= reward_delta,
            StakingErrorCode::InsufficientVaultBalance
        );

        let seeds: &[&[u8]] = &[b"liquidity_vault", &[ctx.bumps.liquidity_vault]];
        let signer = &[seeds];

        let transfer_rewards = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
                to: ctx.accounts.staking_vault_token_account.to_account_info(),
                authority: ctx.accounts.liquidity_vault.to_account_info(),
            },
            signer,
        );
        transfer(transfer_rewards, reward_delta)?;
    }

    // Step 5: Final updates
    staking_account.staked_tokens = updated_stake;
    staking_account.start_time = clock.unix_timestamp;

    let total_transferred = amount + reward_delta;

    if amount == 0 && old_rewards_compounded > 0 {
        // 1. Only restarting cycle with auto-compound
        msg!(
            "🔁 Stake cycle restarted → User: {} | Compounded rewards: {} tokens from previous cycle | Reserved Rewards for new cycle: {} tokens transferred (liquidity_vault → staking_vault)",
            staking_account.participant,
            old_rewards_compounded,
            reward_delta
        );
    } else if amount > 0 && old_rewards_compounded == 0 {
        // 2. Adding new tokens only (no previous rewards)
        msg!(
            "✅ Stake reinforced → User: {} | Added: {} tokens | Reserved Rewards: {} tokens | Total transferred: {} tokens (user wallet + liquidity vault → staking vault)",
            staking_account.participant,
            amount,
            reward_delta,
            total_transferred
        );
    } else {
        // 3. Adding new tokens + compounding rewards
        msg!(
            "✅ Stake reinforced → User: {} | Added: {} tokens | Compounded: {} tokens from previous cycle | Reserved Rewards: {} tokens | Total transferred: {} tokens (user wallet + liquidity vault → staking vault)",
            staking_account.participant,
            amount,
            old_rewards_compounded,
            reward_delta,
            total_transferred
        );
    }

    emit!(StakeReinforced {
        participant: staking_account.participant,
        added_tokens: amount,
        compounded_rewards: old_rewards_compounded,
        new_reserved_rewards: reward_delta,
        total_transferred,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}


