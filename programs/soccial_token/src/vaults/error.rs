use anchor_lang::prelude::*;

// ======================================================================
// Soccial Token – Vault Error Definitions
//
// This module defines all custom error codes related to Vault operations,
// including initialization, access control, transfers, and integrity checks.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum VaultError {
    /// The specified vault has not yet been initialized.
    #[msg("The vault has not been initialized yet.")]
    VaultNotInitialized,

    /// Attempted to initialize a vault that is already set up.
    #[msg("The vault is already initialized.")]
    VaultAlreadyInitialized,

    /// Caller lacks permission or authority to perform this vault operation.
    #[msg("Unauthorized operation on the vault.")]
    UnauthorizedVaultAccess,

    /// The vault does not contain enough tokens for this withdrawal or transfer.
    #[msg("Insufficient vault balance for the requested operation.")]
    InsufficientVaultBalance,

    /// The amount specified is invalid (zero or negative).
    #[msg("Invalid amount specified. Amount must be greater than zero.")]
    InvalidVaultAmount,

    /// The vault type provided is not recognized or supported.
    #[msg("Unknown vault type specified.")]
    UnknownVaultType,

    /// A critical internal invariant has been violated (e.g., misalignment in balances).
    #[msg("Internal vault invariant violation.")]
    VaultInvariantViolation,

    /// It is not allowed to transfer tokens from a vault to itself.
    #[msg("Cannot transfer from a vault to itself.")]
    InvalidItselfVaultTransfer,

    // Transfer between the specified vaults is not allowed by the current policy.
    #[msg("Transfer between these vaults is not allowed.")]
    UnauthorizedVaultTransfer,

     /// This action requires a governance proposal to be approved.
    #[msg("This action requires a valid and approved governance proposal.")]
    MissingProposalApproval,

}