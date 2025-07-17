// ======================================================================
// Soccial Token – Airdrop Contexts
//
// This file defines the account context required to perform token
// airdrop operations, including authority checks, vault validation,
// and associated token account handling.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{auth::user::UserAccessAccount, token::TokenState};

#[derive(Accounts)]
pub struct ManageAirdrop<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller performing the airdrop action (must be authorized).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// User access control (optional). Used for permission checks and rate limiting.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Token State & Mint
    // =========================================================================

    /// The TokenState account, holding global token configuration and airdrop supply.
    #[account(
        seeds = [b"token_state"],
        bump,
    )]
    pub token_state: Account<'info, TokenState>,

    /// CHECK: The Token Mint PDA. Must be manually verified in the handler.
    #[account(
        seeds = [b"token_mint"],
        bump,
    )]
    pub mint: Account<'info, Mint>,

    // =========================================================================
    // Airdrop Vault (Source of tokens)
    // =========================================================================

    /// CHECK: The Airdrop Vault PDA. Must be manually verified in the handler.
    #[account(
        seeds = [b"airdrop_vault"],
        bump,
    )]
    pub airdrop_vault: AccountInfo<'info>,

    /// The Airdrop Vault's ATA holding the tokens to distribute.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = airdrop_vault,
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Recipient Info
    // =========================================================================

    /// The recipient's token account (destination of the airdrop).
    #[account(
        mut,
        token::mint = mint
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Program Dependencies
    // =========================================================================

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}

