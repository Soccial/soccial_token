// ===========================================================================
// TokenState Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module defines the global `TokenState` for the Soccial Token contract,
// including core configuration (owner, status flags, versioning) and fee settings.
//
// ---------------------------------------------------------------------------
// ## Main Components:
// - `TokenState`: Anchor account that stores global contract settings
// - `CoreSettings`: Struct with operational status and ownership info
// - `VersionInfo`: Tracks semantic versioning (major.minor.patch)
//
// ---------------------------------------------------------------------------
// ## Capabilities:
// - Toggle paused state (emergency halt of public ops)
// - Update version and API authority
// - Track initialization status for key systems (economy, SPL, vesting)
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::economics::state::FeeDistribution;

#[event]
pub struct ContractPaused {
    pub caller: Pubkey,
}

#[event]
pub struct ContractResumed {
    pub caller: Pubkey,
}

#[event]
pub struct ApiAuthorityUpdated {
    pub new_authority: Pubkey,
    pub caller: Pubkey,
}

#[event]
pub struct ContractVersionUpdated {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub caller: Pubkey,
}


// ========================================================
// Token State Struct
// ========================================================
#[account]
pub struct TokenState {
    pub core: CoreSettings,
    pub fee: FeeDistribution,
}

impl TokenState {
    /// Returns the total serialized byte size of the `TokenState` account.
    ///
    /// This includes:
    /// - Anchor account discriminator (8 bytes)
    /// - `CoreSettings` struct
    /// - `FeeDistribution` struct
    ///
    /// Used for allocating the correct space when initializing the account.

    pub const LEN: usize =
        8    // Anchor account discriminator
        + CoreSettings::LEN
        + FeeDistribution::LEN;
}

/// Returns the static size (in bytes) of the `VersionInfo` struct.
///
/// Consists of:
/// - `major`: u8
/// - `minor`: u8
/// - `patch`: u8
///
/// Total: 3 bytes

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct VersionInfo {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl VersionInfo {
    pub const LEN: usize = 1 + 1 + 1; // major, minor, patch
}

// ========================================================
// Core general settings
// ========================================================
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct CoreSettings {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub api_authority: Pubkey,
    pub initialized: bool,
    pub economy_initialized: bool,
    pub spl_initialized: bool,
    pub team_vesting_initialized: bool,
    pub paused: bool,
    pub version: VersionInfo,
}

impl CoreSettings {
    /// Returns the total serialized byte size of the `CoreSettings` struct.
    ///
    /// This includes:
    /// - `owner` (Pubkey) – 32 bytes  
    /// - `api_authority` (Pubkey) – 32 bytes  
    /// - 5 booleans (1 byte each)  
    /// - `version` (3 bytes)
    ///
    /// Total: 70 bytes
    pub const LEN: usize =
        32 + // owner
        32 + // mint
        32 + // api_authority
        1  + // initialized
        1  + // economy_initialized
        1  + // spl_initialized
        1  + // team_vesting_initialized 
        1  + // paused
        VersionInfo::LEN; // version (major + minor + patch)

    /// Pauses the entire contract, disabling public entrypoints.
    ///
    /// This sets the `paused` flag to `true`, which is checked in most
    /// sensitive entrypoints to prevent unauthorized access during maintenance,
    /// upgrades, or security incidents.
    ///
    /// # Parameters
    /// - `caller`: The public key of the entity that paused the contract
    ///
    /// # Emits
    /// Logs a message indicating the contract has been paused.
    pub(crate) fn pause(&mut self, caller: Pubkey) {
        self.paused = true;
        msg!("🚨 Contract has been paused by {}. Public operations are disabled.", caller);

        emit!(ContractPaused {
            caller,
        });
    }

    /// Resumes the contract by re-enabling public entrypoints.
    ///
    /// This clears the `paused` flag, allowing operations to continue normally.
    ///
    /// # Parameters
    /// - `caller`: The public key of the entity that resumed the contract
    ///
    /// # Emits
    /// Logs a message indicating the contract has been resumed.
    pub(crate) fn resume(&mut self, caller: Pubkey) {
        self.paused = false;
        msg!("✅ Contract has been resumed by {}. Public operations are enabled.", caller);

        emit!(ContractResumed {
            caller,
        });
    }

    /// Updates the API authority to a new public key.
    ///
    /// This authority is allowed to execute privileged automated actions,
    /// such as off-chain job execution or integration with backend services.
    ///
    /// # Parameters
    /// - `new_authority`: The new public key to set as API authority
    /// - `caller`: The public key of the entity performing the change
    ///
    /// # Emits
    /// A log with the updated authority and actor
    pub(crate) fn set_api_authority(&mut self, new_authority: Pubkey, caller: Pubkey) {
        self.api_authority = new_authority;
        msg!("✅ API authority updated to: {} (by {})", new_authority, caller);

        emit!(ApiAuthorityUpdated {
            new_authority,
            caller,
        });
    }

    /// Updates the contract version (major, minor, patch).
    ///
    /// This function updates the `version` field in `CoreSettings` to reflect
    /// the latest deployed contract version. Useful for tracking deployments
    /// or compatibility between versions.
    ///
    /// # Parameters
    /// - `major`: Major version (breaking changes)
    /// - `minor`: Minor version (feature updates)
    /// - `patch`: Patch version (bug fixes)
    /// - `caller`: Public key of the entity performing the update
    ///
    /// # Emits
    /// A log showing the new version and who triggered the update.
    pub(crate) fn update_version(&mut self, major: u8, minor: u8, patch: u8, caller: Pubkey) {
        self.version = VersionInfo { major, minor, patch };
        msg!(
            "✅ Soccial Version updated to: {}.{}.{} (by {})",
            major, minor, patch, caller
        );

        emit!(ContractVersionUpdated {
            version_major: major,
            version_minor: minor,
            version_patch: patch,
            caller,
        });
    }
}