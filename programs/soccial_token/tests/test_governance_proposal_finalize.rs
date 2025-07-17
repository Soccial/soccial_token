// ======================================================================
// Test: Finalize Proposal
// ======================================================================

use anchor_lang::AccountDeserialize;
use soccial_token::governance::ProposalAccount;
use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;
use testutils::basics::{assert_custom_error, derive_proposal_account, derive_seeds};
mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::trygovernance::*;

#[tokio::test]
async fn test_finalize_proposal_success() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Proposal to finalize".to_string();
    let proposal_types = vec!["StrategicDecision".to_string()];

    // Derive future proposal pubkey before creating
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_pubkey, _) = derive_proposal_account(&context.program_id, current_count);

    // Create the proposal
    try_create_proposal(&mut context, &owner, description.clone(), proposal_types.clone(), None, None)
        .await
        .expect("Proposal creation should succeed");

   // Simulate time passing for reward maturation (19 days)
    let seconds = 60 * 60 * 24 * 19;
    context.warp_forward_seconds(seconds).await;    

    // Finalize the proposal
    try_finalize_proposal(&mut context, &owner, proposal_pubkey)
        .await
        .expect("Finalization should succeed");

    // Verify proposal finalization status when field is added
    let account = context.banks_client.get_account(proposal_pubkey).await?.expect("Proposal account should exist");
    let proposal_state = ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Deserialization failed");
    assert!(proposal_state.is_finalized, "Proposal should be marked as finalized");

    Ok(())
}

#[tokio::test]
async fn test_finalize_proposal_unauthorized() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let description = "Unauthorized finalize attempt".to_string();
    let proposal_types = vec!["StrategicDecision".to_string()];

    let seeds = derive_seeds(&context.program_id, &owner.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;
    let (proposal_pubkey, _) = derive_proposal_account(&context.program_id, current_count);

    try_create_proposal(&mut context, &owner, description.clone(), proposal_types.clone(), None, None)
        .await
        .expect("Proposal creation should succeed");

    // Intruder
    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let result = try_finalize_proposal(&mut context, &intruder, proposal_pubkey).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error");

    Ok(())
}

#[tokio::test]
async fn test_finalize_nonexistent_proposal() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let fake_proposal_pubkey = Keypair::new().pubkey();

    let result = try_finalize_proposal(&mut context, &owner, fake_proposal_pubkey).await;

    assert!(result.is_err(), "Expected error when finalizing non-existent proposal");

    Ok(())
}
