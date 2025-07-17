// ======================================================================
// Soccial Token – Vault Contexts
//
// This file defines account contexts for managing token vault operations,
// including deposits, withdrawals, vault-to-vault transfers, and contract-to-vault moves.
//
// Each context ensures correct PDA validations, access control, and
// secure token handling within the Soccial Token ecosystem.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::{auth::user::UserAccessAccount, governance::{GovernanceState, ProposalAccount}, token::state::TokenState};

// ======================================================================
// Vault Type Enum
// ======================================================================
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultType {
    OffchainReserve,
    Revenue,
    Rewards,
    Airdrop,
    Buyback,
    Vesting,
    Insurance,
    Treasury,
    ReservedSupply,
}

#[derive(Accounts)]
pub struct VaultDepositContext<'info> {
    // =========================================================================
    // Caller & Participant
    // =========================================================================

    /// The user initiating the deposit (must be authorized).
    pub caller: Signer<'info>,

    /// CHECK: The wallet or PDA of the participant making the deposit.
    #[account(mut, signer)]
    pub participant: AccountInfo<'info>,

    // =========================================================================
    // Token Accounts
    // =========================================================================

    /// Participant's ATA holding SCTK tokens to deposit.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = participant
    )]
    pub participant_token_account: Account<'info, TokenAccount>,

    /// Vault's ATA that will receive the deposited tokens.
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Vault PDA
    // =========================================================================

    /// CHECK: Vault metadata account (PDA). Must be validated in logic.
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// CHECK: Vault authority PDA (must match seeds in logic).
    pub vault_authority: AccountInfo<'info>,

    // =========================================================================
    // Token State & Mint
    // =========================================================================

    /// The SCTK token mint.
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    /// Global TokenState configuration account.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Access Control
    // =========================================================================

    /// Optional access control for participant permissions.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Program Dependencies
    // =========================================================================

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct VaultWithdrawContext<'info> {
    // =========================================================================
    // Caller
    // =========================================================================

    /// The user initiating the withdrawal (must be authorized).
    pub caller: Signer<'info>,

    // =========================================================================
    // Token Accounts
    // =========================================================================

    /// Vault's ATA holding tokens to be withdrawn.
    #[account(mut)]
    pub vault_token_account: Account<'info, TokenAccount>,

    /// User's ATA to receive withdrawn tokens.
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Vault PDA
    // =========================================================================

    /// CHECK: Vault metadata account (PDA). Must be validated in logic.
    #[account(mut)]
    pub vault: AccountInfo<'info>,

    /// CHECK: Vault authority PDA (must match seeds in logic).
    pub vault_authority: AccountInfo<'info>,

    // =========================================================================
    // Token State
    // =========================================================================

    /// Global TokenState configuration account.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Access Control
    // =========================================================================

    /// Optional access control for withdrawal permission checks.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Program Dependencies
    // =========================================================================

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct VaultTransferContext<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The user triggering the vault-to-vault transfer (must be authorized).
    pub caller: Signer<'info>,

    /// Optional access control for transfer permission checks.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Token Accounts
    // =========================================================================

    /// Source vault's ATA holding the tokens to transfer.
    #[account(mut)]
    pub source_vault_token_account: Account<'info, TokenAccount>,

    /// Destination vault's ATA that will receive the tokens.
    #[account(mut)]
    pub destination_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Vault PDAs
    // =========================================================================

    /// CHECK: Metadata account for the source vault. Must be validated in logic.
    pub source_vault: AccountInfo<'info>,

    /// CHECK: Authority PDA for the source vault.
    pub source_vault_authority: AccountInfo<'info>,

    // =========================================================================
    // Governance & Token State
    // =========================================================================

    /// Optional proposal linked to this vault transfer.
    #[account(mut)]
    pub proposal: Option<Account<'info, ProposalAccount>>,

    /// Global governance configuration account.
    pub governance_state: Account<'info, GovernanceState>,

    /// Global TokenState configuration account.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Program Dependencies
    // =========================================================================

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct ContractToVaultContext<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The user triggering the contract-to-vault transfer.
    pub caller: Signer<'info>,

    /// Optional access control for permission checks.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Token Accounts
    // =========================================================================

    /// The contract-owned source token account (fallback token pool).
    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,

    /// Destination vault's ATA to receive tokens.
    #[account(mut)]
    pub destination_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Vault & Authority
    // =========================================================================

    /// CHECK: PDA authority of the contract (must match seeds).
    #[account(
        mut,
        seeds = [b"contract_token_owner"],
        bump
    )]
    pub source_authority: AccountInfo<'info>,

    /// CHECK: Metadata PDA of the destination vault.
    #[account(mut)]
    pub destination_vault: AccountInfo<'info>,

    // =========================================================================
    // Token State & Program
    // =========================================================================

    /// Global TokenState configuration account.
    pub token_state: Account<'info, TokenState>,

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}

