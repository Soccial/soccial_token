// ========================================================================
// Soccial Token – Governance Error Definitions
//
// This module defines all custom error codes related to the governance
// system, including proposal creation, voting logic, and validation rules.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ========================================================================

use anchor_lang::prelude::*;

/// Errors related to governance operations.
#[error_code]
pub enum GovernanceError {

    /// User attempted to vote after the voting period ended.
    #[msg("Voting period has ended.")]
    VotingPeriodEnded,

    /// User attempted to vote before the voting period started.
    #[msg("Voting has not started yet.")]
    VotingNotStarted,

    /// Proposal is still in its voting period; cannot finalize yet.
    #[msg("Voting is still active.")]
    VotingStillActive,

    /// User does not hold the minimum required tokens to vote.
    #[msg("Not enough tokens to vote.")]
    InsufficientTokens,

    /// The provided proposal type does not match any known type.
    #[msg("Invalid proposal type.")]
    InvalidProposalType,

    /// User holds tokens, but not enough to meet the threshold to vote.
    #[msg("Insufficient tokens to vote.")]
    InsufficientTokensToVote,

    /// Staff accounts are not allowed to participate in voting.
    #[msg("Staff cant vote.")]
    StaffCannotVote,

    /// A proposal was configured with a start time in the past.
    #[msg("Start time cannot be in the past.")]
    StartTimeInPast,

    /// Proposal has already been finalized and cannot be modified.
    #[msg("Proposal is already finalized.")]
    ProposalAlreadyFinalized,

    /// Proposal is not yet finalized, so it cannot be executed.
    #[msg("Proposal is not finalized yet.")]
    ProposalNotFinalized,

    /// Proposal did not meet quorum or was rejected.
    #[msg("Proposal was rejected or did not reach quorum.")]
    ProposalRejected,

    /// The type of the finalized proposal does not match the expected action.
    #[msg("Proposal type does not match expected action.")]
    MismatchedProposalType,

    /// The proposal has already been used in a system action and cannot be reused.
    #[msg("Proposal already used.")]
    ProposalAlreadyUsed,

    /// The proposal has expired
    #[msg("Proposal already used.")]
    ProposalExpired,
}
