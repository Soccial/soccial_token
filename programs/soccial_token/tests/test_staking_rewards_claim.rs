use anchor_lang::AccountDeserialize;
use soccial_token::staking::StakingState;
use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::{
    signature::Keypair, signer::Signer, transport::TransportError
};
use soccial_token::staking::error::StakingErrorCode;
use testutils::basics::derive_seeds;

mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::testutils::basics::assert_custom_error;
use crate::trymethods::trystaking::*;
use chrono::{TimeZone, Utc};


#[tokio::test]
#[ignore]
async fn test_warp_to_slot() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let slot_delta = 216_000; // 1 day

    // Get slot before advancing
    let initial_slot = context.banks_client.get_root_slot().await.unwrap();

    // Advance 100 slots
    context.warp_forward_slots(slot_delta).await;
        
    // Get current slot after warp
    let current_slot = context.banks_client.get_root_slot().await.unwrap();

    let formatted = context.estimate_datetime_from_slot_formatted(current_slot).await;
    println!("🕒 Future datetime for slot {current_slot}: {formatted}");

    // Assert we advanced the correct number of slots
    assert_eq!(current_slot, initial_slot + slot_delta);

    let current_ts = context.get_current_unix_timestamp().await;
    let datetime = Utc.timestamp_opt(current_ts, 0).unwrap();
    println!("🕒 Clock time (UTC): {}", datetime.format("%Y-%m-%d %H:%M:%S"));

    Ok(())
    
}


#[tokio::test]
async fn claim_staking_rewards_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    // Create participant and fund with SOL
    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 200_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Get current stake_id before staking
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("staking_state must exist");

    let staking_state = StakingState::try_deserialize(&mut &staking_state_account.data[..])
        .expect("Failed to deserialize staking_state");

    let stake_id = staking_state.last_id;

    // Perform the stake
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    // Simulate time passing for reward maturation (~1 year)
    let seconds = 60 * 60 * 24 * 368;
    context.warp_forward_seconds(seconds).await;    

    //(&mut context).send_debug_log().await?;
    
    // Check current slot just for logging/debug
    let current_slot = context.banks_client.get_root_slot().await.unwrap();
    let formatted = context.estimate_datetime_from_slot_formatted(current_slot).await;
    println!("🕒 Future datetime for slot {current_slot}: {formatted}");
        
    // Claim rewards (should succeed now that time passed)
    try_claim_staking_rewards(&mut context, &participant, &participant.pubkey(), stake_id).await?;

    Ok(())
}


#[tokio::test]
async fn claim_staking_rewards_by_owner_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    // Create participant and fund with SOL
    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Mint tokens and send to participant
    context.mint_tokens(&owner, 200_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Get current stake_id before staking
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("staking_state must exist");

    let staking_state = StakingState::try_deserialize(&mut &staking_state_account.data[..])
        .expect("Failed to deserialize staking_state");

    let stake_id = staking_state.last_id;

    // Perform the stake
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    // Simulate time passing for reward maturation (~1 year)
    let seconds = 60 * 60 * 24 * 368;
    context.warp_forward_seconds(seconds).await;    


    //(&mut context).send_debug_log().await?;
    
    // Check current slot just for logging/debug
    let current_slot = context.banks_client.get_root_slot().await.unwrap();
    let formatted = context.estimate_datetime_from_slot_formatted(current_slot).await;
    println!("🕒 Future datetime for slot {current_slot}: {formatted}");
        
    // Claim rewards (should succeed now that time passed)
    try_claim_staking_rewards(&mut context, &owner, &participant.pubkey(), stake_id).await?;

    Ok(())
}


#[tokio::test]
async fn claim_staking_rewards_should_fail_if_nothing_to_claim() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    context.mint_tokens(&owner, 200_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens but do not wait for reward maturation
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;

    // Attempt to claim rewards prematurely
    let result = try_claim_staking_rewards(&mut context, &participant, &participant.pubkey(), 1).await;

    assert_custom_error(result, StakingErrorCode::StakingPeriodNotOver, "Expected failure when no rewards are available");

    Ok(())
}

#[tokio::test]
async fn claim_staking_rewards_should_fail_if_unauthorized() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let participant = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    context.mint_tokens(&owner, 200_000_000).await;
    context.transfer_tokens_to_user(&owner, &participant.pubkey(), 10_000_000).await?;

    // Stake tokens with the legitimate participant
    try_stake_tokens(&mut context, &owner, &participant, 10_000, 1).await?;
    context.warp_forward_seconds(600).await;

    // Intruder tries to claim rewards
    let result = try_claim_staking_rewards(&mut context, &intruder, &participant.pubkey(), 1).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected failure due to unauthorized access");

    Ok(())
}

#[tokio::test]
async fn claim_staking_rewards_should_fail_if_not_staked() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let user = Keypair::new();
    create_user_ata(&mut context, &user).await?;
    fund_lamports(&mut context, &user, 5_000_000).await?;

    // User tries to claim without ever staking
    let result = try_claim_staking_rewards(&mut context, &user, &user.pubkey(), 1).await;

    assert!(result.is_err(), "Expected error, but transaction succeeded");

    Ok(())
}
