
use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::utils::error::ErrorCode;
mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryuser::*;
use soccial_token::auth::user::UserAccessAccount;

#[tokio::test]
async fn test_remove_admin_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let new_admin = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Grant admin rights first
    try_add_admin(&mut context, &owner, &new_admin.pubkey()).await.unwrap();

    // Sanity check
    let seeds = derive_seeds(&context.program_id, &new_admin.pubkey());
    let user_access = context
        .banks_client
        .get_account(seeds.user_access)
        .await?
        .expect("user_access should exist");

    let mut data: &[u8] = &user_access.data;
    let state = UserAccessAccount::try_deserialize(&mut data).unwrap();
    assert!(state.is_admin);

    // Refresh blockhash before next tx
    context.refresh().await;

    // Remove admin rights
    try_remove_admin(&mut context, &owner, &new_admin.pubkey()).await.unwrap();

    context.refresh().await;

    let updated = context
        .banks_client
        .get_account(seeds.user_access)
        .await?
        .expect("user_access should still exist");

    let mut updated_data: &[u8] = &updated.data;
    let updated_state = UserAccessAccount::try_deserialize(&mut updated_data).unwrap();

    assert!(!updated_state.is_admin, "🚨 Admin flag was not removed properly");

    Ok(())
}

#[tokio::test]
async fn test_remove_admin_unauthorized() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let unauthorized = Keypair::new();
    let new_admin = Keypair::new();

    fund_test_accounts_manual(
        &mut context.banks_client,
        &[&unauthorized],
        10_000_000,
        &context.payer,
        &context.recent_blockhash,
    ).await;

    // First add admin
    try_add_admin(&mut context, &owner, &new_admin.pubkey()).await.unwrap();

    context.refresh().await;

    // Now remove it
    let result = try_remove_admin(&mut context, &unauthorized, &new_admin.pubkey()).await;
    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not remove admin");

    Ok(())
}

#[tokio::test]
async fn test_remove_admin_nonexistent_user_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let target = Keypair::new(); // This user has no associated UserAccessAccount

    // Ensure the user_access account truly does not exist
    let seeds = derive_seeds(&context.program_id, &target.pubkey());
    let maybe_account = context.banks_client.get_account(seeds.user_access).await?;
    assert!(maybe_account.is_none(), "Test assumption failed: user_access should not exist");

    // Attempt to remove admin rights from a nonexistent user
    let result = try_remove_admin(&mut context, &owner, &target.pubkey()).await;

    // Expect the operation to fail because the account does not exist
    assert!(
        result.is_err(),
        "🚨 Removing admin from a nonexistent user should fail"
    );

    Ok(())
}
