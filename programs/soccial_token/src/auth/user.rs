// ===========================================================================
// User Access & Permissions Management – Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module defines the user-level access control for the Soccial Token ecosystem.
// It compactly stores both **permission bits** and **status flags** inside a single `u64` field,
// allowing highly efficient checks, updates, and serialization in on-chain programs.
//
// ## Features:
// - Bit-level storage: 48 bits for permissions, 16 bits for status flags
// - Admin flag separate from bitfield for fast lookup
// - Flexible permission assignment via string-based map
// - Enum-based flags with friendly labels and strict validation
//
// ## // - ⚡ Performance: single-word storage (u64) for fast access
// - 💾 Compact: avoids padding and extra serialization cost
// - 🔒 Secure: explicit checks and error codes for all mutations
//
// ## Components:
// - `ExtraFlag`: Enum representing user status (e.g., banned, early adopter)
// - `UserAccessAccount`: Anchor account storing access control data
// - `add_flag`, `assign_permission`, etc.: Utility functions to mutate state safely
//
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::auth::permissions;
use super::UserErrorCode;

#[event]
pub struct FlagAdded {
    pub caller: Pubkey,
    pub target_user: Pubkey,
    pub flag: String,
}

#[event]
pub struct FlagRemoved {
    pub caller: Pubkey,
    pub target_user: Pubkey,
    pub flag: String,
}

#[event]
pub struct PermissionGranted {
    pub caller: Pubkey,
    pub target_user: Pubkey,
    pub permission: String,
}

#[event]
pub struct PermissionRevoked {
    pub caller: Pubkey,
    pub target_user: Pubkey,
    pub permission: String,
}

#[event]
pub struct AdminGranted {
    pub caller: Pubkey,
    pub target_user: Pubkey,
}

#[event]
pub struct AdminRevoked {
    pub caller: Pubkey,
    pub target_user: Pubkey,
}

/// ==========================
/// Enum: ExtraFlag
/// --------------------------
/// Represents special status flags for a user, stored in the upper 16 bits (bits 48–63)
/// of the unified access_bits field in UserAccessAccount.
///
/// ## Examples:
/// - `Banned` blocks the user from all actions
/// - `EarlyAdopter1` grants voting or economic bonuses
/// - `Staff` may restrict actions like voting or access
///
/// Methods include:
/// - `bit_position()`: Returns the absolute bit index
/// - `label()`: Human-readable log when set
/// - `removal_label()`: Log message when unset
/// - `from_str()`: Resolves from case-insensitive name
/// ==========================

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExtraFlag {
    Banned = 0,
    Suspended = 1,
    EarlyAdopter1 = 2,
    EarlyAdopter2 = 3,
    Whitelist = 4,
    Vip = 5,
    BetaTester = 6,
    Staff = 7,
}

impl ExtraFlag {
    /// Offset for flags starts after the permissions (first 48 bits are for permissions).
    pub(crate) fn bit_position(self) -> u8 {
        48 + (self as u8)
    }

    /// Friendly message displayed when setting a flag.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            ExtraFlag::Banned => "🚫 User has been banned.",
            ExtraFlag::Suspended => "⚠️ User account has been suspended.",
            ExtraFlag::EarlyAdopter1 => "🎉 User registered as Early Adopter Phase 1.",
            ExtraFlag::EarlyAdopter2 => "🎉 User registered as Early Adopter Phase 2.",
            ExtraFlag::Whitelist => "✅ User added to the whitelist.",
            ExtraFlag::Vip => "🌟 User granted VIP status.",
            ExtraFlag::BetaTester => "🧪 User registered as Beta Tester.",
            ExtraFlag::Staff => "🛠️ User added as Staff member.",
        }
    }

    /// Friendly message displayed when unsetting a flag.
    pub(crate) fn removal_label(&self) -> &'static str {
        match self {
            ExtraFlag::Banned => "✅ User ban has been lifted.",
            ExtraFlag::Suspended => "✅ User suspension has been lifted.",
            ExtraFlag::EarlyAdopter1 => "❌ User removed from Early Adopter Phase 1.",
            ExtraFlag::EarlyAdopter2 => "❌ User removed from Early Adopter Phase 2.",
            ExtraFlag::Whitelist => "❌ User removed from the whitelist.",
            ExtraFlag::Vip => "❌ VIP status revoked.",
            ExtraFlag::BetaTester => "❌ Beta Tester status removed.",
            ExtraFlag::Staff => "❌ Staff status removed.",
        }
    }

    pub(crate) fn from_str(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "banned" => Ok(ExtraFlag::Banned),
            "suspended" => Ok(ExtraFlag::Suspended),
            "earlyadopter1" | "early1" => Ok(ExtraFlag::EarlyAdopter1),
            "earlyadopter2" | "early2" => Ok(ExtraFlag::EarlyAdopter2),
            "whitelist" => Ok(ExtraFlag::Whitelist),
            "vip" => Ok(ExtraFlag::Vip),
            "betatester" | "beta" => Ok(ExtraFlag::BetaTester),
            "staff" => Ok(ExtraFlag::Staff),
            _ => Err(UserErrorCode::UnknownFlagName.into()),
        }
    }
}

/// ==========================
/// Account: UserAccessAccount
/// --------------------------
/// Stores both **permissions** and **extra flags** in a compact `u64` field called `access_bits`.
/// Also includes a boolean flag for admin status (`is_admin`).
///
/// ## Layout:
/// - Bits 0–47: Permissions
/// - Bits 48–63: Flags (ExtraFlag)
///
/// ## Field Summary:
/// - `is_admin`: Direct admin role for fast checking
/// - `access_bits`: Encoded permission + flags
/// - `bump`: PDA bump seed for Anchor
///
/// This design enables fast reads/writes while minimizing storage cost and compute usage.
/// ==========================
#[account]
pub struct UserAccessAccount {
    pub is_admin: bool,    // True if the user is an admin
    pub access_bits: u64,  // Lower 48 bits = permissions, Upper bits = flags
    pub bump: u8,          // PDA bump seed
}

