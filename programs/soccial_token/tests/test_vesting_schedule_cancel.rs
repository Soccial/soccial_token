use core::result::Result;
use anchor_lang::AccountDeserialize;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::{self};
use testutils::environment::{create_user_ata, fund_lamports, setup_test_env};
mod testutils;
mod trymethods;
use crate::trymethods::tryvesting::*;

/// ✅ Should succeed in cancelling a vesting schedule
#[tokio::test]
async fn test_cancel_vesting_schedule_should_succeed() -> Result<(), TransportError> {
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

    // Create schedule first
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

    // Derive PDA
    let (vesting_state_pda, _) = Pubkey::find_program_address(
        &[b"vesting_state"],
        &context.program_id,
    );
    let vesting_state_account = context.banks_client.get_account(vesting_state_pda).await?.unwrap();
    let state = soccial_token::vesting::state::VestingState::try_deserialize(
        &mut &vesting_state_account.data[..]
    ).expect("Failed to deserialize VestingState account");

    let vesting_id = state.last_id - 1;

    // Cancel the vesting schedule
    try_cancel_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        vesting_id,
    ).await
    .expect("Vesting cancellation should succeed");

    Ok(())
}

#[tokio::test]
async fn test_cancel_vesting_schedule_should_fail_with_invalid_vesting_id() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    // Intentionally skip creation, and cancel a non-existing vesting_id
    let result = try_cancel_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        99999, // non-existent vesting_id
    ).await;

    assert!(result.is_err(), "Should fail due to non-existent vesting ID");

    Ok(())
}

#[tokio::test]
async fn test_cancel_vesting_schedule_should_fail_without_permission() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let malicious_user = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &malicious_user, 5_000_000).await?;

    let start_time = 0;
    let cliff_duration = 0;
    let cycles = 0;
    let vesting_duration = 1;
    let initial_tokens = 0;
    let total_tokens = 100_000_000;
    let immutable = false;

    // Create schedule with owner
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

    // Derive vesting_id
    let (vesting_state_pda, _) = Pubkey::find_program_address(
        &[b"vesting_state"],
        &context.program_id,
    );
    let vesting_state_account = context.banks_client.get_account(vesting_state_pda).await?.unwrap();
   
    let state = soccial_token::vesting::state::VestingState::try_deserialize(
        &mut &vesting_state_account.data[..]
    ).expect("Failed to deserialize VestingState account");

    let vesting_id = state.last_id - 1;

    // Try to cancel with unauthorized user
    let result = try_cancel_vesting_schedule(
        &mut context,
        &malicious_user,
        &participant.pubkey(),
        vesting_id,
    ).await;

    assert!(
        result.is_err(),
        "Should fail due to missing permissions from unauthorized caller"
    );

    Ok(())
}
