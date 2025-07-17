// ======================================================================
// Soccial Token – Governance Contexts
//
// This file defines account contexts used for proposal lifecycle actions,
// including creation, voting, finalization, and upgrade execution.
//
// Each context ensures the correct access control, seed derivation,
// and system constraints required to safely interact with governance logic.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{
    governance::GovernanceState,
    governance::state::*,
    auth::user::UserAccessAccount,
    token::TokenState,
};

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The user casting the vote.
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional user access account, used to check staff and early adopter flags.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Governance Accounts
    // =========================================================================

    /// Governance state configuration.
    #[account(
        seeds = [b"governance_state"],
        bump,
    )]
    pub governance_state: Account<'info, GovernanceState>,

    /// The proposal being voted on.
    #[account(mut)]
    pub proposal_account: Account<'info, ProposalAccount>,

    /// Vote receipt for this caller & proposal (PDA created on first vote).
    #[account(
        init,
        payer = caller,
        space = 8 + core::mem::size_of::<VoteReceipt>(),
        seeds = [b"vote", proposal_account.key().as_ref(), caller.key().as_ref()],
        bump
    )]
    pub vote_receipt: Account<'info, VoteReceipt>,

    // =========================================================================
    // Token Info
    // =========================================================================

    /// Token mint of SCTK.
    /// CHECK: Verified via address constraint at runtime if needed.
    pub token_mint: Account<'info, Mint>,

    /// Associated Token Account (ATA) of the caller for SCTK.
    #[account(
        associated_token::mint = token_mint,
        associated_token::authority = caller
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Global token state for economics (e.g. decimals).
    #[account(
        seeds = [b"token_state"],
        bump,
    )]
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // System Programs
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar clock for time-based logic.
    pub clock: Sysvar<'info, Clock>,
}


#[derive(Accounts)] 
pub struct CreateProposal<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller initiating the proposal creation.
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // State Accounts
    // ─────────────────────────────────────────────────────────────

    /// Governance state config (tracks quorum, durations, counters).
    #[account(mut)]
    pub governance_state: Account<'info, GovernanceState>,

    /// Global token state (may be used for constraints or proposal logic).
    pub token_state: Account<'info, TokenState>,

    // ─────────────────────────────────────────────────────────────
    // Proposal Account (Initialized)
    // ─────────────────────────────────────────────────────────────

    /// The new proposal account to be created (PDA using last_id).
    #[account(
        init,
        payer = caller,
        space = ProposalAccount::LEN,
        seeds = [b"proposal", governance_state.last_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub proposal_account: Account<'info, ProposalAccount>,

    // ─────────────────────────────────────────────────────────────
    // User Access (Optional)
    // ─────────────────────────────────────────────────────────────

    /// Optional user access permissions (e.g., staff or early adopter).
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// System program required to create the proposal account.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)] 
pub struct FinalizeProposal<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller finalizing the proposal.
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // State Accounts
    // ─────────────────────────────────────────────────────────────

    /// Governance state config (holds quorum, durations, etc.).
    pub governance_state: Account<'info, GovernanceState>,

    /// Global token state (may be used in quorum logic).
    pub token_state: Account<'info, TokenState>,

    /// The proposal being finalized.
    #[account(mut)]
    pub proposal_account: Account<'info, ProposalAccount>,

    // ─────────────────────────────────────────────────────────────
    // User Access (Optional)
    // ─────────────────────────────────────────────────────────────

    /// Optional user access permissions (e.g., to restrict or elevate roles).
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Programs & Sysvars
    // ─────────────────────────────────────────────────────────────

    /// System program (in case of future dynamic logic).
    pub system_program: Program<'info, System>,

    /// CHECK: Clock sysvar (used to verify voting end time).
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ManageGovernance<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller managing governance settings (must be authorized).
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // State Accounts
    // ─────────────────────────────────────────────────────────────

    /// The GovernanceState account holding all governance parameters.
    #[account(mut)]
    pub governance_state: Account<'info, GovernanceState>,

    /// Global token configuration (may be referenced for constraints).
    pub token_state: Account<'info, TokenState>,

    // ─────────────────────────────────────────────────────────────
    // User Access (Optional)
    // ─────────────────────────────────────────────────────────────

    /// Optional user access permissions (to validate role/authorization).
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// System program used for possible rent-exempt operations or transfers.
    pub system_program: Program<'info, System>,
}
