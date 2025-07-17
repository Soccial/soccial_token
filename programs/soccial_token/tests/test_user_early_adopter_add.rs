use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::auth::user::{ExtraFlag, UserAccessAccount};
use soccial_token::utils::error::ErrorCode;
use soccial_token::auth::error::UserErrorCode;
mod testutils;
mod trymethods;

use crate::testutils::basics::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryuser::*;

#[tokio::test]
async fn test_add_early_adopter_phase1_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
   

    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    try_add_early_adopter(&mut context, &owner, &user.pubkey(), 1).await.unwrap();

    let seeds = derive_seeds(&context.program_id, &user.pubkey());
    let account = context.banks_client.get_account(seeds.user_access).await?.expect("UserAccess should exist");

    let user_state = UserAccessAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize");
    assert!(user_state.has_flag(ExtraFlag::EarlyAdopter1));
    Ok(())
}

#[tokio::test]
async fn test_add_early_adopter_phase2_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
  
    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    try_add_early_adopter(&mut context, &owner, &user.pubkey(), 2).await.unwrap();

    let seeds = derive_seeds(&context.program_id, &user.pubkey());
    let account = context.banks_client.get_account(seeds.user_access).await?.expect("UserAccess should exist");

    let user_state = UserAccessAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize");
    assert!(user_state.has_flag(ExtraFlag::EarlyAdopter2));
    Ok(())
}

#[tokio::test]
async fn test_add_early_adopter_invalid_phase_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
   
    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    let result = try_add_early_adopter(&mut context, &owner, &user.pubkey(), 99).await;
    assert_custom_error(result, UserErrorCode::InvalidPhase, "Invalid early adopter phase");
    Ok(())
}

#[tokio::test]
async fn test_add_early_adopter_duplicate_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
  
    let user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    try_add_early_adopter(&mut context, &owner, &user.pubkey(), 1).await.unwrap();

    // Refresh for the next transaction
    context.refresh().await;
    
    let result = try_add_early_adopter(&mut context, &owner, &user.pubkey(), 1).await;
    assert_custom_error(result, UserErrorCode::FlagAlreadySet, "Duplicate early adopter flag should fail");
    Ok(())
}

#[tokio::test]
async fn test_add_early_adopter_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;


    let unauthorized = Keypair::new();
    let target = Keypair::new();

    fund_test_accounts_manual(
        &mut context.banks_client,
        &[&unauthorized],
        10_000_000,
        &context.payer,
        &context.recent_blockhash,
    ).await;

    let result = try_add_early_adopter(&mut context, &unauthorized, &target.pubkey(), 1).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not add early adopter");
    Ok(())
}

