// ===========================================================================
// Governance Proposal Logic – Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles the lifecycle of governance proposals:
// from creation and configuration to vote finalization and outcome reporting.
//
// ---------------------------------------------------------------------------
// ## Features:
// - Proposal creation with type bitflags and custom timing
// - Validation of voting periods and quorum
// - Finalization with pass/fail logic and logging
//
// ---------------------------------------------------------------------------
// ## Proposal Mechanics:
// - Each proposal has one or more types (bitflag from `ProposalTypeBit`)
// - Quorum is defined by the percentage of TOTAL_SUPPLY in `GovernanceState`
// - Votes are cast using token balances (with optional bonus logic)
//
// ---------------------------------------------------------------------------
// ## Dependencies:
// - `ProposalTypeBit`: Defines logical categories of proposals
// - `GovernanceState`: Stores config like quorum, durations, counters
// - `ProposalAccount`: Stores each proposal’s metadata and vote state
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::{
    economy::TOTAL_SUPPLY, 
    governance::{
        proposal_type::*, 
        GovernanceError,
        context::{CreateProposal, FinalizeProposal}
    }, 
};

#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub creator: Pubkey,
    pub description: String,
    pub proposal_bitflag: u32,
    pub start_time: i64,
    pub end_time: i64,
    pub duration: i64,
}

#[event]
pub struct ProposalFinalized {
    pub proposal_id: u64,
    pub passed: bool,
    pub votes_for: u64,
    pub votes_against: u64,
    pub total_votes: u64,
    pub quorum_required: u64,
    pub finalized_at: i64,
}

/// ===========================================================================
/// Function: create_proposal
///
/// Initializes a new proposal for governance voting.
/// This function allows the proposer to define one or more types (bitflags),
/// a description, and optional start and duration times.
///
/// ## Behavior:
/// - Resolves `start_time` (defaults to now) and `end_time`
/// - Converts string identifiers to `ProposalTypeBit` and builds bitflag
/// - Increments global proposal counter in `GovernanceState`
/// - Saves all metadata in `ProposalAccount`
///
/// ## Parameters:
/// - `ctx`: Context with governance state, proposal account and caller
/// - `description`: Human-readable explanation of the proposal
/// - `proposal_type_strs`: List of string identifiers for the proposal categories
/// - `start_time`: Optional UNIX timestamp (or uses current blockchain time)
/// - `duration`: Optional voting period in seconds (or uses default)
///
/// ## Errors:
/// - `InvalidProposalType`: If any string type is unrecognized
/// - `StartTimeInPast`: If a future start time is not respected
///
/// ## Example:

/// create_proposal(
///     ctx,
///     "Upgrade system fees".to_string(),
///     vec!["AdjustTaxRate".to_string()],
///     None,
///     Some(86400)
/// );

/// ===========================================================================
pub(crate) fn create_proposal(
    ctx: Context<CreateProposal>,
    description: String,
    proposal_type_strs: Vec<String>,
    start_time: Option<i64>,
    duration: Option<i64>,
) -> Result<()> {
    let governance_state = &mut ctx.accounts.governance_state;
    let proposal_account = &mut ctx.accounts.proposal_account;
    let clock = Clock::get()?;

    // Build bitflag from proposal types
    let mut bitmask: u32 = 0;
    for type_str in proposal_type_strs.iter() {
        let parsed = ProposalTypeBit::from_str(type_str)
            .ok_or(GovernanceError::InvalidProposalType)?;
        bitmask |= parsed.as_bit();
    }

    // Resolve times
    let resolved_start_time = start_time.unwrap_or(clock.unix_timestamp);
    require!(
        resolved_start_time >= clock.unix_timestamp,
        GovernanceError::StartTimeInPast
    );

    let resolved_duration = duration.unwrap_or(governance_state.voting_duration);
    let end_time = resolved_start_time + resolved_duration;

    // Initialize proposal account
    
    proposal_account.id = governance_state.last_id;
    proposal_account.description = description.clone();
    proposal_account.proposal_bitflag = bitmask;
    proposal_account.votes_for = 0;
    proposal_account.votes_against = 0;
    proposal_account.start_time = resolved_start_time;
    proposal_account.end_time = end_time;
    proposal_account.creator = ctx.accounts.caller.key();

    msg!(
        "📜 Proposal {} created: {} | Starts at: {} | Ends at: {} | Duration: {}s",
        proposal_account.id,
        description,
        resolved_start_time,
        end_time,
        resolved_duration
    );

    emit!(ProposalCreated {
        proposal_id: proposal_account.id,
        creator: ctx.accounts.caller.key(),
        description,
        proposal_bitflag: bitmask,
        start_time: resolved_start_time,
        end_time,
        duration: resolved_duration,
    });

    // Increment global counter
    governance_state.last_id += 1;


    Ok(())
}

/// ===========================================================================
/// Function: finalize_proposal
///
/// Finalizes a governance proposal after its voting period ends.
/// It checks quorum and vote majority, then logs whether the proposal passed.
///
/// ## Behavior:
/// - Ensures voting period has ended
/// - Calculates total votes and required quorum
/// - Marks the proposal as finalized
/// - Logs PASS/FAIL result with detail
///
/// ## Parameters:
/// - `ctx`: Context with the governance and proposal accounts
///
/// ## Quorum:
/// - Calculated as `(TOTAL_SUPPLY * quorum_percent) / 100`
/// - Proposal must exceed this threshold and have majority FOR votes
///
/// ## Errors:
/// - `ProposalAlreadyFinalized`: If called more than once
/// - `VotingStillActive`: If called before end time
///
/// ## Example:

/// finalize_proposal(ctx)?;

/// ===========================================================================
pub(crate) fn finalize_proposal(
    ctx: Context<FinalizeProposal>,
) -> Result<()> {
    let governance_state = &ctx.accounts.governance_state;
    let proposal_account = &mut ctx.accounts.proposal_account;

    require!(
        !proposal_account.is_finalized,
        GovernanceError::ProposalAlreadyFinalized
    );

    let clock = &ctx.accounts.clock;

    // Ensure the voting period has ended
    require!(
        clock.unix_timestamp > proposal_account.end_time,
        GovernanceError::VotingStillActive
    );
    
    // Calculate total votes and required quorum
    let total_votes = proposal_account.votes_for + proposal_account.votes_against;
    let quorum_required = (TOTAL_SUPPLY * governance_state.quorum_percent) / 100;

    proposal_account.is_finalized = true;

    // Check if the proposal passes based on quorum and votes
   if total_votes >= quorum_required && proposal_account.votes_for > proposal_account.votes_against {
        msg!(
            "✅ Proposal {} PASSED | ✅ Votes For: {} > ❌ Votes Against: {} | 🗳️ Total Votes: {} / Quorum Required: {}",
            proposal_account.id,
            proposal_account.votes_for,
            proposal_account.votes_against,
            total_votes,
            quorum_required
        );
    } else {
        msg!(
            "❌ Proposal {} FAILED | ✅ Votes For: {} <= ❌ Votes Against: {} | 🗳️ Total Votes: {} / Quorum Required: {}",
            proposal_account.id,
            proposal_account.votes_for,
            proposal_account.votes_against,
            total_votes,
            quorum_required
        );
    }

    let passed = total_votes >= quorum_required && proposal_account.votes_for > proposal_account.votes_against;

    emit!(ProposalFinalized {
        proposal_id: proposal_account.id,
        passed,
        votes_for: proposal_account.votes_for,
        votes_against: proposal_account.votes_against,
        total_votes,
        quorum_required,
        finalized_at: clock.unix_timestamp,
    });


    Ok(())
}
