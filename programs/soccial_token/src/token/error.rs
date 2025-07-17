use anchor_lang::prelude::*;

// ======================================================================
// Soccial Token – Token Error Definitions
//
// This module defines error codes related to token-level operations,
// including supply validation, balance checks, and access control.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum TokenError {
    /// Only the designated token owner can perform this action.
    #[msg("Only the token owner can perform this action.")]
    Unauthorized,

    /// Not enough reserved supply available to proceed.
    #[msg("Insufficient reserved supply for this operation.")]
    InsufficientSupply,

    /// The token balance is too low to execute the requested transfer.
    #[msg("Insufficient balance to perform this transfer.")]
    InsufficientBalance,

    /// An arithmetic overflow occurred.
    #[msg("Overflow occurred during math operation.")]
    Overflow,

    /// Total token supply would exceed the capped maximum.
    #[msg("Total supply exceeded.")]
    MaxSupplyExceeded,
}
