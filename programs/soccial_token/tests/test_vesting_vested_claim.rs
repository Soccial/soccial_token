use core::result::Result;
use solana_program_test::*;
use anchor_lang::AccountDeserialize;
use solana_sdk::{
    signature::{Keypair, Signer}, transport::TransportError
};
use soccial_token::{self, utils::error::ErrorCode, vesting::{VestingErrorCode, VestingState}};
use testutils::{basics::assert_custom_error, environment::{create_user_ata, fund_lamports}};
mod testutils;
mod trymethods;
use crate::testutils::{basics::derive_seeds, environment::setup_test_env};
use crate::trymethods::tryvesting::*;

#[tokio::test]
async fn test_claim_vested_tokens_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let start_time = 0;
    let cliff_duration = 0;
    let cycles = 0;
    let vesting_duration = 1;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    let before_vault_balance = context.get_vault_balance("vesting").await;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

    // Get the last vesting_id;
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("vesting state missing");

    let vesting_state = VestingState::try_deserialize(&mut &vesting_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let vesting_id = vesting_state.last_id - 1;


    context.warp_forward_slots(150).await;


    let during_vault_balance = context.get_vault_balance("vesting").await;
    let before_user_balance = context.get_user_balance(&participant.pubkey()).await;

   assert_eq!(
        before_vault_balance + total_tokens, during_vault_balance,
        "Vault should have exactly {} tokens during the vesting.",
        during_vault_balance
    );

    try_claim_vested_tokens(
        &mut context,
        &participant,
        &participant.pubkey(),
        vesting_id,
    ).await?;

    context.refresh().await;

    let after_vault_balance = context.get_vault_balance("vesting").await;

    let after_user_balance = context.get_user_balance(&participant.pubkey()).await;


    println!("🔒📦 before_vault_balance: {} during_vault_balance: {} after_vault_balance: {} / before_user_balance: {} - after_user_balance: {} - / total_tokens: {}", before_vault_balance, during_vault_balance, after_vault_balance, before_user_balance, after_user_balance, total_tokens);

    assert_eq!(
        after_user_balance, total_tokens,
        "User should receive exactly {} tokens",
        total_tokens
    );
  
    assert_eq!(
        during_vault_balance - after_vault_balance, total_tokens,
        "Vault should decrease exactly by {} tokens",
        total_tokens
    );

    Ok(())
}


#[tokio::test]
async fn test_claim_vested_tokens_middle_period_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let start_time = 0;
    let cliff_duration = 0;
    let cycles = 3;
    let vesting_duration = 2_592_000;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

     // Get the last vesting_id;
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("vesting state missing");

    let vesting_state = VestingState::try_deserialize(&mut &vesting_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let vesting_id = vesting_state.last_id - 1;

    context.warp_forward_seconds(60 * 60 * 24 * 10).await;


    let during_vault_balance = context.get_vault_balance("vesting").await;
    let before_user_balance = context.get_user_balance(&participant.pubkey()).await;

    try_claim_vested_tokens(
        &mut context,
        &participant,
        &participant.pubkey(),
        vesting_id,
    ).await?;

    context.refresh().await;

    let after_vault_balance = context.get_vault_balance("vesting").await;

    let after_user_balance = context.get_user_balance(&participant.pubkey()).await;


    println!("🔒📦 during_vault_balance: {} after_vault_balance: {} / before_user_balance: {} - after_user_balance: {} - / total_tokens: {}", during_vault_balance, after_vault_balance, before_user_balance, after_user_balance, total_tokens);

    /* 
    assert_eq!(
        after_user_balance, total_tokens,
        "User should receive exactly {} tokens",
        total_tokens
    );
  
    assert_eq!(
        during_vault_balance - after_vault_balance, total_tokens,
        "Vault should decrease exactly by {} tokens",
        total_tokens
    );
    */

    Ok(())
}


#[tokio::test]
async fn test_claim_vested_tokens_by_owner_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;


    let start_time = 0;
    let cliff_duration = 0;
    let cycles = 0;
    let vesting_duration = 1;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

     // Get the last vesting_id;
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("vesting state missing");

    let vesting_state = VestingState::try_deserialize(&mut &vesting_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let vesting_id = vesting_state.last_id - 1;

    context.warp_forward_slots(150).await;

    try_claim_vested_tokens(
        &mut context,
        &owner,
        &participant.pubkey(),
        vesting_id,
    ).await?;

    let user_balance = context.get_user_balance(&participant.pubkey()).await;

    assert_eq!(
        user_balance, total_tokens,
        "User should receive exactly {} tokens",
        total_tokens
    );

    Ok(())
}

#[tokio::test]
async fn test_claim_vested_tokens_should_fail_without_permission() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let cliff_duration = 0;
    let cycles = 0;
    let start_time = 0;
    let vesting_duration = 1;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

     // Get the last vesting_id;
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("vesting state missing");

    let vesting_state = VestingState::try_deserialize(&mut &vesting_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let vesting_id = vesting_state.last_id - 1;

    let result = try_claim_vested_tokens(
        &mut context,
        &intruder,
        &participant.pubkey(),
        vesting_id
    ).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error.");

    Ok(())
}

#[tokio::test]
async fn test_claim_vested_tokens_should_fail_if_locked() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // TODO - test cliff


    let now = context.get_current_unix_timestamp().await;

     let cliff_duration = 0;
    let cycles = 0;
    let start_time = now.try_into().unwrap();
    let vesting_duration = 10000;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

     // Get the last vesting_id;
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("vesting state missing");

    let vesting_state = VestingState::try_deserialize(&mut &vesting_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let vesting_id = vesting_state.last_id - 1;

    let result = try_claim_vested_tokens(
        &mut context,
        &participant,
        &participant.pubkey(),
        vesting_id,
    ).await;

    assert_custom_error(result, VestingErrorCode::NoTokensToRelease, "Expected NoTokensToRelease error.");

    Ok(())
}
