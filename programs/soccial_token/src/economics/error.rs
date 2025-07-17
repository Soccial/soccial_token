// ========================================================================
// Soccial Token – Economics Error Definitions
//
// This module defines all custom errors related to the tokenomics logic,
// such as fee configuration, distribution boundaries, and economic safety.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ========================================================================

use anchor_lang::prelude::*;

/// EconomicsErrorCode defines custom errors related to economic configuration,
/// such as fee settings, distribution logic, and value constraints.
#[error_code]
pub enum EconomicsErrorCode {
    /// The total of all defined fees (rewards + airdrop) exceeds 100% (10,000 BPS).
    #[msg("Invalid fee distribution. Defined fees exceed 100% (10,000 BPS).")]
    InvalidFeeDistribution,

    /// The provided fee value is outside the allowed range (e.g., below MIN or above MAX).
    #[msg("Invalid fee value. Must be between allowed min and max basis points.")]
    InvalidFeeValue,

    /// An arithmetic overflow occurred during internal fee calculations.
    #[msg("Arithmetic overflow occurred during fee distribution.")]
    Overflow,
}
