// ===========================================================================
// Governance Validation Utilities – Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module provides helper functions to validate governance proposals
// before executing actions that depend on prior approval. It ensures that
// only finalized, unused, and correctly categorized proposals are accepted.
//
// ---------------------------------------------------------------------------
// ## Features:
// - Enforces quorum and majority approval
// - Checks proposal finalization and usage status
// - Ensures proposal type matches the expected action
//
// ---------------------------------------------------------------------------
// ## Use Case:
// Used in critical flows like system upgrades, tax updates, or token transfers
// that require community approval before execution.
//
// ---------------------------------------------------------------------------
// ## Dependencies:
// - `GovernanceState` → stores global quorum rules
// - `ProposalAccount` → tracks voting state and proposal metadata
// - `ProposalTypeBit` → identifies the category of the proposal
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use crate::governance::{GovernanceState, ProposalAccount, GovernanceError, proposal_type::ProposalTypeBit};
use crate::economy::TOTAL_SUPPLY;
use anchor_lang::prelude::*;

/// =========================================================================
/// Event: ProposalUsed
///
/// Emitted when a governance proposal is successfully consumed by a
/// sensitive operation (e.g. tax update, system upgrade).
///
/// Useful for off-chain monitoring and audits.
/// =========================================================================
#[event]
pub struct ProposalUsed {
    pub proposal_id: u64,
    pub used_at: i64,
}

/// ===========================================================================
/// Function: require_approved_proposal
///
/// Validates that a governance proposal is **finalized, unused, approved**,
/// and matches the expected proposal type. This is typically called prior
/// to executing sensitive operations that depend on community consensus,
/// such as system upgrades, reward adjustments, or token transfers.
///
/// ## Parameters:
/// - `proposal`: The [`ProposalAccount`] representing the vote outcome
/// - `governance_state`: The global [`GovernanceState`] with quorum settings
/// - `expected_type`: The expected [`ProposalTypeBit`] (e.g. `SystemUpgrade`)
///
/// ## Errors:
/// - [`ProposalNotFinalized`] → Proposal not yet finalized
/// - [`ProposalAlreadyUsed`] → Proposal already consumed
/// - [`ProposalRejected`] → Insufficient quorum or more NO than YES votes
/// - [`MismatchedProposalType`] → The proposal does not match the expected type
///
/// ## Example:
/// ```ignore
/// require_approved_proposal(
///     &proposal_account,
///     &governance_state,
///     ProposalTypeBit::SystemUpgrade,
/// )?;
/// ```
/// ===========================================================================
pub(crate) fn require_approved_proposal(
    proposal: &mut ProposalAccount,
    governance_state: &GovernanceState,
    expected_type: ProposalTypeBit,
) -> Result<()> {
    require!(proposal.is_finalized, GovernanceError::ProposalNotFinalized);
    require!(!proposal.is_used, GovernanceError::ProposalAlreadyUsed);

    let clock = Clock::get()?;

    // Enforce proposal validity window from end_time
    let expiry_limit = governance_state.validity_period;
    let expiration_time = proposal.end_time + expiry_limit;
    require!(
        clock.unix_timestamp <= expiration_time,
        GovernanceError::ProposalExpired
    );

    // Check quorum and vote majority
    let total_votes = proposal.votes_for + proposal.votes_against;
    let quorum_required = (TOTAL_SUPPLY * governance_state.quorum_percent) / 100;

    let passed = total_votes >= quorum_required && proposal.votes_for > proposal.votes_against;
    require!(passed, GovernanceError::ProposalRejected);
    
    // Check expected proposal type is present in bitflag
    let proposal_type_matches = proposal.proposal_bitflag & expected_type.as_bit() != 0;
    require!(proposal_type_matches, GovernanceError::MismatchedProposalType);

    
    Ok(())
}

/// ===========================================================================
/// Function: mark_proposal_as_used
///
/// Marks a finalized and approved governance proposal as used,
/// preventing it from being reused for future operations.  
///
/// This should only be called **after** the corresponding action
/// that depended on the proposal has been executed successfully.
///
/// Emits a [`ProposalUsed`] event for audit purposes.
///
/// ## Parameters:
/// - `proposal`: Mutable reference to the [`ProposalAccount`] to mark as used.
///
/// ## Side Effects:
/// - Sets `is_used = true`
/// - Emits the `ProposalUsed` event
///
/// ## Errors:
/// - Fails if the current clock cannot be retrieved (very rare case)
///
/// ## Example:
/// ```ignore
/// mark_proposal_as_used(&mut proposal_account)?;
/// ```
/// ===========================================================================
pub(crate) fn mark_proposal_as_used(proposal: &mut ProposalAccount) -> Result<()> {
    let clock = Clock::get()?;

    // Mark as used
    proposal.is_used = true;
    emit!(ProposalUsed {
        proposal_id: proposal.id,
        used_at: clock.unix_timestamp,
    });
    Ok(())
}