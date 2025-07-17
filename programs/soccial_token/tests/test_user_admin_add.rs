// ======================================================================
// File: tests/test_add_admin.rs
// Description: Updated tests for AddAdmin logic using unified user.rs
// ======================================================================

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
async fn test_add_admin_should_succeed() -> Result<(), TransportError> {

    let (mut context, owner) = setup_test_env().await;

    let new_admin = Keypair::new();

    context.mint_tokens(&owner, 1_000_000).await;

    try_add_admin(&mut context, &owner, &new_admin.pubkey()).await.unwrap();

    let seeds = derive_seeds(&context.program_id, &new_admin.pubkey());
    let user_access = context
        .banks_client
        .get_account(seeds.user_access)
        .await?
        .expect("user_access account should exist");

        let user_state = UserAccessAccount::try_deserialize(&mut &user_access.data[..])
    .expect("Failed to deserialize user_access");

    assert!(user_state.is_admin, "Admin flag not set correctly");

    Ok(())
}

#[tokio::test]
async fn test_add_admin_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let unauthorized = Keypair::new();
    let target_admin = Keypair::new();

    fund_test_accounts_manual(
        &mut context.banks_client,
        &[&unauthorized],
        10_000_000,
        &context.payer,
        &context.recent_blockhash,
    ).await;

    let result = try_add_admin(&mut context, &unauthorized, &target_admin.pubkey()).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error");


    Ok(())
}

