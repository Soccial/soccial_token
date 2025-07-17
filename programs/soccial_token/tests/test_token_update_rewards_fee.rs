use soccial_token::economics::EconomicsErrorCode;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair,
    transport::TransportError,
};
use soccial_token::utils::error::ErrorCode;

mod testutils;
mod trymethods;

use crate::testutils::environment::setup_test_env;
use crate::trymethods::trygovernance::try_approve_proposal_flow;
use crate::trymethods::trytoken::*;
use crate::testutils::basics::*;
use soccial_token::economy::fee::MAX_REWARDS_FEE_BPS;

#[tokio::test]
async fn test_update_rewards_fee_success() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;


    let proposal_types = vec!["UpdateRewardFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing proposal".to_string(),
        proposal_types
    ).await?;
    
    let new_fee = 777; // Valid BPS
    try_update_rewards_fee(&mut context, &admin, vec![new_fee.to_string()], proposal_id).await.unwrap();

    let state = context.load_token_state().await;
    assert_eq!(state.fee.rewards_fee_bps, new_fee, "Rewards fee should be updated correctly");

    Ok(())
}

#[tokio::test]
async fn test_update_rewards_fee_without_approval_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let proposal_id = 1;

    let new_fee = 777; // Valid BPS
    let result = try_update_rewards_fee(&mut context, &admin, vec![new_fee.to_string()], proposal_id).await;

    assert!(
        result.is_err(),
        "🚨 Expected failure due to lack of governance approval."
    );

    Ok(())
}


#[tokio::test]
async fn test_update_rewards_fee_exceeds_max_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    
    let proposal_types = vec!["UpdateRewardFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing proposal".to_string(),
        proposal_types
    ).await?;

    let invalid_fee = MAX_REWARDS_FEE_BPS + 1;
    let result = try_update_rewards_fee(&mut context, &admin, vec![invalid_fee.to_string()], proposal_id).await;

    assert_custom_error(result, EconomicsErrorCode::InvalidFeeValue, "Fee exceeding max should fail");

    Ok(())
}

#[tokio::test]
async fn test_update_rewards_fee_unauthorized_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let proposal_types = vec!["UpdateRewardFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing proposal".to_string(),
        proposal_types
    ).await?;

    let unauthorized = Keypair::new();
    fund_test_accounts_manual(&mut context.banks_client, &[&unauthorized], 10_000_000, &context.payer, &context.recent_blockhash).await;

    let result = try_update_rewards_fee(&mut context, &unauthorized, vec!["500".to_string()], proposal_id).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not update rewards fee");

    Ok(())
}
