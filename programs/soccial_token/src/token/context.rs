// ======================================================================
// Soccial Token – Contract Management Contexts
//
// Defines account contexts for core contract-level operations,
// including configuration management, system logging, and development-only
// token minting for testing purposes.
//
// Access control is enforced via PDA-derived user access accounts.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================
use anchor_lang::{prelude::*, solana_program};
use crate::governance::GovernanceState;
use crate::{auth::user::UserAccessAccount, governance::ProposalAccount, token::TokenState};

#[derive(Accounts)]
pub struct ManageContract<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller performing contract-level operations.
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────

    /// TokenState account managing the global configuration.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    // ─────────────────────────────────────────────────────────────
    // Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// System program for potential rent and PDA operations.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageContractGovernance<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller modifying contract settings via governance.
    #[account(mut)]
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────

    /// TokenState account managing the global configuration.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    /// Governance proposal approval needed for the instruction.
    #[account(mut)]
    pub proposal: Account<'info, ProposalAccount>,

    /// Governance state account (quorum config, etc).
    #[account()]
    pub governance_state: Account<'info, GovernanceState>,

    // ─────────────────────────────────────────────────────────────
    // Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // Programs
    // ─────────────────────────────────────────────────────────────

    /// System program for potential rent and PDA operations.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EmitSystemLog<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The caller emitting a system-level log message.
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for log authorization.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // State
    // ─────────────────────────────────────────────────────────────

    /// TokenState account for context verification.
    pub token_state: Account<'info, TokenState>,
}


#[derive(Accounts)]
pub struct ChangeUpdateAuthority<'info> {
    /// The signer initiating the instruction
    pub caller: Signer<'info>,

    /// CHECK: PDA used as both mint_authority and metadata update_authority
    #[account(
        mut,
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// CHECK: PDA for the token mint
    #[account(
        mut,
        seeds = [b"token_mint"],
        bump
    )]
    pub mint: AccountInfo<'info>,

    /// CHECK: Metadata account (PDA derived from [b"metadata", program_id, mint])
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: Token Metadata program (Metaplex)
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,

    /// System program for CPI
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar required by Metaplex CPI (read-only)
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: AccountInfo<'info>,
    
    /// Token state account (used for current config if needed)
    pub token_state: Account<'info, TokenState>,

    /// Sysvar Rent
    pub rent: Sysvar<'info, Rent>,
}


#[cfg(feature = "dev")]
/// MintTokens - development-only context for minting tokens for testing
#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    /// CHECK: This is a PDA used as mint authority
    pub mint_authority: AccountInfo<'info>,

    #[account(mut)]
    pub mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = mint,
        associated_token::authority = recipient
    )]
    pub recipient_token_account: Account<'info, anchor_spl::token::TokenAccount>,

    /// CHECK: Only used for ATA derivation; validation not needed
    pub recipient: AccountInfo<'info>,

    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}
