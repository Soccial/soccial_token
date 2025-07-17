// ======================================================================
// Test file: test_vesting_schedule_update
// ======================================================================

// This test file corresponds to the `update_vesting_schedule` function in the Soccial Token contract.
use core::result::Result;
use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::{self, utils::error::ErrorCode};
use testutils::{basics::assert_custom_error, environment::{create_user_ata, fund_lamports, setup_test_env}};
mod testutils;
mod trymethods;
use crate::trymethods::tryvesting::*;

#[tokio::test]
async fn test_update_vesting_schedule_should_succeed() -> Result<(), TransportError> {
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

    // Create schedule
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

    // Get the correct vesting_id
    let (vesting_state_pda, _) = Pubkey::find_program_address(&[b"vesting_state"], &context.program_id);
    let vesting_state_account = context.banks_client.get_account(vesting_state_pda).await?.unwrap();
    let state = soccial_token::vesting::state::VestingState::try_deserialize(&mut &vesting_state_account.data[..]).unwrap();
    let vesting_id = state.last_id - 1;


      let vesting_duration = 20;
       let cliff_duration = 10;

    // Call update
    try_update_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        vesting_id,
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await?;

    // Fetch vesting schedule and assert values
    let pda = derive_vesting_schedule_pda(&context.program_id, &participant.pubkey(), vesting_id);
    let account = context.banks_client.get_account(pda).await?.unwrap();
    let schedule = soccial_token::vesting::state::VestingSchedule::try_deserialize(&mut &account.data[..]).unwrap();

    assert_eq!(schedule.vesting_duration, 20);
    assert_eq!(schedule.cliff_duration, 10);

    Ok(())
}

#[tokio::test]
async fn test_update_vesting_schedule_should_fail_with_wrong_id() -> Result<(), TransportError> {
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


    // Create a valid vesting
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


    let start_time = 999;
    let vesting_id = 12;

    // Use a non-existent vesting_id 
    let result = try_update_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        vesting_id,
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await;

    assert!(result.is_err(), "Update should fail with invalid vesting_id");

    Ok(())
}

#[tokio::test]
async fn test_update_vesting_schedule_should_fail_without_permission() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

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

    let (vesting_state_pda, _) = Pubkey::find_program_address(&[b"vesting_state"], &context.program_id);
    let state_account = context.banks_client.get_account(vesting_state_pda).await?.unwrap();
    let state = soccial_token::vesting::state::VestingState::try_deserialize(&mut &state_account.data[..]).unwrap();
    let vesting_id = state.last_id - 1;

    let result = try_update_vesting_schedule(
        &mut context,
        &intruder,
        &participant.pubkey(),
        vesting_id,
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await;


    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error.");

    Ok(())
}
