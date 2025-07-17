use anchor_lang::prelude::*;

// ======================================================================
// Soccial Token – General Error Definitions
//
// This module defines the core error codes used across the Soccial Token
// smart contract, including common access control errors, argument issues,
// initialization checks, and cross-module validations.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum ErrorCode {
    // ────────────────────────────────────────────────────────────────
    // Access & Permission Errors
    // ────────────────────────────────────────────────────────────────

    /// Caller lacks required permission or authority.
    #[msg("Unauthorized access. You do not have permission.")]
    Unauthorized,

    /// The caller must be the initial contract owner to perform this action.
    #[msg("Caller is not the initial owner.")]
    UnauthorizedInitialOwner,

    /// Attempted to remove the contract owner, which is not allowed.
    #[msg("The owner cannot be removed.")]
    OwnerCannotBeRemoved,

    // ────────────────────────────────────────────────────────────────
    // Execution & Environment
    // ────────────────────────────────────────────────────────────────

    /// The specified function does not exist or is disabled.
    #[msg("Function not found.")]
    FunctionNotFound,

    /// The contract is currently paused.
    #[msg("The contract is currently paused.")]
    ContractPaused,

    /// Too many requests in a short period — try again later.
    #[msg("Rate limit exceeded, please try again later.")]
    RateLimitExceeded,

    // ────────────────────────────────────────────────────────────────
    // Argument & Parameter Errors
    // ────────────────────────────────────────────────────────────────

    /// A required argument was missing.
    #[msg("Missing argument.")]
    MissingArgument,

    /// The provided argument is invalid or malformed.
    #[msg("Invalid argument.")]
    InvalidArgument,

    /// Not enough arguments were provided to execute the operation.
    #[msg("Not enough arguments provided.")]
    NotEnoughArguments,

    /// The permission string or key is invalid.
    #[msg("Invalid permission.")]
    InvalidPermission,

    // ────────────────────────────────────────────────────────────────
    // Platform & Token Initialization
    // ────────────────────────────────────────────────────────────────

    /// Platform mismatch occurred between caller and expected source.
    #[msg("Platform mismatch.")]
    PlatformMismatch,

    /// The platform specified is not supported.
    #[msg("Unsupported platform.")]
    UnsupportedPlatform,

    /// The token state account has not yet been initialized.
    #[msg("Token is not initialized.")]
    TokenNotInitialized,

    /// The token economy (governance, fee system) is not initialized.
    #[msg("Token Economy must be initialized first.")]
    TokenEconomyNotInitialized,

    /// SPL token mint must be initialized before use.
    #[msg("Token SPL must be initialized first.")]
    SplNotInitialized,

    // ────────────────────────────────────────────────────────────────
    // Token & Account Management
    // ────────────────────────────────────────────────────────────────

    /// The provided account is invalid or does not meet constraints.
    #[msg("Invalid account.")]
    InvalidAccount,

    /// No tokens are available to release for this operation.
    #[msg("No tokens to release.")]
    NoTokensToRelease,
}
