// ===========================================================================
// Governance Voting Logic – Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module implements the on-chain governance voting logic for the Soccial Token.
//
// ## Highlights:
// - Allows users to vote YES/NO on proposals
// - Applies bonus voting power for Early Adopters
// - Prevents staff accounts from voting
// - Enforces proposal timing and minimum token thresholds
//
// ---------------------------------------------------------------------------
// ## Voting Power Calculation:
// - Based on user's token balance (`amount` from their token account)
// - EarlyAdopter1 → +X bonus tokens
// - EarlyAdopter2 → +Y bonus tokens
//
// ---------------------------------------------------------------------------
// ## Restrictions:
// - Voting only allowed during proposal window
// - Staff users are disqualified from voting
// - Votes must meet the min voting power defined in GovernanceState
//
// ---------------------------------------------------------------------------
// ## Emits:
// - Log detailing who voted, vote direction, base amount, bonus (if any), and total voting power
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::{
    auth::user::ExtraFlag, 
    economy::TOKEN_DECIMAL, 
    governance::{context::VoteOnProposal, 
        GovernanceError, VoteReceipt
    }, 
    utils::math::format_sctk
};

 /// Bonus votes for early adopters phase 1
const EARLY_ADOPTER1_BONUS: u64 = 1_000 * 10u64.pow(TOKEN_DECIMAL as u32);

/// Bonus votes for early adopters phase 2
const EARLY_ADOPTER2_BONUS: u64 = 500 * 10u64.pow(TOKEN_DECIMAL as u32);

#[event]
pub(crate) struct VoteCast {
    pub caller: Pubkey,
    pub proposal: Pubkey,
    pub support: bool,
    pub base_tokens: u64,
    pub offchain_tokens: u64,
    pub staking_tokens: u64,
    pub bonus_applied: Option<String>,
    pub bonus_tokens: Option<u64>,
    pub total_votes: u64,
    pub timestamp: i64,
}

/// ===========================================================================
/// Casts a YES or NO vote on a governance proposal, using token balance as voting power.
///
/// ## Parameters:
/// - `ctx`: Context containing proposal account, user access, SPL token balance, and clock
/// - `support`: `true` for YES, `false` for NO
///
/// ## Behavior:
/// - Reads SPL token balance from user's associated token account
/// - Checks if voting is within the proposal’s valid time window
/// - Adds voting power bonuses for Early Adopters (optional)
/// - Staff accounts are blocked from voting
/// - Validates minimum voting power required
/// - Increments `votes_for` or `votes_against` on the proposal
///
/// ## Bonus System:
/// - `EarlyAdopter1` flag → adds `EARLY_ADOPTER1_BONUS` tokens to vote power
/// - `EarlyAdopter2` flag → adds `EARLY_ADOPTER2_BONUS` tokens to vote power
///
/// ## Restrictions:
/// - Staff members (`ExtraFlag::Staff`) cannot vote
/// - Voting must occur within start and end time of proposal
/// - Vote must meet the `min_vote_tokens` requirement
///
/// ## Emits:
/// - Logs vote direction, caller, base amount, bonus (if any), and total power used
///
/// ## Errors:
/// - `VotingNotStarted` if before proposal start
/// - `VotingPeriodEnded` if after proposal end
/// - `StaffCannotVote` if caller is flagged as staff
/// - `InsufficientTokens` if vote power is below threshold
/// ===========================================================================
pub(crate) fn vote(
    ctx: Context<VoteOnProposal>,
    support: bool,
    total_offchain: u64,
    total_staking: u64
) -> Result<()> {
    let caller = ctx.accounts.caller.key();
    let clock = &ctx.accounts.clock;

    let governance_state = &ctx.accounts.governance_state;
    let proposal_account = &mut ctx.accounts.proposal_account;

    // ------------------------------------------------------------------------
    // Voting time check
    // ------------------------------------------------------------------------

    require!(
        clock.unix_timestamp >= proposal_account.start_time,
        GovernanceError::VotingNotStarted
    );

    require!(
        clock.unix_timestamp < proposal_account.end_time,
        GovernanceError::VotingPeriodEnded
    );

    // ------------------------------------------------------------------------
    // Vote power calculation
    // ------------------------------------------------------------------------

    let user_balance = ctx.accounts.user_token_account.amount;
    let mut vote_amount = user_balance + total_offchain + total_staking;

    let mut bonus_applied: Option<(&str, u64)> = None;

    if let Some(user_access) = &ctx.accounts.user_access {
        if user_access.has_flag(ExtraFlag::Staff) {
            return err!(GovernanceError::StaffCannotVote);
        }

        if user_access.has_flag(ExtraFlag::EarlyAdopter1) {
            vote_amount += EARLY_ADOPTER1_BONUS;
            bonus_applied = Some(("EarlyAdopter1", EARLY_ADOPTER1_BONUS));
        } else if user_access.has_flag(ExtraFlag::EarlyAdopter2) {
            vote_amount += EARLY_ADOPTER2_BONUS;
            bonus_applied = Some(("EarlyAdopter2", EARLY_ADOPTER2_BONUS));
        }
    }

    // ------------------------------------------------------------------------
    // Minimum voting power check
    // ------------------------------------------------------------------------

    require!(
        vote_amount >= governance_state.min_vote_tokens,
        GovernanceError::InsufficientTokens
    );

    // ------------------------------------------------------------------------
    // Tally vote
    // ------------------------------------------------------------------------

    if support {
        proposal_account.votes_for += vote_amount;
    } else {
        proposal_account.votes_against += vote_amount;
    }

    // ------------------------------------------------------------------------
    // Vote receipt
    // ------------------------------------------------------------------------
    let vote_receipt = &mut ctx.accounts.vote_receipt;
        **vote_receipt = VoteReceipt {
            proposal: proposal_account.key(),
            voter: caller,
            support,
            vote_power: vote_amount,
            timestamp: clock.unix_timestamp,
        };
    
    // ------------------------------------------------------------------------
    // Final log
    // ------------------------------------------------------------------------

    let direction = if support { "✅ YES" } else { "❌ NO" };
    let formatted_base = format_sctk(user_balance);
    let formatted_offchain = format_sctk(total_offchain);
    let formatted_staking = format_sctk(total_staking);
    let formatted_total = format_sctk(vote_amount);

    match bonus_applied {
        Some((bonus_type, bonus_value)) => {
            let formatted_bonus = format_sctk(bonus_value);
            msg!(
                "🗳️ VOTE → {} by {} | base: {} SCTK | offchain: {} | staking: {} | bonus: {} ({}) | total: {} SCTK",
                direction,
                caller,
                formatted_base,
                formatted_offchain,
                formatted_staking,
                formatted_bonus,
                bonus_type,
                formatted_total
            );
        }
        None => {
            msg!(
                "🗳️ VOTE → {} by {} | base: {} SCTK | offchain: {} | staking: {} | total: {} SCTK (no bonus)",
                direction,
                caller,
                formatted_base,
                formatted_offchain,
                formatted_staking,
                formatted_total
            );
        }
    }

    emit!(VoteCast {
        caller,
        proposal: ctx.accounts.proposal_account.key(),
        support,
        base_tokens: user_balance,
        offchain_tokens: total_offchain,
        staking_tokens: total_staking,
        bonus_applied: bonus_applied.map(|(bonus_type, _)| bonus_type.to_string()),
        bonus_tokens: bonus_applied.map(|(_, value)| value),
        total_votes: vote_amount,
        timestamp: clock.unix_timestamp,
    });


    Ok(())
}
