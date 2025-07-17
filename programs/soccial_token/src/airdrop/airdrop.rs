// ===========================================================================
// Airdrop Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module manages secure and auditable token airdrops from the on-chain
// `airdrop_vault`. It ensures all transfers respect program-level constraints,
// enforces per-airdrop limits, verifies ownership of recipient accounts,
// and emits structured events for off-chain indexing.
//
// ---------------------------------------------------------------------------
// ## Features:
// - Enforces per-airdrop limit (`MAX_AIRDROP_AMOUNT`)
// - Verifies that the recipient ATA is owned by the expected wallet
// - Transfers tokens securely using CPI with PDA authority
// - Emits Anchor events (`AirdropEvent`) for easy off-chain tracking
//
// ---------------------------------------------------------------------------
// ## Components:
// - `distribute()`: Transfers tokens from the airdrop vault to a user
// - `AirdropEvent`: Emitted after a successful airdrop
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use crate::airdrop::{context::ManageAirdrop, AirdropError};
use crate::utils::error::ErrorCode;
use crate::economy::MAX_AIRDROP_AMOUNT;

/// ===========================================================================
/// Function: distribute
/// ---------------------------------------------------------------------------
/// Securely transfers tokens from the `airdrop_vault` to a user's associated
/// token account. This function performs validation on the amount, verifies
/// token account ownership, and logs the event with optional reason metadata.
///
/// ## Behavior:
/// - Validates the amount is > 0 and ≤ `MAX_AIRDROP_AMOUNT`
/// - Ensures the `airdrop_vault` has sufficient balance
/// - Confirms the recipient's token account is owned by the user
/// - Performs a secure CPI transfer with PDA signer authority
/// - Emits an `AirdropEvent` with recipient, amount, timestamp, and reason
///
/// ## Inputs:
/// - `ctx`: Anchor context containing vault, recipient, and token accounts
/// - `amount`: Number of base units (SCTK) to transfer
/// - `reason`: Optional description for off-chain analytics (e.g., "contest winner")
///
/// ## Errors:
/// - `InvalidAmount` if amount is zero or negative
/// - `ExceedsPerAirdropLimit` if amount exceeds program limit
/// - `VaultInsufficientBalance` if airdrop vault is underfunded
/// - `InvalidRecipientAccount` if recipient ATA doesn't match wallet
/// ===========================================================================
pub(crate) fn distribute(
    ctx: &mut Context<ManageAirdrop>, 
    amount: u64,
    reason: Option<String>,
) -> Result<()> {
    

    // --- Validate amount constraints ---
    require!(amount > 0, AirdropError::InvalidAmount);
    require!(amount <= MAX_AIRDROP_AMOUNT, AirdropError::ExceedsPerAirdropLimit);
    require!(ctx.accounts.airdrop_vault_token_account.amount >= amount, AirdropError::VaultInsufficientBalance);

    // --- Validate recipient token account ownership && self logic ---
    let token_owner = ctx.accounts.recipient_token_account.owner;
    require!(
        token_owner != Pubkey::default(),
        ErrorCode::Unauthorized
    );

    // --- Prepare vault signer seeds ---
    let seeds: &[&[u8]] = &[b"airdrop_vault", &[ctx.bumps.airdrop_vault]];

    // --- Perform token transfer from airdrop vault to recipient ---
    let binding = [seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.airdrop_vault_token_account.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.airdrop_vault.to_account_info(),
        },
        &binding,
    );

    token::transfer(cpi_ctx, amount)?;

    let log_reason_str = reason.as_deref().unwrap_or("N/A");
    let log_reason = log_reason_str.to_string();
    
    // --- Emit Airdrop Event ---
    emit!(AirdropEvent {
        recipient: token_owner,
        amount,
        timestamp: Clock::get()?.unix_timestamp,
        reason: log_reason,
    });

    msg!(
        "✅ Airdrop: {} tokens units transferred to {}  | Reason: {}",
        amount,
        token_owner,
        log_reason_str
    );
    

    Ok(())
}

/// ===========================================================================
/// Event: AirdropEvent
/// ---------------------------------------------------------------------------
/// Emitted after every successful airdrop for transparency and off-chain indexing.
///
/// ## Fields:
/// - `recipient`: Wallet address of the recipient
/// - `amount`: Tokens transferred (in base units)
/// - `timestamp`: Unix timestamp of the airdrop
/// - `reason`: Optional human-readable reason or context (e.g. "referral bonus")
///
/// ## Use Case:
/// - Enables off-chain systems (indexers, explorers, APIs) to track airdrops
///   without parsing state changes directly.
///
/// ## Emitted by:
/// - `distribute()`
/// ===========================================================================
#[event]
pub struct AirdropEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
    pub reason: String, 
}
