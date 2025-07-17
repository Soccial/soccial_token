// ===========================================================================
// Math Utilities Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module provides safe conversion and formatting helpers for the SCTK token,
// handling precision based on the global `TOKEN_DECIMAL` setting (typically 9).
//
// ---------------------------------------------------------------------------
// ## Functions:
// - `units_to_tokens()` → Converts base units (u64) into whole tokens
// - `tokens_to_units()` → Converts whole tokens into base units
// - `format_sctk()`     → Formats raw token amounts into display strings
//
// ---------------------------------------------------------------------------
// ## Error Handling:
// All arithmetic operations use `checked_*` and return `MathErrorCode::Overflow` on failure.
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::economy::TOKEN_DECIMAL;


/// ===========================================================================
/// Converts base units (u64) into human-readable token value.
///
/// # Example
/// - Input: 12_000_000_000 with 9 decimals → Output: 12
///
/// # Arguments
/// * `units` - Token amount in base units.
///
/// # Returns
/// `Ok(u64)` - Token amount as whole tokens.
/// 
/// ===========================================================================
pub fn units_to_tokens(units: u64) -> Result<u64> {
    let divisor = 10u64
        .checked_pow(TOKEN_DECIMAL as u32)
        .ok_or(MathErrorCode::Overflow)?;

    let tokens = units
        .checked_div(divisor)
        .ok_or(MathErrorCode::Overflow)?;

    Ok(tokens)
}

/// ===========================================================================
/// Converts whole tokens into raw base units for storage or transfer.
///
/// # Example
/// - Input: 12 with 9 decimals → Output: 12_000_000_000
///
/// # Arguments
/// * `tokens` - Token amount as a whole number.
///
/// # Returns
/// `Ok(u64)` - Token amount in base units.
/// ===========================================================================
pub fn tokens_to_units(tokens: u64) -> Result<u64> {
    let multiplier = 10u64
        .checked_pow(TOKEN_DECIMAL as u32)
        .ok_or(MathErrorCode::Overflow)?;

    let units = tokens
        .checked_mul(multiplier)
        .ok_or(MathErrorCode::Overflow)?;

    Ok(units)
}


/// ===========================================================================
/// Formats a raw base-unit token amount into a human-readable string.
///
/// Trims trailing zeroes after the decimal point if present.
///
/// # Examples
/// - Input: 12_000_000_000 → Output: `"12"`  
/// - Input: 123_456_789   → Output: `"0.123456789"`  
/// - Input: 12_005_000_000 → Output: `"12.005"`
///
/// # Arguments
/// * `amount` - Raw SCTK amount in base units.
///
/// # Returns
/// Formatted `String` suitable for UI display.
/// 
/// ===========================================================================
pub fn format_sctk(amount: u64) -> String {
    let divisor = 10u64.pow(TOKEN_DECIMAL as u32);
    let whole = amount / divisor;
    let fraction = amount % divisor;

    if fraction == 0 {
        return format!("{}", whole);
    }

    let fraction_str = format!("{:0width$}", fraction, width = TOKEN_DECIMAL as usize)
        .trim_end_matches('0')
        .to_string();

    format!("{}.{}", whole, fraction_str)
}

/// ----------------------------------------------------------------------------
/// Math Error Codes
/// ----------------------------------------------------------------------------

/// Custom math-related errors used across conversion utilities.
#[error_code]
pub enum MathErrorCode {
    #[msg("Arithmetic overflow occurred during calculation.")]
    Overflow,
}
