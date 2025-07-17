// ========================================================================
// Soccial Token – Airdrop Error Definitions
//
// This module defines custom errors related to airdrop campaigns,
// including supply limits, validation of recipient accounts,
// and safety checks for vaults and token transfers.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ========================================================================

use anchor_lang::prelude::*;

/// Custom errors for the Airdrop module
#[error_code]
pub enum AirdropError {
    /// Not enough tokens available in the airdrop supply to fulfill the request.
    #[msg("Insufficient airdrop supply.")]
    InsufficientSupply,

    /// Arithmetic overflow occurred during airdrop calculations.
    #[msg("Arithmetic overflow.")]
    Overflow,

    /// The recipient’s associated token account is invalid or incorrect.
    #[msg("Invalid recipient token account.")]
    InvalidRecipientAccount,

    /// The requested airdrop amount exceeds the per-user limit set by the system.
    #[msg("Requested amount exceeds maximum allowed per airdrop.")]
    ExceedsPerAirdropLimit,

    /// The specified amount is zero or invalid.
    #[msg("Invalid amount.")]
    InvalidAmount,

    /// The airdrop vault does not have enough tokens to complete the operation.
    #[msg("Airdrop Vault does not have enough tokens.")]
    VaultInsufficientBalance,
}
