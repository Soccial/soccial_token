// ======================================================================
// Soccial Token – Macros
//
// This file defines reusable macros for argument parsing, permission
// checks, ownership verification, and governance proposal validation.
//
// Macros simplify error handling and enforce consistent access control
// across instructions.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

/// Parses an argument from a `Vec<String>` safely, returning an error if the argument is missing or cannot be parsed.
///
/// # Example
/// let amount = parse_arg!(args, 0, u64)?;
#[macro_export]
macro_rules! parse_arg {
    ($args:expr, $index:expr, $typ:ty) => {{
        let value = $args.get($index)
            .ok_or(anchor_lang::error::Error::from($crate::ErrorCode::MissingArgument))?;
        value.parse::<$typ>()
            .map_err(|_| anchor_lang::error::Error::from($crate::ErrorCode::InvalidArgument))
    }};
}

/// Ensures that the provided `Vec<String>` has at least the required number of arguments.
///
/// # Example
/// require_args!(args, 2)?;
#[macro_export]
macro_rules! require_args {
    ($args:expr, $min:expr) => {{
        if $args.len() < $min {
            Err(anchor_lang::error::Error::from($crate::ErrorCode::NotEnoughArguments))
        } else {
            Ok(())
        }
    }};
}


/// Performs system-level checks to extract the user context for permission validation.
///
/// Uses the new `UserAccessAccount` structure with compact access_bits.
///
/// # Example
/// let user_ctx = check!(ctx, &caller)?;
#[macro_export]
macro_rules! check {
    ($ctx:expr, $caller:expr) => {
        crate::utils::system::check_core(
            $caller,
            $ctx.accounts.user_access.as_ref(),
            &$ctx.accounts.token_state
        )
    };
}


/// Verifies whether a given caller has the required permission using the new compact permission model.
///
/// # Example
/// verify_permission!(&user_ctx, &ctx.accounts.user_access, "mint_tokens")?;
#[macro_export]
macro_rules! verify_permission {
    ($user_ctx:expr, $user_access:expr, $permission:expr) => {
        crate::utils::system::verify_permission(
            $user_ctx,
            $user_access,
            $permission,
        )
    };
}

/// Combines `check!` and `verify_permission!` to perform a full permission check for a given action.
///
/// Returns early with an error if the user context is invalid or the permission is not granted.
///
/// # Example
/// secure!(ctx, &caller, "mint_tokens")?;
#[macro_export]
macro_rules! secure_old {
    ($ctx:expr, $caller:expr, $perm:expr) => {{
        let user_ctx = check!($ctx, $caller)?;
        verify_permission!(&user_ctx, $ctx.accounts.user_access.as_ref(), $perm)?;
    }};
}

/// Combines check! and verify_permission! to perform a full permission check for a given action.
///
/// Returns early with an error if the user context is invalid or the permission is not granted.
///
/// # Example
/// secure!(ctx, &caller, "mint_tokens")?;
#[macro_export]
macro_rules! secure {
    // Case: with explicit allow_api flag
    ($ctx:expr, $caller:expr, $perm:expr, $allow_api:expr) => {{
        let user_ctx = check!($ctx, $caller)?;
        if !($allow_api && user_ctx.is_api()) {
            verify_permission!(&user_ctx, $ctx.accounts.user_access.as_ref(), $perm)?;
        }
    }};

    // Case: default allow_api = false
    ($ctx:expr, $caller:expr, $perm:expr) => {{
        secure!($ctx, $caller, $perm, false)
    }};
}

/// Verifies whether the given caller is the contract owner.
///
/// # Example
/// check_owner!(ctx, &caller)?;
#[macro_export]
macro_rules! check_owner {
    ($ctx:expr, $caller:expr) => {
        crate::utils::system::check_owner(
            $caller,
            &$ctx.accounts.token_state,
        )
    };
}

/// Ensures the caller is the target user, or has a specific permission (admin/owner or explicit).
///
/// # Example
/// secure_user_or_permission!(ctx, &caller, &participant, "manage_staking")?;
#[macro_export]
macro_rules! secure_user_or_permission {
    ($ctx:expr, $caller:expr, $target:expr, $perm:expr) => {{
        let user_ctx = match check!($ctx, $caller) {
            Ok(ctx) => ctx,
            Err(e) => return Err(e),
        };

        if $caller != $target {
            verify_permission!(&user_ctx, $ctx.accounts.user_access.as_ref(), $perm)?;
        }
    }};
}

/// Ensures the caller is exactly the participant (resource owner).
///
/// This macro enforces that only the participant themselves can perform the action,
/// even if the caller is an admin or owner.
///
/// # Example
/// secure_user_only!(ctx, &caller, &participant_pubkey)?;
#[macro_export]
macro_rules! secure_user_only {
    ($ctx:expr, $caller:expr, $target:expr) => {{
        if $caller != $target {
            return Err($crate::ErrorCode::Unauthorized.into());
        }
    }};
}
