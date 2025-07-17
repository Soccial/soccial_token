// ===========================================================================
// Soccial Token – User Access & Associated Token Account Contexts
// ---------------------------------------------------------------------------
//
// This module defines account contexts related to:
// - Creating a user's Associated Token Account (ATA) for SCTK
// - Managing user access control (permission assignment & revocation)
//
// These contexts are used to ensure proper authority validation, seed derivation,
// and permission handling across the Soccial Token smart contract.
//
// ---------------------------------------------------------------------------
// Key Features:
// - Permissioned user access with optional `UserAccessAccount` validation
// - Supports both initialization and removal of user access entries
// - Ensures the TokenState is initialized before performing operations
//
// ---------------------------------------------------------------------------
// Contexts:
// - `CreateUserAta`: Initializes user's ATA for the SCTK token
// - `ManageUser`: Grants permissions or marks user as admin
// - `ManageUserRemove`: Revokes user permissions or admin status
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;

use crate::{auth::user::UserAccessAccount, token::TokenState};

#[derive(Accounts)]
pub struct ManageUser<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller managing a user (must be authorized).
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // Caller Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Target User + Access Control
    // ─────────────────────────────────────────────────────────────

    /// CHECK: The target user public key; used for PDA derivation only.
    pub target_user: AccountInfo<'info>,

    /// Access control account for the target user (created if missing).
    #[account(
        init_if_needed,
        payer = caller,
        seeds = [b"user_access", target_user.key().as_ref()],
        bump,
        space = 8 + core::mem::size_of::<UserAccessAccount>(),
    )]
    pub target_user_access: Account<'info, UserAccessAccount>,

    // ─────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────

    /// Global token configuration state.
    pub token_state: Account<'info, TokenState>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// Required for PDA creation and rent transfers.
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct ManageUserRemove<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller removing a user access entry (must be authorized).
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // Caller Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Target User + Access Control
    // ─────────────────────────────────────────────────────────────

    /// CHECK: The target user public key; used for PDA derivation only.
    pub target_user: AccountInfo<'info>,

    /// Access control account to be removed for the target user.
    #[account(
        mut,
        seeds = [b"user_access", target_user.key().as_ref()],
        bump,
    )]
    pub target_user_access: Account<'info, UserAccessAccount>,

    // ─────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────

    /// Global token configuration state.
    pub token_state: Account<'info, TokenState>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// Required for PDA manipulation and cleanup operations.
    pub system_program: Program<'info, System>,
}
