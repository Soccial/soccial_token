// ======================================================================
// Test file
// ======================================================================

// This test file corresponds to the `create_proposal` function in the Soccial Token contract.
use anchor_lang::AccountDeserialize;
use soccial_token::governance::GovernanceError;
use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;
use soccial_token::governance::state::ProposalAccount;
use testutils::basics::{assert_custom_error, derive_proposal_account, derive_seeds};
mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::trygovernance::*; 

#[tokio::test]
async fn test_create_proposal_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let description = "Testing create proposal".to_string();
    let proposal_types = vec!["UpdateGovernance".to_string()];

    try_create_proposal(&mut context, &admin, description.clone(), proposal_types.clone(), None, None)
        .await
        .expect("Proposal creation should succeed");
    
    
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) = derive_proposal_account(&context.program_id, current_count - 1);
    
    let account = context.banks_client.get_account(proposal_account_pubkey).await?.expect("Proposal account should exist");
    let proposal_state = ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize ProposalAccount");

    println!("✅ Proposal id: {}", proposal_state.id);
    println!("✅ description: {}", proposal_state.description);

    assert_eq!(proposal_state.description, description);

    Ok(())
}

#[tokio::test]
async fn test_create_proposal_with_time_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Proposal with custom timing".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];

    let current_timestamp = context.get_current_unix_timestamp().await;
    let start_time = Some(current_timestamp + 60); // start in 1 minute
    let duration = Some(3600); // duration: 1 hour

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) = derive_proposal_account(&context.program_id, current_count);

    try_create_proposal(
        &mut context,
        &owner,
        description.clone(),
        proposal_types.clone(),
        start_time,
        duration,
    )
    .await
    .expect("Proposal with time should succeed");

    let account = context
        .banks_client
        .get_account(proposal_account_pubkey)
        .await?
        .expect("Proposal account should exist");

    let proposal_state =
        ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Deserialization failed");

    assert_eq!(proposal_state.description, description);
    assert_eq!(proposal_state.start_time, start_time.unwrap());
    assert_eq!(
        proposal_state.end_time,
        start_time.unwrap() + duration.unwrap()
    );

    Ok(())
}

#[tokio::test]
async fn test_create_proposal_with_past_start_time() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Invalid start time".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];
    
    let current_timestamp = context.get_current_unix_timestamp().await;
    let past_start_time = Some(current_timestamp - 300); // 5 mins ago
    let duration = Some(600);

    let result = try_create_proposal(
        &mut context,
        &owner,
        description,
        proposal_types,
        past_start_time,
        duration,
    )
    .await;

    assert_custom_error(result, GovernanceError::StartTimeInPast, "Expected StartTimeInPast error");

    Ok(())
}

#[tokio::test]
async fn test_create_proposal_invalid_type() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Invalid proposal type".to_string();
    let proposal_types = vec!["ThisDoesNotExist".to_string()];

    let result = try_create_proposal(&mut context, &owner, description, proposal_types, None, None).await;

    assert_custom_error(result, GovernanceError::InvalidProposalType, "Expected InvalidProposalType error");

    Ok(())
}

#[tokio::test]
async fn test_create_proposal_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let description = "Unauthorized proposal attempt".to_string();
    let proposal_types = vec!["StrategicDecision".to_string()];

    let result = try_create_proposal(&mut context, &intruder, description, proposal_types, None, None).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error");

    Ok(())
}

#[tokio::test]
async fn test_create_and_list_proposals() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Test protocol upgrade".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) = derive_proposal_account(&context.program_id, current_count);

    try_create_proposal(&mut context, &owner, description.clone(), proposal_types.clone(), None, None)
        .await
        .expect("Proposal creation should succeed");

    let account = context.banks_client.get_account(proposal_account_pubkey).await?.expect("Proposal account should exist");
    let proposal_state = ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize ProposalAccount");

    assert_eq!(proposal_state.description, description);

    let listed_account = context.banks_client.get_account(proposal_account_pubkey).await?.expect("Listed proposal should exist");
    let listed_proposal = ProposalAccount::try_deserialize(&mut &listed_account.data[..]).expect("Failed to deserialize listed proposal");

    assert_eq!(listed_proposal.description, description);

    Ok(())
}