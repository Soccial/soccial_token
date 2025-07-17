use anchor_lang::AccountDeserialize;
use soccial_token::utils::error::ErrorCode;
use soccial_token::{self, staking::state::StakingAccount, staking::error::StakingErrorCode};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};


mod testutils;
mod trymethods;

use crate::testutils::environment::*;
use crate::testutils::basics::{assert_custom_error, derive_seeds};
use crate::trymethods::trystaking::*;

#[tokio::test]
async fn add_to_stake_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let user = Keypair::new();

    create_user_ata(&mut context, &user).await?;
    fund_lamports(&mut context, &user, 5_000_000).await?;

    // Mint tokens to user
    context.mint_tokens(&admin, 100_000_000).await;
    context.transfer_tokens_to_user(&admin, &user.pubkey(), 10_000_000).await?;

    // Stake tokens first
    let initial_amount = 10_000;
    let plan_id = 1;
    try_stake_tokens(&mut context, &admin, &user, initial_amount, plan_id).await?;

    // Add more to stake
    let reinforce_amount = 5_000;
    try_add_to_stake(&mut context, &admin, &user, 1, reinforce_amount).await?;

    // Verify staking account updated
    let _seeds = derive_seeds(&context.program_id, &user.pubkey());
    let staking_account_pubkey =
        derive_staking_account_pda(&context.program_id, &user.pubkey(), 1);

    let account = context
        .banks_client
        .get_account(staking_account_pubkey)
        .await?
        .expect("Missing staking account");

    let stake = StakingAccount::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize StakingAccount");

    assert_eq!(stake.participant, user.pubkey());
    assert_eq!(stake.staked_tokens, initial_amount + reinforce_amount);
    assert_eq!(stake.plan_id, plan_id);
    assert!(!stake.withdrawn);

    Ok(())
}

#[tokio::test]
async fn add_to_stake_if_user_has_no_tokens_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let user = Keypair::new();

    create_user_ata(&mut context, &user).await?;
    fund_lamports(&mut context, &user, 5_000_000).await?;

    context.mint_tokens(&admin, 100_000_000).await;
    context.transfer_tokens_to_user(&admin, &user.pubkey(), 1_000).await?;

    try_stake_tokens(&mut context, &admin, &user, 1_000, 1).await?;

    let result = try_add_to_stake(&mut context, &admin, &user, 1, 10_000).await;

    assert_custom_error(
        result,
        StakingErrorCode::InsufficientUserBalance,
        "Expected failure due to low user token balance",
    );

    Ok(())
}

#[tokio::test]
async fn add_to_stake_without_permissions_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let user = Keypair::new();

    create_user_ata(&mut context, &user).await?;
    fund_lamports(&mut context, &user, 5_000_000).await?;

    context.mint_tokens(&admin, 100_000_000).await;
    context.transfer_tokens_to_user(&admin, &user.pubkey(), 100_000_000).await?;

    try_stake_tokens(&mut context, &admin, &user, 10_000, 1).await?;

    let result = try_add_to_stake(&mut context, &user, &user, 1, 10_000).await;

    assert_custom_error(
        result,
        ErrorCode::Unauthorized,
        "Expected failure due to unnauthorized access",
    );

    Ok(())
}


#[tokio::test]
async fn add_to_stake_wrong_stake_account_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let user = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &user).await?;
    fund_lamports(&mut context, &user, 5_000_000).await?;

    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    context.mint_tokens(&admin, 200_000_000).await;
    context.transfer_tokens_to_user(&admin, &user.pubkey(), 100_000_000).await?;
    context.transfer_tokens_to_user(&admin, &intruder.pubkey(), 100_000_000).await?;

    try_stake_tokens(&mut context, &admin, &user, 10_000, 1).await?;
    try_stake_tokens(&mut context, &admin, &intruder, 10_000, 2).await?;

    let result = try_add_to_stake(&mut context, &admin, &intruder, 1, 10_000).await;
 
    assert!(
        result.is_err(),
        "Expected failure due to unnauthorized staking accout (should return AccountNotInitialized because there's no account with the staking_id + user combination).",
    );

    Ok(())
}
