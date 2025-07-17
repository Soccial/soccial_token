use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::{auth::{user::ExtraFlag, UserErrorCode}, utils::error::ErrorCode};
mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryuser::*;
use soccial_token::auth::user::UserAccessAccount;

#[tokio::test]
async fn test_remove_flag_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // First, add the flag
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["vip".to_string()]).await.unwrap();
    context.refresh().await;

    // Then remove the flag
    let result = try_remove_flag(&mut context, &owner, &user.pubkey(), vec!["vip".to_string()]).await;
    assert!(result.is_ok(), "Removing flag should succeed");

    let seeds = derive_seeds(&context.program_id, &user.pubkey());
    let account = context.banks_client.get_account(seeds.user_access).await?.expect("UserAccess should still exist");

    let user_state = UserAccessAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize");
    assert!(!user_state.has_flag(ExtraFlag::Vip), "VIP flag should be removed");
    Ok(())
}

#[tokio::test]
async fn test_remove_flag_not_set_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Add a different flag just to create the account
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["vip".to_string()]).await.unwrap();
    context.refresh().await;

    // Attempt to remove a flag that was never set
    let result = try_remove_flag(&mut context, &owner, &user.pubkey(), vec!["staff".to_string()]).await;
    assert_custom_error(
        result,
        UserErrorCode::FlagNotSet,
        "Removing a flag that was never set should fail",
    );
    Ok(())
}

#[tokio::test]
async fn test_remove_flag_invalid_name_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Add a valid flag first, just to create the target_user_access account
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["staff".to_string()]).await.unwrap();
    context.refresh().await;

    // Now attempt to remove an invalid flag name
    let result = try_remove_flag(&mut context, &owner, &user.pubkey(), vec!["ghost".to_string()]).await;
    assert_custom_error(
        result,
        UserErrorCode::UnknownFlagName,
        "Invalid flag name should trigger UnknownFlagName error",
    );
    Ok(())
}


#[tokio::test]
async fn test_remove_flag_unauthorized_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let unauthorized = Keypair::new();
    let target = Keypair::new();

    fund_test_accounts_manual(&mut context.banks_client, &[&unauthorized], 10_000_000, &context.payer, &context.recent_blockhash).await;

    // Pre-add flag with owner
    try_add_flag(&mut context, &owner, &target.pubkey(), vec!["staff".to_string()]).await.unwrap();
    context.refresh().await;

    // Try to remove with an unauthorized user
    let result = try_remove_flag(&mut context, &unauthorized, &target.pubkey(), vec!["staff".to_string()]).await;
    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized removal should fail");
    Ok(())
}

#[tokio::test]
async fn test_remove_flag_user_has_different_flag_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Add "beta" flag first
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["beta".to_string()]).await.unwrap();
    context.refresh().await;

    // Attempt to remove "vip" flag which was never set
    let result = try_remove_flag(&mut context, &owner, &user.pubkey(), vec!["vip".to_string()]).await;
    assert_custom_error(result, UserErrorCode::FlagNotSet, "Removing flag that was never set should fail");
    Ok(())
}