impl UserAccessAccount {
    // -------- FLAGS MANAGEMENT -------- //

    /// Adds a specific ExtraFlag to the user's access_bits.
    ///
    /// ## Behavior:
    /// - Computes the bit position (48+X)
    /// - Fails if flag is already set
    /// - Logs a label message upon success
    ///
    /// ## Errors:
    /// - `FlagAlreadySet` if bit is already present
    pub fn add_flag(&mut self, flag: ExtraFlag, caller: Pubkey, target_user: Pubkey, flag_name: &str) -> Result<()> {
        let bit = flag.bit_position();
        let mask = 1u64 << bit;
        require!(self.access_bits & mask == 0, UserErrorCode::FlagAlreadySet);
        self.access_bits |= mask;
        msg!("✅ {}", flag.label());

        emit!(FlagAdded {
            caller,
            target_user,
            flag: flag_name.to_string(),
        });
       
        Ok(())
    }

    /// Removes an ExtraFlag from the user's access_bits.
    ///
    /// ## Behavior:
    /// - Validates presence before removal
    /// - Logs a user-friendly message
    ///
    /// ## Errors:
    /// - `FlagNotSet` if the bit is not present
    pub fn remove_flag(&mut self, flag: ExtraFlag, caller: Pubkey, target_user: Pubkey, flag_name: &str) -> Result<()> {
        let bit = flag.bit_position();
        let mask = 1u64 << bit;
        require!(self.access_bits & mask != 0, UserErrorCode::FlagNotSet);
        self.access_bits &= !mask;
        msg!("✅ {}", flag.removal_label());

        emit!(FlagRemoved {
            caller,
            target_user,
            flag: flag_name.to_string(),
        });

        Ok(())
    }

    /// Returns `true` if the specified ExtraFlag is currently set.
    pub fn has_flag(&self, flag: ExtraFlag) -> bool {
        let bit = flag.bit_position();
        (self.access_bits & (1u64 << bit)) != 0
    }

    // -------- PERMISSIONS MANAGEMENT -------- //

    /// Grants a named permission to the user.
    ///
    /// ## Behavior:
    /// - Resolves bit position via PermissionMap
    /// - Mutates access_bits accordingly
    ///
    /// ## Errors:
    /// - `UnknownPermissionName` if the name is invalid
    /// - `PermissionAlreadySet` if already present
    pub fn assign_permission(&mut self, name: &str, caller: Pubkey, target_user: Pubkey, permission_name: &str) -> Result<()> {
        if let Some(bit) = permissions::PermissionMap::get_bit(name) {
            let mask = 1u64 << bit;
            require!(self.access_bits & mask == 0, UserErrorCode::PermissionAlreadySet);
            self.access_bits |= mask;
            msg!("✅ Permission '{}' granted.", name);

            emit!(PermissionGranted {
                caller,
                target_user,
                permission: permission_name.to_string(),
            });

            Ok(())
        } else {
            Err(UserErrorCode::UnknownPermissionName.into())
        }
    }

    /// Revokes a previously granted permission.
    ///
    /// ## Behavior:
    /// - Resolves bit and clears it from access_bits
    ///
    /// ## Errors:
    /// - `UnknownPermissionName`
    /// - `PermissionNotSet` if the permission is not granted
    pub fn unsign_permission(&mut self, name: &str, caller: Pubkey, target_user: Pubkey, permission_name: &str) -> Result<()> {
        if let Some(bit) = permissions::PermissionMap::get_bit(name) {
            let mask = 1u64 << bit;
            require!(self.access_bits & mask != 0, UserErrorCode::PermissionNotSet);
            self.access_bits &= !mask;
            msg!("✅ Permission '{}' revoked.", name);

            emit!(PermissionRevoked {
                caller,
                target_user,
                permission: permission_name.to_string(),
            }); 

            Ok(())
        } else {
            Err(UserErrorCode::UnknownPermissionName.into())
        }
    }

    /// Returns `true` if the user has the specified named permission.
    ///
    /// ## Behavior:
    /// - Resolved via `PermissionMap::get_bit`
    /// - Returns `false` if the name is unknown or bit is unset
    pub fn has_permission(&self, name: &str) -> bool {
        if let Some(bit) = permissions::PermissionMap::get_bit(name) {
            (self.access_bits & (1u64 << bit)) != 0
        } else {
            false
        }
    }

      // -------- ADMIN MANAGEMENT -------- //

    /// Grants administrative status to the user.
    ///
    /// ## Errors:
    /// - `AdminAlreadySet` if already an admin

    pub fn grant_admin(&mut self, caller: Pubkey, target_user: Pubkey) -> Result<()> {

        require!(!self.is_admin, UserErrorCode::AdminAlreadySet);

        self.is_admin = true;
        msg!("✅ Admin granted.");

        emit!(AdminGranted {
            caller,
            target_user,
        });

        Ok(())
    }

    /// Revokes administrative status.
    ///
    /// ## Errors:
    /// - `AdminNotSet` if not currently an admin
    pub fn revoke_admin(&mut self, caller: Pubkey, target_user: Pubkey) -> Result<()> {
        require!(self.is_admin, UserErrorCode::AdminNotSet);
        
        self.is_admin = false;
        msg!("✅ Admin rights revoked.");

        emit!(AdminRevoked {
            caller,
            target_user,
        });

        Ok(())
    }

}
