use anchor_lang::error_code;

// ======================================================================
// Soccial Token – Market Error Definitions
//
// This module defines custom errors related to token market operations,
// including transactions such as buying, selling, or transferring tokens.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum MarketError {
    /// The provided amount is invalid (e.g., zero or negative).
    #[msg("Invalid amount.")]
    InvalidAmount,
    
    /// The user does not have enough balance to complete the operation.
    #[msg("Insufficient funds.")]
    InsufficientFunds,

    /// The requested fee is above the allowed threshold defined in the economy module.
    #[msg("The specified fee exceeds the maximum allowed limit.")]
    FeeTooHigh,

    /// Arithmetic overflow occurred during a calculation (e.g., addition or multiplication).
    #[msg("Overflow occurred during calculation.")]
    Overflow,

    /// Arithmetic underflow occurred during a calculation (e.g., subtraction below zero).
    #[msg("Underflow occurred during calculation.")]
    Underflow,
}
