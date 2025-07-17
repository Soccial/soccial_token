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
async fn test_disable_staking_plan_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let plan_id = 5;

    // Add a staking plan
    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 700)
        .await
        .expect("Should be able to add plan");

    // Disable the staking plan
    try_disable_staking_plan(&mut context, &owner, plan_id)
        .await
        .expect("Should be able to disable plan");

    // Fetch the staking state and confirm plan is inactive
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState account must exist");
    let staking_state = StakingState::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize StakingState");

    let plan = staking_state.plans.iter().find(|p| p.plan_id == plan_id).expect("Plan must exist");
    assert!(!plan.active, "Plan should be inactive after disable");

    Ok(())
}


#[tokio::test]
async fn test_disable_staking_plan_should_fail_if_plan_does_not_exist() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let invalid_plan_id = 99;

    let result = try_disable_staking_plan(&mut context, &owner, invalid_plan_id).await;

    assert_custom_error(result, StakingErrorCode::PlanNotFound, "Expected PlanNotFound for non-existent plan");

    Ok(())
}

#[tokio::test]
async fn test_disable_staking_plan_should_fail_if_already_disabled() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let plan_id = 5;

    // Add and then disable the plan
    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 600).await?;
    try_disable_staking_plan(&mut context, &owner, plan_id).await?;

    context.refresh().await;

    // Try to disable again
    let result = try_disable_staking_plan(&mut context, &owner, plan_id).await;

    assert_custom_error(result, StakingErrorCode::PlanNotFound, "Expected PlanNotFound when disabling an already inactive plan");

    Ok(())
}

#[tokio::test]
async fn test_disable_staking_plan_should_fail_if_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;
    let intruder = Keypair::new();

    let plan_id = 4;

    let result = try_disable_staking_plan(&mut context, &intruder, plan_id).await;
    
    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error for disable_staking_plan");

    Ok(())
}
