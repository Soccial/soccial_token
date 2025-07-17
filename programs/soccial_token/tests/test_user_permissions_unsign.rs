
use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::{auth::UserErrorCode, utils::error::ErrorCode};
mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryuser::*;
use soccial_token::auth::user::UserAccessAccount;

#[tokio::test]
async fn test_unsign_permission_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    try_assign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_permissions".to_string()]).await.unwrap();
    context.refresh().await;

    let result = try_unsign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_permissions".to_string()]).await;
    assert!(result.is_ok(), "Unsigning permission should succeed");

    let seeds = derive_seeds(&context.program_id, &user.pubkey());
    let account = context.banks_client.get_account(seeds.user_access).await?.expect("User access account should exist");
    let user_state = UserAccessAccount::try_deserialize(&mut &account.data[..]).unwrap();
    assert!(!user_state.has_permission("manage_permissions"), "Permission should be removed");

    Ok(())
}

#[tokio::test]
async fn test_unsign_permission_not_set_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Assign unrelated permission just to ensure account exists
    try_assign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_permissions".to_string()]).await.unwrap();
    context.refresh().await;

    let result = try_unsign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_contract".to_string()]).await;
    assert_custom_error(result, UserErrorCode::PermissionNotSet, "Unsigning non-existent permission should fail");

    Ok(())
}

#[tokio::test]
async fn test_unsign_permission_invalid_name_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Assign any valid permission to ensure account exists
    try_assign_permission(&mut context, &owner, &user.pubkey(), vec!["manage_permissions".to_string()]).await.unwrap();
    context.refresh().await;

    let result = try_unsign_permission(&mut context, &owner, &user.pubkey(), vec!["unknown_perm".to_string()]).await;
    assert_custom_error(result, UserErrorCode::UnknownPermissionName, "Unknown permission name should fail");

    Ok(())
}

#[tokio::test]
async fn test_unsign_permission_unauthorized_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let unauthorized = Keypair::new();
    let target = Keypair::new();
    fund_test_accounts_manual(&mut context.banks_client, &[&unauthorized], 10_000_000, &context.payer, &context.recent_blockhash).await;

    try_assign_permission(&mut context, &owner, &target.pubkey(), vec!["manage_permissions".to_string()]).await.unwrap();
    context.refresh().await;

    let result = try_unsign_permission(&mut context, &unauthorized, &target.pubkey(), vec!["edit_profile".to_string()]).await;
    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not unsign permissions");

    Ok(())
}
