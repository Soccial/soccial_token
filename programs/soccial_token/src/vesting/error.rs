// ========================================================================
// Soccial Token – Vesting Error Definitions
//
// This module defines all custom error codes related to the Vesting system,
// including validation of durations, immutability, and liquidity checks.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ========================================================================

use anchor_lang::prelude::*;

#[error_code]
pub enum VestingErrorCode {
    /// No vested tokens are currently available to claim.
    #[msg("No tokens available to release.")]
    NoTokensToRelease,

    /// An invalid argument was provided (e.g., negative duration or mismatched types).
    #[msg("Invalid argument.")]
    InvalidArgument,

    /// Vesting schedule is already initialized for the given vesting_id and participant.
    #[msg("Vesting schedule is already initialized.")]
    AlreadyInitialized,

    /// The vesting schedule is not active (e.g., was cancelled or never activated).
    #[msg("Vesting schedule is not active.")]
    VestingNotActive,

    /// The configured start time is invalid (e.g., in the past or unaligned).
    #[msg("Invalid start time.")]
    InvalidStartTime,

    /// The duration provided for vesting is invalid or too short.
    #[msg("Invalid vesting duration.")]
    InvalidVestingDuration,

    /// The cliff period is longer than the total vesting period, which is not allowed.
    #[msg("Cliff duration cannot exceed total vesting duration.")]
    CliffExceedsVesting,

    /// The cliff duration value is invalid (e.g., negative).
    #[msg("Invalid cliff duration.")]
    InvalidCliff,

    /// Vesting token amount must be strictly greater than zero.
    #[msg("Token amount must be greater than zero.")]
    InvalidTokenAmount,

    /// The vesting schedule has already been cancelled.
    #[msg("Vesting schedule has already been cancelled.")]
    AlreadyCancelled,

    /// This vesting schedule is immutable and cannot be edited.
    #[msg("This vesting schedule is immutable and cannot be edited.")]
    VestingScheduleIsImmutable,

    /// Liquidity vault does not have enough available tokens to allocate to this vesting.
    #[msg("Insufficient available funds in the liquidity vault to create the vesting schedule.")]
    InsufficientFunds,
}
