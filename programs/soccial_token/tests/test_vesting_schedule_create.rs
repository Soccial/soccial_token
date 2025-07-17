// ======================================================================
// Test file: test_create_vesting_schedule.rs
// ======================================================================

use core::result::Result;
use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::{Keypair, Signer}, transport::TransportError
};
use soccial_token::{self};
use testutils::environment::{create_user_ata, fund_lamports};
mod testutils;
mod trymethods;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryvesting::*;

#[tokio::test]
async fn test_create_vesting_schedule_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let cliff_duration = 0;
    let cycles = 0;
    let start_time = 0;
    let vesting_duration = 5;
    let initial_tokens = 0;
    let total_tokens = 1000;
    let immutable = false;

    // Derive vesting_state PDA
    let (vesting_state_pda, _) = Pubkey::find_program_address(
        &[b"vesting_state"],
        &context.program_id,
    );

    // Fetch current vesting_id BEFORE executing the instruction
    let vesting_state_account_before = context
        .banks_client
        .get_account(vesting_state_pda)
        .await?
        .expect("VestingState account should exist");

    let vesting_state_data_before = soccial_token::vesting::state::VestingState::try_deserialize(
        &mut &vesting_state_account_before.data[..],
    )
    .expect("Failed to deserialize VestingState account");

    let vesting_id = vesting_state_data_before.last_id;

    // Derive the expected vesting_schedule PDA based on the vesting_id BEFORE increment
    let vesting_schedule_pda = derive_vesting_schedule_pda(
        &context.program_id,
        &participant.pubkey(),
        vesting_id,
    );

    // Call the vesting creation function (this will increment the vesting_id)
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
    )
    .await
    .expect("Vesting Schedule creation should succeed");

    // Fetch the created VestingSchedule account
    let account = context
        .banks_client
        .get_account(vesting_schedule_pda)
        .await?
        .expect("VestingSchedule account should exist");

    let vesting_schedule = soccial_token::vesting::state::VestingSchedule::try_deserialize(
        &mut &account.data[..],
    )
    .expect("Failed to deserialize VestingSchedule account");

    assert_eq!(
        vesting_schedule.participant,
        participant.pubkey(),
        "Participant pubkey should match"
    );

    assert_eq!(
        vesting_schedule.vesting_id,
        vesting_id,
        "Vesting ID should match"
    );

    assert_eq!(
        vesting_schedule.total_tokens,
        total_tokens,
        "Total tokens should match"
    );
    assert_eq!(
        vesting_schedule.cliff_duration,
        cliff_duration as i64,
        "Cliff duration should match"
    );
    assert_eq!(
        vesting_schedule.vesting_duration,
        vesting_duration as i64,
        "Vesting duration should match"
    );

    Ok(())
}


/*
#[tokio::test]
async fn test_create_vesting_schedule_should_fail_with_invalid_pubkey() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    // Use a fake participant (not registered or funded)
    let fake_participant = Pubkey::new_unique();

    let result = try_create_vesting_schedule(
        &mut context,
        &owner,
        &fake_participant,
        0,
        0,
        5,
        1000,
    ).await;

    assert!(
        result.is_err(),
        "Should fail with a participant Pubkey that is not prepared"
    );
    Ok(())
}
 */

#[tokio::test]
async fn test_create_vesting_schedule_should_fail_without_permission() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;
    let unauthorized_user = Keypair::new();
    let participant = Keypair::new();

    // Fund the unauthorized user and create the participant ATA
    fund_lamports(&mut context, &unauthorized_user, 5_000_000).await?;
    create_user_ata(&mut context, &participant).await?;

    let cliff_duration = 0;
    let cycles = 0;
    let start_time = 0;
    let vesting_duration = 5;
    let initial_tokens = 0;
    let total_tokens = 1000;
    let immutable = false;

    // Try to create a vesting schedule without proper permissions
    let result = try_create_vesting_schedule(
        &mut context,
        &unauthorized_user,
        &participant.pubkey(),
        start_time,
        cliff_duration,
        cycles,
        vesting_duration,
        initial_tokens,
        total_tokens,
        immutable
    ).await;

    assert!(
        result.is_err(),
        "Should fail when the caller is unauthorized"
    );
    Ok(())
}
