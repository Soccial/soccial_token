// ============================================================================
// Soccial Token – User Access Management Helpers
// ----------------------------------------------------------------------------
//
// This module provides utility functions to test all instructions related to
// user access, admin roles, flags (VIP, staff, etc.), and permission management
// in the Soccial Token smart contract.
//
// It wraps high-level logic for modifying user roles and attributes using
// unified access control across the system.
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Add or remove admin access  
// ✔ Assign early adopter flags (phase 1/2)  
// ✔ Add or remove arbitrary user flags  
// ✔ Assign or revoke custom permissions  
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_add_admin` / `try_remove_admin`: Grant or revoke admin privileges  
// - `try_add_early_adopter`: Assign phase-based early adopter status  
// - `try_add_flag` / `try_remove_flag`: Handle user flags like VIP, STAFF  
// - `try_assign_permission` / `try_unsign_permission`: Manage fine-grained permissions  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    system_program,
};
use solana_sdk::transport::TransportError;
use crate::testutils::environment::EnvProgramTestContext;
use crate::testutils::basics::*;
use soccial_token::{instruction as soccial_instruction, accounts as soccial_accounts};


// ============================================================================
/// Attempts to add a new admin via the unified user access system.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized user (must have admin or owner rights)
/// - `new_admin`: Public key of the user to promote
///
/// # Returns:
/// `Ok(())` if added, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_add_admin(&mut context, &owner, &user.pubkey()).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_add_admin(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    new_admin: &Pubkey,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, new_admin);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUser {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *new_admin,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::AddAdmin {}, // sem args
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Attempts to remove admin rights from a user via the unified user access system.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer (admin or owner)
/// - `target_admin`: Public key of the admin to demote
///
/// # Returns:
/// `Ok(())` if removed, or `TransportError` otherwise
///
/// # Example:
/// ```
/// try_remove_admin(&mut context, &owner, &admin.pubkey()).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_remove_admin(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_admin: &Pubkey,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_admin);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUserRemove {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_admin,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::RemoveAdmin {}, 
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Adds a user as early adopter (phase 1 or 2) using the unified access system.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `target_user`: Target user to label as early adopter
/// - `phase`: Either 1 or 2 (phase type)
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_add_early_adopter(&mut context, &admin, &user.pubkey(), 1).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_add_early_adopter(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_user: &Pubkey,
    phase: u8,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_user);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUser {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_user,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::AddEarlyAdopter {
            args: vec![phase.to_string()],
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Adds a flag (e.g., VIP, STAFF) to a user account.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Admin or authorized manager
/// - `target_user`: User to assign the flag to
/// - `args`: Vector with flags (e.g., `vec!["VIP"]`)
///
/// # Returns:
/// `Ok(())` if success, or error on invalid permission
///
/// # Example:
/// ```
/// try_add_flag(&mut context, &admin, &user.pubkey(), vec!["VIP".to_string()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_add_flag(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_user: &Pubkey,
    args: Vec<String>,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_user);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUser {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_user,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::AddFlag {
            args, // directly pass the vector
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Removes a flag (e.g., VIP, STAFF) from a user account.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `target_user`: User to remove the flag from
/// - `args`: Flag(s) to remove as strings (e.g., `vec!["STAFF"]`)
///
/// # Returns:
/// `Ok(())` if removed, or error otherwise
///
/// # Example:
/// ```
/// try_remove_flag(&mut context, &admin, &user.pubkey(), vec!["VIP".to_string()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_remove_flag(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_user: &Pubkey,
    args: Vec<String>,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_user);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUser {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_user,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::RemoveFlag {
            args, // directly pass the vector
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Assigns a custom permission to a user (e.g., `manage_economy`).
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `target_user`: The user to assign the permission
/// - `args`: Permission string(s) (e.g., `vec!["manage_staking"]`)
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_assign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_governance".to_string()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_assign_permission(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_user: &Pubkey,
    args: Vec<String>,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_user);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUser {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_user,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::AssignPermission {
            args,
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Revokes a previously assigned custom permission from a user.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `target_user`: The user to remove the permission from
/// - `args`: Permissions to remove (e.g., `vec!["manage_token"]`)
///
/// # Returns:
/// `Ok(())` if successful, or error on failure
///
/// # Example:
/// ```
/// try_unsign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_staking".to_string()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_unsign_permission(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    target_user: &Pubkey,
    args: Vec<String>,
) -> Result<(), TransportError> {
    let caller_seeds = derive_seeds(&context.program_id, &caller.pubkey());
    let target_seeds = derive_seeds(&context.program_id, target_user);

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageUserRemove {
            caller: caller.pubkey(),
            user_access: None,
            target_user: *target_user,
            target_user_access: target_seeds.user_access,
            token_state: caller_seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::UnsignPermission {
            args,
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}
