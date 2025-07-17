// ===========================================================================
// Staking Output Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles **reward claiming** and **stake withdrawals** in the
// Soccial Token staking system. It ensures users can securely access their
// rewards or exit the staking cycle once the lockup period completes.
//
// ---------------------------------------------------------------------------
// ## Core Capabilities:
// - Claim rewards for each completed cycle (auto-renewing system)
// - Withdraw full stake (principal + reward) after lockup ends
// - Ensures PDA and ATA integrity before transferring tokens
//
// ---------------------------------------------------------------------------
// ## Reward Claiming Logic:
// - `claim_rewards`: Allows reward-only withdrawal per cycle
//   - Resets `start_time` for the next cycle
//   - Updates `total_rewards_claimed` and `cycles_completed`
//   - Keeps staked tokens locked until withdrawn
//
// ---------------------------------------------------------------------------
// ## Withdrawal Logic:
// - `withdraw_staked_tokens`: Full exit
//   - Transfers staked amount + reward
//   - Validates staking account ownership and ATA match
//   - Flags account as `withdrawn = true` (irreversible)
//
// ---------------------------------------------------------------------------
// ## Design Notes:
// - Reward rate based on APR in basis points (100 = 1%)
// - Withdrawals and claims are **mutually exclusive**
// - Rewards can be claimed multiple times (one per cycle)
// - PDA address integrity is enforced for security
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================


use anchor_lang::prelude::*;
use anchor_spl::token::{Transfer, transfer};
use spl_associated_token_account::get_associated_token_address;

use crate::staking::{context::*, StakingErrorCode};
use crate::utils::error::ErrorCode;

#[event]
pub struct StakingRewardClaimed {
    pub participant: Pubkey,
    pub stake_id: u64,
    pub amount: u64,
    pub cycle: u16,
    pub total_rewards_claimed: u64,
    pub claimed_at: i64,
}

#[event]
pub struct StakingWithdrawn {
    pub participant: Pubkey,
    pub stake_id: u64,
    pub staked_amount: u64,
    pub reward_amount: u64,
    pub total_withdrawn: u64,
    pub withdrawn_at: i64,
}

/// ===========================================================================
/// Claims Rewards After Each Completed Staking Cycle
///
/// Allows a user to claim **only the reward portion** of a completed cycle,
/// without withdrawing the originally staked tokens. The cycle is then reset,
/// allowing rewards to continue accumulating in future cycles.
///
/// ## Behavior:
/// - Validates that staking lockup period has passed
/// - Calculates the reward based on stake and APR
/// - Transfers reward tokens from the staking vault to the user
/// - Updates cycle state (`start_time`, `total_rewards_claimed`, etc.)
///
/// ## Notes:
/// - Multiple claims are allowed, one per cycle
/// - Full withdrawal (principal + reward) should use `withdraw_staked_tokens`
///
/// ## Errors:
/// - `StakingPeriodNotOver` if lockup time hasn't passed
/// - `RewardOverflow`, `InsufficientVaultBalance`
/// ===========================================================================
pub(crate) fn claim_rewards(
    ctx: &mut Context<ReleaseStaked>
) -> Result<()> {
    let staking_account = &mut ctx.accounts.staking_account;
    let clock = &ctx.accounts.clock;

    // Step 1: Ensure the current cycle is complete
    let lockup_end = staking_account.start_time + staking_account.lockup_duration;
    require!(
        clock.unix_timestamp >= lockup_end,
        StakingErrorCode::StakingPeriodNotOver
    );

    // Step 2: Calculate reward
    let apr = staking_account.apr_bps as u128;
    let stake = staking_account.staked_tokens as u128;

    let reward = stake
        .checked_mul(apr)
        .ok_or(StakingErrorCode::RewardOverflow)?
        .checked_div(10_000)
        .ok_or(StakingErrorCode::RewardOverflow)? as u64;

    // Step 3: Ensure vault has enough balance
    require!(
        ctx.accounts.staking_vault_token_account.amount >= reward,
        StakingErrorCode::InsufficientVaultBalance
    );

    // -------------------------------------
    // Step 4: Transfer reward from staking vault to user
    // -------------------------------------
    let bump = ctx.bumps.staking_vault;
    let signer_seeds: &[&[u8]] = &[b"staking_vault", &[bump]];
    let signer: &[&[&[u8]]] = &[signer_seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.staking_vault.to_account_info(),
        },
        signer,
    );

    transfer(cpi_ctx, reward)?;

    // Step 5: Update staking state
    staking_account.start_time = clock.unix_timestamp;
    staking_account.total_rewards_claimed = staking_account
        .total_rewards_claimed
        .checked_add(reward)
        .ok_or(StakingErrorCode::Overflow)?;
    staking_account.cycles_completed += 1;

    // Step 6: Final log
    msg!(
        "🎁 Claimed {} tokens for cycle {} → Staker: {} | Total rewards so far: {}",
        reward,
        staking_account.cycles_completed,
        staking_account.participant,
        staking_account.total_rewards_claimed
    );

    emit!(StakingRewardClaimed {
        participant: staking_account.participant,
        stake_id: staking_account.stake_id,
        amount: reward,
        cycle: staking_account.cycles_completed,
        total_rewards_claimed: staking_account.total_rewards_claimed,
        claimed_at: clock.unix_timestamp,
    });


    Ok(())
}


