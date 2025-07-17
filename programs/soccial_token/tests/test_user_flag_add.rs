
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
async fn test_add_flag_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Add flag "vip"
    let result = try_add_flag(&mut context, &owner, &user.pubkey(), vec!["vip".to_string()]).await;
    assert!(result.is_ok(), "Adding VIP flag should succeed");

    let seeds = derive_seeds(&context.program_id, &user.pubkey());
    let account = context.banks_client.get_account(seeds.user_access).await?.expect("UserAccess should exist");

    let user_state = UserAccessAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize");
    assert!(user_state.has_flag(ExtraFlag::Vip), "VIP flag should be set");
    Ok(())
}

#[tokio::test]
async fn test_add_flag_already_set_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // First time should succeed
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["staff".to_string()]).await.unwrap();

    context.refresh().await;

    // Second time should fail
    let result = try_add_flag(&mut context, &owner, &user.pubkey(), vec!["staff".to_string()]).await;
    assert_custom_error(result, UserErrorCode::FlagAlreadySet, "Adding flag twice should fail");
    Ok(())
}

#[tokio::test]
async fn test_add_flag_invalid_name_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Add a valid flag to force the creation of the user_access account
    try_add_flag(&mut context, &owner, &user.pubkey(), vec!["staff".to_string()]).await.unwrap();
    context.refresh().await;

    // Now attempt to add an invalid flag, expecting a specific error
    let result = try_add_flag(&mut context, &owner, &user.pubkey(), vec!["alien".to_string()]).await;
    assert_custom_error(result, UserErrorCode::UnknownFlagName, "Unknown flag name should fail");

    Ok(())
}

#[tokio::test]
async fn test_add_flag_unauthorized_should_fail() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let unauthorized = Keypair::new();
    let target = Keypair::new();

    fund_test_accounts_manual(&mut context.banks_client, &[&unauthorized], 10_000_000, &context.payer, &context.recent_blockhash).await;

    let result = try_add_flag(&mut context, &unauthorized, &target.pubkey(), vec!["whitelist".to_string()]).await;
    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not add flags");
    Ok(())
}
