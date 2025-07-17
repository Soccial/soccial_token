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
async fn test_edit_staking_plan_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let plan_id = 6;
    let new_lockup = 172800; // 2 dias
    let new_apr = 900;

    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 500).await?;

    try_edit_staking_plan(&mut context, &owner, plan_id, new_lockup, new_apr).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState account must exist");

    let staking_state = StakingState::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize StakingState");

    let plan = staking_state.get_plan(plan_id).expect("Plan should exist");
    assert_eq!(plan.lockup_duration, new_lockup);
    assert_eq!(plan.apr_bps, new_apr);

    Ok(())
}


#[tokio::test]
async fn test_edit_staking_plan_should_fail_if_plan_not_found() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let nonexistent_plan_id = 99;

    let result = try_edit_staking_plan(&mut context, &owner, nonexistent_plan_id, 86400, 500).await;

    assert_custom_error(result, StakingErrorCode::PlanNotFound, "Expected PlanNotFound for nonexistent plan_id");

    Ok(())
}

#[tokio::test]
async fn test_edit_staking_plan_should_fail_with_zero_lockup() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let plan_id = 8;

    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 500).await?;

    let result = try_edit_staking_plan(&mut context, &owner, plan_id, 0, 500).await;

    assert_custom_error(result, ErrorCode::InvalidArgument, "Expected InvalidArgument for zero lockup duration");

    Ok(())
}

#[tokio::test]
async fn test_edit_staking_plan_should_fail_with_zero_apr() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let plan_id = 9;

    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 500).await?;

    let result = try_edit_staking_plan(&mut context, &owner, plan_id, 86400, 0).await;

    assert_custom_error(result, ErrorCode::InvalidArgument, "Expected InvalidArgument for zero APR");

    Ok(())
}

#[tokio::test]
async fn test_edit_staking_plan_should_fail_if_unauthorized() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let intruder = Keypair::new();

    let plan_id = 11;
    try_add_staking_plan(&mut context, &owner, plan_id, 86400, 500).await?;

    let result = try_edit_staking_plan(&mut context, &intruder, plan_id, 172800, 900).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error for edit_staking_plan");

    Ok(())
}
