use anchor_lang::AccountDeserialize;
use soccial_token::staking::StakingState;
use soccial_token::{self, staking::state::StakingAccount, staking::error::StakingErrorCode};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};


mod testutils;
mod trymethods;

use crate::testutils::environment::*;
use crate::testutils::basics::{assert_custom_error, derive_seeds};
use crate::trymethods::trystaking::*;

#[tokio::test]
async fn buy_and_stake_tokens_should_succeed() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let amount = 10_000;
    let plan_id = 1;
    
    try_buy_and_stake_tokens(&mut context, &caller, &participant.pubkey(), amount, plan_id).await?;

    // Check staking account
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("staking state missing");

    let staking_state = StakingState::try_deserialize(&mut &staking_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let stake_id = staking_state.last_id - 1;

    let staking_account_pubkey =
        derive_staking_account_pda(&context.program_id, &participant.pubkey(), stake_id);

    let account = context
        .banks_client
        .get_account(staking_account_pubkey)
        .await?
        .expect("staking account missing");

    let stake = StakingAccount::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize staking state");

    assert_eq!(stake.participant, participant.pubkey());
    assert_eq!(stake.plan_id, plan_id);
    assert_eq!(stake.staked_tokens, amount);
    assert!(!stake.withdrawn);

    Ok(())
}

#[tokio::test]
async fn buy_and_stake_tokens_should_fail_with_invalid_plan() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 1_000_000).await?;

    let amount = 5_000;

    let result = try_buy_and_stake_tokens(
        &mut context,
        &caller,
        &participant.pubkey(),
        amount,
        99, // invalid plan_id
    )
    .await;

    assert_custom_error(
        result,
        StakingErrorCode::InvalidStakingPlan,
        "Expected failure with invalid plan ID",
    );

    Ok(())
}

#[tokio::test]
async fn buy_and_stake_tokens_should_fail_with_insufficient_liquidity() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let participant = Keypair::new();
       create_user_ata(&mut context, &participant).await?;
    
    // Do NOT fund liquidity vault on purpose
    let amount = 100_000_000_000_000_100;
    let plan_id = 1;

    let result = try_buy_and_stake_tokens(
        &mut context,
        &admin,
        &participant.pubkey(),
        amount,
        plan_id,
    )
    .await;

    assert_custom_error(
        result,
        StakingErrorCode::InsufficientVaultBalance,
        "Expected failure due to insufficient liquidity",
    );

    Ok(())
}