/// ===========================================================================
/// Withdraws Staked Tokens and Rewards After Lockup
///
/// Transfers both the **original staked tokens** and the **earned rewards**
/// back to the user, after the staking lockup period has ended.
///
/// ## Behavior:
/// - Validates lockup period
/// - Checks PDA integrity (security)
/// - Calculates reward and total payout
/// - Transfers tokens from `staking_vault` to the user’s ATA
///
/// ## Notes:
/// - Mutually exclusive with `claim_rewards` – both mark `withdrawn = true`
/// - Ensures destination ATA matches the staking participant
///
/// ## Errors:
/// - `StakingPeriodNotOver`, `AlreadyWithdrawn`
/// - `Unauthorized`, `RewardOverflow`, `Overflow`
/// ===========================================================================
pub(crate) fn withdraw_staked_tokens(
    ctx: &mut Context<WithdrawStaked>
) -> Result<()> {
    let clock = &ctx.accounts.clock;
    let staking_account = &mut ctx.accounts.staking_account;

    // Step 1: Check if staking period has ended
    require!(
        clock.unix_timestamp >= staking_account.start_time + staking_account.lockup_duration,
        StakingErrorCode::StakingPeriodNotOver
    );

    // Step 2: Ensure the stake has not already been withdrawn
    require!(
        !staking_account.withdrawn,
        StakingErrorCode::AlreadyWithdrawn
    );

    // Step 3: Validate that the staking account matches expected PDA
    let (expected_pda, _) = Pubkey::find_program_address(
        &[
            b"staking_account",
            staking_account.participant.as_ref(),
            &staking_account.stake_id.to_le_bytes()
        ],
        ctx.program_id
    );

    require_keys_eq!(
        staking_account.key(),
        expected_pda,
        ErrorCode::Unauthorized 
    );

    // Step 4: Calculate reward and total payout
    let reward = staking_account.staked_tokens
        .checked_mul(staking_account.apr_bps as u64)
        .ok_or(StakingErrorCode::RewardOverflow)?
        / 10_000;

    let total_payout = staking_account
        .staked_tokens
        .checked_add(reward)
        .ok_or(StakingErrorCode::Overflow)?;

    // Step 5: Mark as withdrawn
    staking_account.withdrawn = true;

    // Step 6: Validate destination ATA is the participant's
    let expected_ata = get_associated_token_address(
        &staking_account.participant,
        &ctx.accounts.mint.key()
    );

    require_keys_eq!(
        ctx.accounts.destination_token_account.key(),
        expected_ata,
        ErrorCode::Unauthorized 
    );

    // Step 7: Transfer staked + reward from staking vault to participant
    let seeds: &[&[u8]] = &[b"staking_vault", &[ctx.bumps.staking_vault]];
    let signer: &[&[&[u8]]] = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staking_vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.staking_vault.to_account_info(),
        },
        signer,
    );
    transfer(cpi_ctx, total_payout)?;

    // Step 8: Final log
    msg!(
        "💸 Withdrawn total {} ({} staked + {} reward) tokens for staker {}",
        total_payout,
        staking_account.staked_tokens,
        reward,
        staking_account.participant
    );

    emit!(StakingWithdrawn {
        participant: staking_account.participant,
        stake_id: staking_account.stake_id,
        staked_amount: staking_account.staked_tokens,
        reward_amount: reward,
        total_withdrawn: total_payout,
        withdrawn_at: clock.unix_timestamp,
    });

    Ok(())
}
