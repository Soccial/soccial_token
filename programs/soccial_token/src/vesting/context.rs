// ======================================================================
// Soccial Token – Vesting Contexts
//
// This file defines all account contexts used for managing vesting schedules,
// including creation, editing, claiming, immutability enforcement and release.
//
// These contexts enforce correct access control, PDA derivation and token flow
// between liquidity vaults, vesting vaults and recipients.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Token, Mint, TokenAccount};
use crate::vesting::state::VestingSchedule;
use crate::{auth::user::UserAccessAccount, token::TokenState};

use super::VestingState;

/// Context for releasing vested tokens, either by the participant or an admin.
#[derive(Accounts)]
pub struct ReleaseVestedTokens<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The user or admin triggering the vesting release.
    pub caller: Signer<'info>,

    /// Access control metadata for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    /// Global token configuration state.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Vesting Schedule & Metadata
    // =========================================================================

    /// The schedule tracking vesting amounts and timestamps.
    #[account(
        mut,
        seeds = [
            b"vesting_schedule", 
            vesting_schedule.participant.as_ref(), 
            vesting_schedule.vesting_id.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub vesting_schedule: Account<'info, VestingSchedule>,

    /// System account to receive lamports when vesting account is closed.
    #[account(mut, address = token_state.core.owner)]
    pub recipient_of_lamports: SystemAccount<'info>,

    // =========================================================================
    // Token Mint & Authorities
    // =========================================================================

    /// The SPL token mint (SCTK).
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority over mint and vesting vaults.
    #[account(
        mut,
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    // =========================================================================
    // Vault & Token Accounts
    // =========================================================================

    /// CHECK: PDA representing the vault holding vested tokens.
    #[account(
        mut,
        seeds = [b"vesting_vault"],
        bump,
    )]
    pub vesting_vault: AccountInfo<'info>,

    /// Token account owned by `vesting_vault`, holding vested tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting_vault,
    )]
    pub vesting_vault_token_account: Account<'info, TokenAccount>,

    /// User’s token account to receive vested tokens.
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Programs & Sysvars
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ManageVesting<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller managing vesting schedules.
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional access control for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    /// Global token configuration.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Participant & Vesting Schedule
    // =========================================================================

    /// CHECK: The participant receiving the vesting schedule.
    #[account()]
    pub participant: AccountInfo<'info>,

    /// Vesting schedule account for this participant and vesting ID.
    #[account(
        init_if_needed,
        seeds = [
            b"vesting_schedule", 
            participant.key().as_ref(), 
            vesting_state.last_id.to_le_bytes().as_ref()
        ],
        bump,
        payer = caller,
        space = VestingSchedule::LEN,
    )]
    pub vesting_schedule: Account<'info, VestingSchedule>,

    /// Global vesting state to track ID counters and settings.
    #[account(
        mut,
        seeds = [b"vesting_state"],
        bump,
    )]
    pub vesting_state: Account<'info, VestingState>,

    // =========================================================================
    // Token Mint & Authorities
    // =========================================================================

    /// The token mint used for vesting allocations.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA mint authority for vaults and transfers.
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    // =========================================================================
    // Vaults & Token Accounts
    // =========================================================================

    /// CHECK: Liquidity vault PDA (source of vested tokens).
    #[account(
        mut,
        seeds = [b"liquidity_vault"],
        bump
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account owned by `liquidity_vault` holding source tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = liquidity_vault,
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: Vesting vault PDA (target where tokens are locked).
    #[account(
        mut,
        seeds = [b"vesting_vault"],
        bump
    )]
    pub vesting_vault: AccountInfo<'info>,

    /// Token account owned by `vesting_vault` to receive reserved tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting_vault,
    )]
    pub vesting_vault_token_account: Account<'info, TokenAccount>,

    /// Participant’s associated token account (optional use).
    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = mint,
        associated_token::authority = participant,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct EditVestingSchedule<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller performing the action (e.g. cancel, update).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional user access control account.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Participant & Vesting Schedule
    // =========================================================================

    /// CHECK: Participant associated with the vesting schedule.
    pub participant: AccountInfo<'info>,

    /// Vesting schedule being modified.
    #[account(
        mut,
        seeds = [
            b"vesting_schedule",
            participant.key().as_ref(),
            vesting_schedule.vesting_id.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub vesting_schedule: Account<'info, VestingSchedule>,

    /// CHECK: Vesting state (used optionally for stats or validations).
    #[account(
        seeds = [b"vesting_state"],
        bump,
    )]
    pub vesting_state: Account<'info, VestingState>,

    // =========================================================================
    // Token Mint & Authorities
    // =========================================================================

    /// Token mint used for vesting allocations.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA mint authority.
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    // =========================================================================
    // Vaults & Token Accounts
    // =========================================================================

    /// CHECK: Liquidity vault PDA (source of refund tokens).
    #[account(
        mut,
        seeds = [b"liquidity_vault"],
        bump
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account owned by liquidity_vault.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = liquidity_vault,
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: Vesting vault PDA (holding reserved tokens).
    #[account(
        mut,
        seeds = [b"vesting_vault"],
        bump
    )]
    pub vesting_vault: AccountInfo<'info>,

    /// Token account owned by vesting_vault.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = vesting_vault,
    )]
    pub vesting_vault_token_account: Account<'info, TokenAccount>,

    /// Associated token account for participant (may receive refunds).
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = participant,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ImmutableVestingSchedule<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller performing the action (e.g. cancel, update).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional user access control account.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Participant & Vesting Schedule
    // =========================================================================

    /// CHECK: Participant associated with the vesting schedule.
    pub participant: AccountInfo<'info>,

    /// Vesting schedule being validated or marked immutable.
    #[account(
        mut,
        seeds = [
            b"vesting_schedule",
            participant.key().as_ref(),
            vesting_schedule.vesting_id.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub vesting_schedule: Account<'info, VestingSchedule>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub system_program: Program<'info, System>,
}
