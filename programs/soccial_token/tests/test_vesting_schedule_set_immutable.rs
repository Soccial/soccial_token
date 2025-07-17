use core::result::Result;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer}, transport::TransportError
};
use soccial_token::{utils::error::ErrorCode, vesting::VestingState};
use testutils::{basics::assert_custom_error, environment::{create_user_ata, fund_lamports}};
mod testutils;
mod trymethods;
use crate::testutils::{basics::derive_seeds, environment::setup_test_env};
use crate::trymethods::tryvesting::*;
use anchor_lang::AccountDeserialize;

#[tokio::test]
async fn test_set_vesting_immutable_should_succeed() -> Result<(), TransportError> {
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
        immutable,
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

    // Execute set_vesting_immutable
    try_set_vesting_immutable(&mut context, &owner, &participant.pubkey(), vesting_id).await?;

    // Query the vesting schedule and verify that it's now immutable
    //let vesting_schedule = fetch_vesting_schedule(&mut context, &participant.pubkey(), 1).await;
    //assert!(vesting_schedule.immutable, "Vesting schedule should be immutable");

    Ok(())
}


#[tokio::test]
async fn test_set_vesting_immutable_should_fail_not_found() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        0, 0, 0, 1, 0, 100_000_000, false,
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
    
    // Attempt to call set_immutable without permission
    // It will return a AccountNotInitialized Error because it will not find the account with the combination vesting_id + intruder.pubkey()
    let result = try_set_vesting_immutable(&mut context, &owner, &intruder.pubkey(), vesting_id).await;
     assert!(
        result.is_err(),
        "🚨 Expected failure. Probably the Error Code: AccountNotInitialized Error."
    );

    Ok(())
}


#[tokio::test]
async fn test_set_vesting_immutable_should_fail_without_permission() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let participant = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    try_create_vesting_schedule(
        &mut context,
        &owner,
        &participant.pubkey(),
        0, 0, 0, 1, 0, 100_000_000, false,
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
    
    
    // Attempt to call set_immutable without permission
    let result = try_set_vesting_immutable(&mut context, &intruder, &participant.pubkey(), vesting_id).await;
    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error.");

    Ok(())
}
