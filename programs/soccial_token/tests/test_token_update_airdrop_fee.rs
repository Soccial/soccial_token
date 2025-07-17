use anchor_lang::AccountDeserialize;
use soccial_token::economics::EconomicsErrorCode;
use soccial_token::economy::fee::MAX_AIRDROP_FEE_BPS;
use soccial_token::governance::ProposalAccount;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, transport::TransportError};
use soccial_token::utils::error::ErrorCode;

mod testutils;
mod trymethods;

use crate::testutils::environment::setup_test_env;
use crate::trymethods::trygovernance::try_approve_proposal_flow;
use crate::trymethods::trytoken::*;
use crate::testutils::basics::*;

#[tokio::test]
async fn test_update_airdrop_fee_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;


    let proposal_types = vec!["UpdateAirdropFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing airdrop proposal".to_string(),
        proposal_types
    ).await?;
    
    context.refresh().await;

    // let's confirm this is working has expected. (it's the first time I am using this try
    /* 
    let seeds = derive_seeds(&context.program_id, &admin.pubkey());
    let current_count = get_proposal_last_id(&mut context, seeds.governance_state).await?;*/
    
    let (proposal_account_pubkey, _) = derive_proposal_account(&context.program_id, proposal_id);

    let account = context.banks_client.get_account(proposal_account_pubkey).await?.expect("Proposal account should exist");
    let proposal_state = ProposalAccount::try_deserialize(&mut &account.data[..]).expect("Failed to deserialize ProposalAccount");

    println!("✅ Proposal id: {}", proposal_state.id);
    println!("✅ description: {}", proposal_state.description);

    let new_fee = 123; // Valid BPS

    try_update_airdrop_fee(&mut context, &admin, vec![new_fee.to_string()], proposal_id).await.unwrap();

    let state = context.load_token_state().await;
    assert_eq!(state.fee.airdrop_fee_bps, new_fee, "Airdrop fee should be updated correctly");

    Ok(())
}


#[tokio::test]
async fn test_update_airdrop_fee_without_approval_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    // This need
    let proposal_id = 1;

    let new_fee = 123; // Valid BPS
    
    let result = try_update_airdrop_fee(&mut context, &admin, vec![new_fee.to_string()], proposal_id).await;

    assert!(
        result.is_err(),
        "🚨 Expected failure due to lack of governance approval."
    );

    Ok(())
}

#[tokio::test]
async fn test_update_airdrop_fee_exceeds_max_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let proposal_types = vec!["UpdateAirdropFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing proposal".to_string(),
        proposal_types
    ).await?;

    let invalid_fee = MAX_AIRDROP_FEE_BPS + 1;
    let result = try_update_airdrop_fee(&mut context, &admin, vec![invalid_fee.to_string()], proposal_id).await;

    assert_custom_error(result, EconomicsErrorCode::InvalidFeeValue, "Fee exceeding max should fail");

    Ok(())
}

#[tokio::test]
async fn test_update_airdrop_fee_unauthorized_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let proposal_types = vec!["UpdateAirdropFee".to_string()];
    let proposal_id = try_approve_proposal_flow(
        &mut context,
        &admin,
        "testing proposal".to_string(),
        proposal_types
    ).await?;

    let unauthorized = Keypair::new();
    fund_test_accounts_manual(&mut context.banks_client, &[&unauthorized], 10_000_000, &context.payer, &context.recent_blockhash).await;

    let result = try_update_airdrop_fee(&mut context, &unauthorized, vec!["222".to_string()], proposal_id).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Unauthorized user should not update airdrop fee");

    Ok(())
}
