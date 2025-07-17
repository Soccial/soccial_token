use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transport::TransportError
};
use soccial_token::staking::error::StakingErrorCode;

mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::testutils::basics::assert_custom_error;
use crate::trymethods::trystaking::*;

/// Test: Withdraw should succeed after lockup period expires
#[tokio::test]
async fn withdraw_should_succeed_after_lockup() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    // Setup user ATA and fund with SOL
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 100_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    // Simulate 1 year passing
    let stake_id = 1;
    let seconds = 60 * 60 * 24 * 365 + 1;
    context.warp_forward_seconds(seconds).await;

    // Withdraw staked tokens and reward
    try_withdraw_staked_tokens(&mut context, &participant, &participant.pubkey(), stake_id).await?;

    Ok(())
}

/// Test: Withdraw should fail before lockup period is over
#[tokio::test]
async fn withdraw_should_fail_before_lockup() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    // Setup user ATA and fund with SOL
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 100_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens but do not wait
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    // Attempt early withdrawal
    let result = try_withdraw_staked_tokens(&mut context, &participant, &participant.pubkey(), 1).await;

    assert_custom_error(result, StakingErrorCode::StakingPeriodNotOver, "Expected failure before lockup ends");

    Ok(())
}

/// Test: Withdraw should fail if already withdrawn
#[tokio::test]
async fn withdraw_should_fail_if_already_done() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    // Setup user ATA and fund with SOL
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 100_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake and wait for lockup
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;
    context.warp_forward_seconds(60 * 60 * 24 * 366).await;

    // First withdrawal should succeed
    try_withdraw_staked_tokens(&mut context, &participant, &participant.pubkey(), 1).await?;
    context.refresh().await;

    // Second withdrawal should fail
    let _result = try_withdraw_staked_tokens(&mut context, &participant, &participant.pubkey(), 1).await;

     // This should now fail with AccountNotInitialized since staking_account was closed
    let result = try_withdraw_staked_tokens(&mut context, &participant, &participant.pubkey(), 1).await;

    assert!(
        result.is_err(),
        "Expected error on second withdrawal attempt, but it succeeded"
    );

    Ok(())
}

/// Test: Withdraw should fail if caller is not authorized
#[tokio::test]
async fn withdraw_should_fail_if_unauthorized() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let intruder = Keypair::new();

    // Setup ATAs and fund both users
    create_user_ata(&mut context, &participant).await?;
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 100_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens and wait for lockup
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    
    context.warp_forward_seconds(60 * 60 * 24 * 366).await;

    // Intruder attempts to withdraw
    let result = try_withdraw_staked_tokens(&mut context, &intruder, &participant.pubkey(), 1).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected failure due to unauthorized caller");

    Ok(())
}


#[tokio::test]
async fn withdraw_should_fail_if_unauthorized_even_if_owner() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    // Setup ATAs and fund both users
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 100_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens and wait for lockup
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    
    context.warp_forward_seconds(60 * 60 * 24 * 366).await;

    // Intruder attempts to withdraw
    let result = try_withdraw_staked_tokens(&mut context, &owner, &participant.pubkey(), 1).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected failure due to unauthorized caller");

    Ok(())
}

