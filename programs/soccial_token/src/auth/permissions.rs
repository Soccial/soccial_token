// ===========================================================================
// PermissionMap Module – Named Access Control Bitmask
// ---------------------------------------------------------------------------
//
// This module defines the mapping between human-readable permission names
// (e.g., "manage_contract", "stake_tokens") and their associated bit
// positions in a `u32` bitfield.
//
// Design:
// - Each permission is assigned a **unique bit (0–31)**.
// - Permissions are compactly stored in the `UserAccessAccount` using the lower 48 bits.
// - Lookups are done via `PermissionMap::get_bit("permission_name")`.
//
// Motivation:
// - Enables fine-grained, role-based access control across the smart contract.
// - Ensures low-cost computation using bitwise operations instead of string comparisons.
//
// Usage:
// if user_access.has_permission("manage_vaults") {
//     // allow vault management
// }
//
// Total Capacity:
// - Supports up to 32 named permissions in `u32`.
// - Flags (like Staff or EarlyAdopter) are stored separately in upper bits (see `ExtraFlag`).
//
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

pub struct PermissionMap;

/// Returns the bit position associated with a permission name.
///
/// # Arguments
/// - `name`: The name of the permission (e.g., `"manage_contract"`, `"stake_tokens"`).
///
/// # Returns
/// - `Some(bit)` if the permission name is known.
/// - `None` if the permission name is invalid or unregistered.
///
/// # Use Case
/// Used to dynamically resolve permission names during runtime checks or admin updates.
///
/// # Example

/// assert_eq!(PermissionMap::get_bit("manage_contract"), Some(0));
/// assert_eq!(PermissionMap::get_bit("invalid_permission"), None);

impl PermissionMap {
    pub(crate) fn get_bit(name: &str) -> Option<u8> {
        match name {
            // ─────────────────────
            // Contract Management
            // ─────────────────────
            "manage_contract"          => Some(0),
            "set_api_authority"        => Some(1),
            "emit_log"                 => Some(2),
            "bypass_contract_pause"    => Some(3),

            // ─────────────────────
            // User & Permissions & Admin
            // ─────────────────────
            "manage_admin"             => Some(4),
            "manage_permissions"       => Some(5),
            "manage_user"              => Some(6),

            // ─────────────────────
            // Vaults
            // ─────────────────────
            "deposit_vaults"           => Some(7),
            "transfer_between_vaults"  => Some(8),
            "manage_vaults"            => Some(9),

            // ─────────────────────
            // Token Economy
            // ─────────────────────
            "manage_economy"           => Some(10),
            "airdrop_tokens"           => Some(11),

            // ─────────────────────
            // Staking
            // ─────────────────────
            "stake_tokens"             => Some(12),
            "withdraw_staked_tokens"  => Some(13),
            "manage_staking"          => Some(14),

            // ─────────────────────
            // Vesting
            // ─────────────────────
            "create_vesting"          => Some(15),
            "update_vesting"          => Some(16),
            "manage_vesting"          => Some(17),

            // ─────────────────────
            // Governance
            // ─────────────────────
            "create_proposal"         => Some(18),
            "finalize_proposal"       => Some(19),

            // ─────────────────────
            // Market
            // ─────────────────────
            "buy_tokens"              => Some(20),
            "transfer_tokens"         => Some(21),
            "deposit_tokens"          => Some(22),

            _ => None,
        }
    }
}
