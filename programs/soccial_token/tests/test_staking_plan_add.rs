// ======================================================================
// 
// ======================================================================

use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use soccial_token::utils::error::ErrorCode;
use solana_sdk::{signature::Keypair, signer::Signer};
use solana_sdk::transport::TransportError;
use soccial_token::{self, staking:: {error::StakingErrorCode, StakingState}};
use testutils::basics::{assert_custom_error, derive_seeds};
mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::trymethods::trystaking::*;

#[tokio::test]
async fn test_add_staking_plan_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;


    let plan = 5;
    // Add valid staking plan
    try_add_staking_plan(&mut context, &owner, plan, 86400, 500).await?;

    // Check that staking_state contains the new plan
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState account should exist");
    let staking_state = StakingState::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize StakingState");

    let plan = staking_state.get_plan(plan).expect("Plan should exist");
    assert_eq!(plan.lockup_duration, 86400);
    assert_eq!(plan.apr_bps, 500);
    assert_eq!(plan.active, true);

    Ok(())
}

#[tokio::test]
async fn test_add_staking_plan_should_fail_with_zero_lockup() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let result = try_add_staking_plan(&mut context, &owner, 2, 0, 500).await;

    assert_custom_error(result, ErrorCode::InvalidArgument, "Expected InvalidArgument for zero lockup");

    Ok(())
}

#[tokio::test]
async fn test_add_staking_plan_should_fail_with_zero_apr() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let result = try_add_staking_plan(&mut context, &owner, 3, 86400, 0).await;

    assert_custom_error(result, ErrorCode::InvalidArgument, "Expected InvalidArgument for zero APR");

    Ok(())
}

#[tokio::test]
async fn test_add_staking_plan_should_fail_with_duplicate_id() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let result = try_add_staking_plan(&mut context, &owner, 4, 604800, 800).await;

    assert_custom_error(result, StakingErrorCode::PlanAlreadyExists, "Expected PlanAlreadyExists error for duplicate plan_id");

    Ok(())
}

#[tokio::test]
async fn test_add_staking_plan_should_fail_if_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;
    let intruder = Keypair::new();

    let result = try_add_staking_plan(&mut context, &intruder, 10, 86400, 500).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error for add_staking_plan");

    Ok(())
}
