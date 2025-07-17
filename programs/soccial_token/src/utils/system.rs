// ===========================================================================
// System Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module implements the **core system-level logic** for the Soccial Token smart contract.
//
// It is responsible for:
// - Determining the caller’s **role** (Owner, Admin, API, User)
// - Enforcing **contract-wide paused state**
// - Checking for **explicit user permissions**
//
// All entry points in the contract invoke the `check_core` function to **establish the caller context**,  
// ensuring **centralized access control** and consistent **security checks** throughout the program.
//
// ---------------------------------------------------------------------------
// ## Compact Permissions & Flags
//
// The system uses the `UserAccessAccount`, which compactly stores:
// - **48 permission bits** (0–47)
// - **16 flag bits** (48–63)
//
// This compact format minimizes storage and compute usage, and makes permission lookups extremely efficient.
//
// ---------------------------------------------------------------------------
// ## Components:
// - `UserRole`: Enum identifying the caller’s effective role
// - `UserContext`: Wrapper that abstracts the role and capabilities of the caller
// - `check_core()`: Assigns role, enforces paused state
// - `check_owner()`: Confirms caller is contract owner
// - `verify_permission()`: Checks if a user has a specific permission
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::{
    token::TokenState,
    auth::user::UserAccessAccount,
    utils::error::ErrorCode,
};
/// ===========================================================================
/// Enum: UserRole
/// ---------------------------------------------------------------------------
///
/// Identifies the logical access level of the caller, based on:
/// - Ownership
/// - Admin status
/// - Explicit assignment as API authority
/// ===========================================================================
#[derive(Debug, PartialEq, Eq)]
pub enum UserRole {
    Owner,
    Admin,
    Api,
    User,
}

/// ===========================================================================
/// Struct: UserContext
/// ---------------------------------------------------------------------------
///
/// Lightweight abstraction over user role for use in permission checks.
///
pub struct UserContext {
    pub role: UserRole,
}

impl UserContext {
    /// Returns `true` if the caller is the contract owner.
    pub(crate) fn is_owner(&self) -> bool {
        self.role == UserRole::Owner
    }

    /// Returns `true` if the caller is an admin or the owner.
    pub(crate) fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Owner | UserRole::Admin)
    }

    /// Returns `true` if the caller is the API authority.
    pub(crate) fn is_api(&self) -> bool {
        self.role == UserRole::Api
    }
}

/// ===========================================================================
/// Function: check_core – Verifies Role and Paused Status
///
/// Determines the effective role of the caller and enforces **contract-level restrictions**
/// such as the `paused` state.
///
/// ## Role Resolution:
/// - If `caller == owner`         → `UserRole::Owner`
/// - If `caller == api_authority` → `UserRole::Api`
/// - If `user_access.is_admin`    → `UserRole::Admin`
/// - Otherwise                    → `UserRole::User`
///
/// ## Paused Contract Behavior:
/// - If the contract is `paused` and caller is only a `User`:
///   - Requires the `bypass_contract_pause` permission in `UserAccessAccount`
///   - Otherwise, throws `ContractPaused`
///
/// ## Returns:
/// - `UserContext` with the resolved role
///
/// ## Usage:
/// Must be called at the top of **all restricted entrypoints** to ensure global
/// contract state (e.g., paused) and access control is consistently enforced.
///
/// ## Errors:
/// - `ContractPaused` if user lacks bypass permission and contract is paused
///
/// ===========================================================================
pub(crate) fn check_core<'info>(
    caller: &Pubkey,
    user_access: Option<&Account<'info, UserAccessAccount>>,
    token_state: &TokenState,
) -> Result<UserContext> {
    use UserRole::*;

    // Determine role based on ownership, API authority or admin status
    let role = match *caller {
        owner if owner == token_state.core.owner => Owner,
        api   if api   == token_state.core.api_authority => Api,
        _ => match user_access {
            Some(access) if access.is_admin => Admin,
            _ => User,
        },
    };
    
    // If the contract is paused, only specific roles or permissions may bypass
    if role == User && token_state.core.paused {
        let has_bypass = user_access
            .map(|access| access.has_permission("bypass_contract_pause"))
            .unwrap_or(false);

        if !has_bypass {
            return Err(ErrorCode::ContractPaused.into());
        }
    }

    Ok(UserContext { role })
}

/// ===========================================================================
/// Function: check_owner – Restricts Action to Contract Owner
///
/// Validates whether the given `caller` is the **owner of the contract**.
///
/// ## Behavior:
/// - Compares the caller’s public key to the `token_state.core.owner`
/// - Returns `Ok(())` only if the keys match
///
/// ## Usage:
/// - Should be used on **owner-only instructions**, such as:
///   - Transferring ownership
///   - Updating global authorities (e.g., API role)
///   - Performing irreversible or critical admin actions
///
/// ## Returns:
/// - `Ok(())` if the caller is the owner
///
/// ## Errors:
/// - `Unauthorized` if the caller is not the contract owner
///
/// ===========================================================================
pub(crate) fn check_owner(
    caller: &Pubkey,
    token_state: &TokenState,
) -> Result<()> {
    if *caller == token_state.core.owner {
        Ok(())
    } else {
        Err(ErrorCode::Unauthorized.into())
    }
}

/// ===========================================================================
/// Function: verify_permission – Checks Named Permission for a User
///
/// Validates whether a given user has a **specific named permission**.
///
/// ## Behavior:
/// - **Owner** and **Admin** roles automatically pass the check
/// - For regular users, the `UserAccessAccount` is inspected for the requested permission
/// - Permission strings map to internal bitflags (e.g., `"create_proposal"`)
///
/// ## Usage:
/// - Use inside restricted functions to gate access by fine-grained roles
/// - Combine with `check_core()` to ensure paused-state and role safety
///
/// ## Parameters:
/// - `ctx`: The `UserContext` describing caller role
/// - `user_access`: Optional reference to the caller’s `UserAccessAccount`
/// - `required_permission`: The string name of the permission being checked
///
/// ## Returns:
/// - `Ok(())` if the permission is granted
///
/// ## Errors:
/// - `Unauthorized` if the permission is missing or the access account is absent
///
/// ===========================================================================
pub(crate) fn verify_permission(
    ctx: &UserContext,
    user_access: Option<&Account<UserAccessAccount>>,
    required_permission: &str,
) -> Result<()> {
    if ctx.is_owner() || ctx.is_admin() {
        return Ok(());
    }

    if let Some(access) = user_access {
        if access.has_permission(required_permission) {
            return Ok(());
        }
    }

    Err(ErrorCode::Unauthorized.into())
}