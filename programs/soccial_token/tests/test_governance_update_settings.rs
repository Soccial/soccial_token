// ======================================================================
// Test file: test_governance_update_settings.rs
// ======================================================================

use anchor_lang::AccountDeserialize;
use soccial_token::utils::error::ErrorCode;
use soccial_token::governance::state::GovernanceState;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;
use testutils::basics::{assert_custom_error, derive_seeds};
mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::trymethods::trygovernance::*;

#[tokio::test]
async fn test_update_governance_settings_success() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let args = vec![
        "min_tokens=500".to_string(),
        "quorum_percent=30".to_string(),
        "voting_duration=172800".to_string(),   // 2 days
        "validity_period=604800".to_string(),   // 7 days
    ];

    // Perform the update
    try_update_governance_settings(&mut context, &owner, args)
        .await
        .expect("Governance settings update should succeed");

    // Fetch the updated GovernanceState
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client
        .get_account(seeds.governance_state)
        .await?
        .expect("GovernanceState account should exist");

    let governance_state = GovernanceState::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize GovernanceState");

    // Validate updated fields
    assert_eq!(governance_state.min_vote_tokens, 500, "Min vote tokens should match");
    assert_eq!(governance_state.quorum_percent, 30, "Quorum percent should match");
    assert_eq!(governance_state.voting_duration, 172800, "Voting duration should match");
    assert_eq!(governance_state.validity_period, 604800, "Validity period should match");

    Ok(())
}

#[tokio::test]
async fn test_update_governance_settings_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let intruder = Keypair::new();
    create_and_fund_intruder(&mut context, &intruder).await?;

    let args = vec![
        "min_tokens=1000".to_string(),
        "quorum_percent=50".to_string(),
        "voting_duration=86400".to_string(),
        "validity_period=259200".to_string(),
    ];

    // Try to update with an unauthorized user
    let result = try_update_governance_settings(&mut context, &intruder, args).await;

    assert_custom_error(
        result,
        ErrorCode::Unauthorized,
        "Unauthorized user should not update governance settings",
    );

    Ok(())
}

#[tokio::test]
async fn test_update_min_tokens_only() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(&mut context, &owner, vec!["min_tokens=777".to_string()]).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.min_vote_tokens, 777);
    Ok(())
}

#[tokio::test]
async fn test_update_quorum_percent_only() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(&mut context, &owner, vec!["quorum_percent=42".to_string()]).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.quorum_percent, 42);
    Ok(())
}

#[tokio::test]
async fn test_update_voting_duration_only() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(&mut context, &owner, vec!["voting_duration=123456".to_string()]).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.voting_duration, 123456);
    Ok(())
}

#[tokio::test]
async fn test_update_validity_period_only() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(&mut context, &owner, vec!["validity_period=888888".to_string()]).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.validity_period, 888888);
    Ok(())
}

#[tokio::test]
async fn test_update_min_tokens_and_quorum() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(
        &mut context,
        &owner,
        vec!["min_tokens=1000".to_string(), "quorum_percent=33".to_string(),]
    ).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.min_vote_tokens, 1000);
    assert_eq!(state.quorum_percent, 33);
    Ok(())
}

#[tokio::test]
async fn test_update_voting_duration_and_validity() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    try_update_governance_settings(
        &mut context,
        &owner,
        vec!["voting_duration=9999".to_string(), "validity_period=2222".to_string(),]
    ).await?;

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let account = context.banks_client.get_account(seeds.governance_state).await?.unwrap();
    let state = GovernanceState::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(state.voting_duration, 9999);
    assert_eq!(state.validity_period, 2222);
    Ok(())
}

