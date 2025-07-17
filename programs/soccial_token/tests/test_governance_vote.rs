// ======================================================================
// Test
// ======================================================================

// This test file corresponds to the `create_proposal` function in the Soccial Token contract.
use anchor_lang::AccountDeserialize;
use soccial_token::governance::GovernanceError;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;
use soccial_token::governance::state::ProposalAccount;
use testutils::basics::{derive_proposal_account, derive_seeds};
mod testutils;
mod trymethods;
use crate::testutils::basics::assert_custom_error;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::trygovernance::*; 

#[tokio::test]
async fn test_vote_success() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    // Arrange: Create a proposal
    let description = "Vote for improvement".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) =
        derive_proposal_account(&context.program_id, current_count);

    fund_lamports(&mut context, &admin, 1_000_000_000).await?;

    try_create_proposal(
        &mut context,
        &admin,
        description.clone(),
        proposal_types.clone(),
        None,     // start_time
        None,     // duration
    )
    .await
    .expect("Proposal creation should succeed");

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Define the amount of tokens to transfer and vote with
    let vote_tokens = 200_000_000u64;

    // Mint tokens to admin and transfer to participant
    context.mint_tokens(&admin, vote_tokens).await;
    context
        .transfer_tokens_to_user(&admin, &participant.pubkey(), vote_tokens)
        .await?;

    // Act: Vote YES with participant
    try_vote(&mut context, &participant, proposal_account_pubkey, true)
        .await
        .expect("Voting should succeed");

    // Assert: Check proposal state
    let account = context
        .banks_client
        .get_account(proposal_account_pubkey)
        .await?
        .expect("Proposal account should exist");

    let proposal_state =
        ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Deserialization failed");

    assert_eq!(
        proposal_state.votes_for,
        vote_tokens,
        "Votes for should match the token amount used"
    );

    Ok(())
}

#[tokio::test]
async fn test_vote_twice_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    // Arrange: Create a proposal
    let description = "Vote for improvement".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) =
        derive_proposal_account(&context.program_id, current_count);

    fund_lamports(&mut context, &admin, 1_000_000_000).await?;

    try_create_proposal(
        &mut context,
        &admin,
        description.clone(),
        proposal_types.clone(),
        None,     // start_time
        None,     // duration
    )
    .await
    .expect("Proposal creation should succeed");

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Define the amount of tokens to transfer and vote with
    let vote_tokens = 200_000_000u64;

    // Mint tokens to admin and transfer to participant
    context.mint_tokens(&admin, vote_tokens).await;
    context
        .transfer_tokens_to_user(&admin, &participant.pubkey(), vote_tokens)
        .await?;

    // Act: Vote YES with participant
    try_vote(&mut context, &participant, proposal_account_pubkey, true)
        .await
        .expect("Voting should succeed");

    // Act: Second vote should fail
    let result = try_vote(&mut context, &participant, proposal_account_pubkey, false).await;

    // Assert: Should fail due to vote_receipt account already initialized
    assert!(
        result.is_err(),
        "Second vote should fail with an error"
    );

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("custom program error: 0x0")
            || err.contains("already in use")
            || err.contains("AccountAlreadyInitialized"),
        "Expected account initialization error, got: {}",
        err
    );

    Ok(())
}


#[tokio::test]
async fn test_vote_nonexistent_proposal() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let fake_proposal = Keypair::new().pubkey();

    let result = try_vote(&mut context, &admin, fake_proposal, true).await;

    assert!(result.is_err(), "Should error on non-existent proposal");
    
    Ok(())
}

#[tokio::test]
async fn test_vote_insufficient_tokens() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let description = "Vote for improvement".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_account_pubkey, _) =
        derive_proposal_account(&context.program_id, current_count);

    fund_lamports(&mut context, &admin, 1_000_000_000).await?;

    try_create_proposal(
        &mut context,
        &admin,
        description.clone(),
        proposal_types.clone(),
        None,
        None,
    )
    .await?;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    // fund lamports. Because we want the error on Insufficient SCTK
    fund_lamports(&mut context, &participant, 1_000_000_000).await?;


    let result = try_vote(&mut context, &participant, proposal_account_pubkey, true).await;

    assert_custom_error(
        result,
        GovernanceError::InsufficientTokens,
        "Expected InsufficientTokens",
    );

    Ok(())
}

#[tokio::test]
async fn test_vote_after_deadline_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let description = "Vote after deadline".to_string();
    let proposal_types = vec!["SystemUpgrade".to_string()];
    
    // Derivar pubkey antecipadamente
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_pubkey, _) = derive_proposal_account(&context.program_id, current_count);

    let current_timestamp = context.get_current_unix_timestamp().await;
    let start_time = Some(current_timestamp);
    let duration = Some(60); // 1 min

    try_create_proposal(
        &mut context,
        &admin,
        description.clone(),
        proposal_types.clone(),
        start_time,
        duration,
    )
    .await
    .expect("Proposal creation should succeed");

    // ⏩ Avançar no tempo
    context.warp_forward_seconds(61).await;

    // Criar votante com tokens
    let voter = Keypair::new();
    create_user_ata(&mut context, &voter).await?;
    fund_lamports(&mut context, &voter, 5_000_000).await?;
    
    context.mint_tokens(&admin, 1_000_000).await;
    context.transfer_tokens_to_user(&admin, &voter.pubkey(), 1_000_000).await?;

    // Votar após deadline
    let result = try_vote(&mut context, &voter, proposal_pubkey, true).await;
    assert_custom_error(result, GovernanceError::VotingPeriodEnded, "Expected VotingPeriodEnded");

    Ok(())
}



