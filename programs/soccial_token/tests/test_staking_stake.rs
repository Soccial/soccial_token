use anchor_lang::AccountDeserialize;
use soccial_token::staking::StakingState;
use soccial_token::{self, staking::state::StakingAccount, staking::error::StakingErrorCode, utils::error::ErrorCode};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

mod testutils;
mod trymethods;

use crate::testutils::environment::*;
use crate::testutils::basics::{assert_custom_error, derive_seeds};
use crate::trymethods::trystaking::*;
use crate::trymethods::tryvaults::{try_transfer_between_vaults_without_funding_test};

#[tokio::test]
async fn stake_tokens_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    context.mint_tokens(&owner, 200_000_000).await;
    context
        .transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000)
        .await?;

    let amount = 10_000;

    // Perform staking
    try_stake_tokens(&mut context, &owner, &participant, amount, 1).await?;

    // Load staking_state to get last_id (after increment)
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState must exist");

    let staking_state = StakingState::try_deserialize(&mut &staking_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let stake_id = staking_state.last_id - 1;

    // Derive correct PDA for staking_account
    let staking_account_pubkey = derive_staking_account_pda(
        &context.program_id,
        &participant.pubkey(),
        stake_id,
    );

    let account = context
        .banks_client
        .get_account(staking_account_pubkey)
        .await?
        .expect("StakingAccount must exist");

    let stake = StakingAccount::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize");

    assert_eq!(stake.participant, participant.pubkey());
    assert_eq!(stake.plan_id, 1);
    assert_eq!(stake.staked_tokens, amount);
    assert!(!stake.withdrawn);

    Ok(())
}


#[tokio::test]
async fn stake_tokens_should_fail_with_invalid_plan() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    context.mint_tokens(&owner, 100_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 100_000).await?;

    let result = try_stake_tokens(&mut context, &owner, &participant, 1000, 99).await;

    assert_custom_error(result, StakingErrorCode::InvalidStakingPlan, "Expected failure with invalid plan ID");

    Ok(())
}

#[tokio::test]
async fn stake_tokens_should_fail_with_insufficient_balance() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let result = try_stake_tokens(&mut context, &owner, &participant, 10_000_000_000_000_000_000, 2).await;

    assert_custom_error(result, StakingErrorCode::InsufficientUserBalance, "Expected failure due to insufficient token balance");

    Ok(())
}

#[tokio::test]
async fn stake_tokens_should_fail_with_insufficient_liquidity() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    // Create participant
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 100_000_000).await?;

    // Mint tokens to participant
    context.mint_tokens(&owner, 100_000_200_000_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 100_000_200_000_000_000).await?;

    // Fetch reserved supply amount
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let account = context
        .banks_client
        .get_account(seeds.liquidity_vault_token_account)
        .await?
        .expect("Reserved supply vault token account must exist");

    let liquidity = anchor_spl::token::TokenAccount::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize token account")
        .amount;

    assert!(liquidity > 0, "Reserved supply must be > 0 for this test");
    
    // Carefully calculate a required stake to empty reserved supply
    let liquidity = anchor_spl::token::TokenAccount::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize token account")
        .amount;

    println!("📦 Reserved supply in vault: {}", liquidity);
    

    try_transfer_between_vaults_without_funding_test(&mut context, &owner, "liquidity_vault", "reserved_supply_vault", liquidity, None).await;

    // Attempt to stake and expect failure
    let result = try_stake_tokens(&mut context, &owner, &participant, 100_000_000, 3).await;
    
    assert_custom_error(
        result,
        StakingErrorCode::InsufficientVaultBalance,
        "Expected failure due to insufficient reward coverage",
    );

    Ok(())
}


#[tokio::test]
async fn stake_tokens_should_fail_unauthorized() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let result = try_stake_tokens(&mut context, &intruder, &intruder, 5000, 4).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected failure due to missing permission");

    Ok(())
}
