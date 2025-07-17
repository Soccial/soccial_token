use soccial_token::utils::error::ErrorCode;
use soccial_token::vaults::VaultError;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;
use crate::testutils::basics::assert_custom_error;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::trymarket::*;
//use crate::trymethods::tryvaults::try_deposit_offchain_reserve_vault;

 
/// applying a fee and distributing it between rewards and revenue vaults.
#[tokio::test]
async fn test_deposit_tokens_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let user = Keypair::new();
    create_user_ata(&mut context, &user).await?;
    context.mint_tokens(&user, 100_000_000_000).await;

    let amount = 12_000_000_000;
    let fee_bps = 300; // 3%

    //context.mint_tokens(&admin, 100_000_000_000).await;
    //try_deposit_offchain_reserve_vault(&mut context, &admin, 100_000_000_000).await?;

    context.mint_tokens_to_vault("offchain_reserve", 100_000_000_000).await?;

    // Get fee config from TokenState
    let state = context.load_token_state().await;
    let rewards_fee_bps = state.fee.rewards_fee_bps as u64;
    let airdrop_fee_bps = state.fee.airdrop_fee_bps as u64;

    // Compute total fee and sub-distributions
    let fee = amount * fee_bps as u64 / 10_000;
    let to_user = amount - fee;
    let to_rewards = fee * rewards_fee_bps / 10_000;
    let to_airdrop = fee * airdrop_fee_bps / 10_000;
    let to_revenue = fee - to_rewards - to_airdrop;

    // Get balances before deposit
    let user_before = context.get_user_balance(&user.pubkey()).await;
    let vault_before = context.get_vault_balance("offchain_reserve").await;
    let rewards_before = context.get_vault_balance("rewards").await;
    let airdrop_before = context.get_vault_balance("airdrop").await;
    let revenue_before = context.get_vault_balance("revenue").await;
   
    // Perform deposit
    try_deposit_tokens(&mut context, &admin, &user.pubkey(), amount, fee_bps).await?;
    
    context.refresh().await;

    // Get balances after deposit
    let user_after = context.get_user_balance(&user.pubkey()).await;
    let vault_after = context.get_vault_balance("offchain_reserve").await;
    let rewards_after = context.get_vault_balance("rewards").await;
    let airdrop_after = context.get_vault_balance("airdrop").await;
    let revenue_after = context.get_vault_balance("revenue").await;

    // Assertions
    assert_eq!(
        user_after - user_before,
        to_user,
        "❌ User should receive {} tokens, but received {}",
        to_user,
        user_after - user_before
    );

    assert_eq!(
        vault_before - vault_after,
        amount,
        "❌ offchain reserve vault should decrease by exactly {} tokens, but decreased by {}",
        amount,
        vault_before - vault_after
    );

    assert_eq!(
        rewards_after - rewards_before,
        to_rewards,
        "❌ Rewards vault should receive {} tokens, but received {}",
        to_rewards,
        rewards_after - rewards_before
    );

    assert_eq!(
        airdrop_after - airdrop_before,
        to_airdrop,
        "❌ Airdrop vault should receive {} tokens, but received {}",
        to_airdrop,
        airdrop_after - airdrop_before
    );

    assert_eq!(
        revenue_after - revenue_before,
        to_revenue,
        "❌ Revenue vault should receive {} tokens, but received {}",
        to_revenue,
        revenue_after - revenue_before
    );

    log_all_balances(&mut context, &admin).await?;
    Ok(())
}

/// ❌ Should fail if trying to deposit more than available in the vault.
#[tokio::test]
async fn test_deposit_tokens_insufficient_vault_balance_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let user = Keypair::new();
    create_user_ata(&mut context, &user).await?;

    let result = try_deposit_tokens(&mut context, &admin, &user.pubkey(), 1_100_000_000_000_000_000, 0).await;

    assert_custom_error(result, VaultError::InsufficientVaultBalance, "Expected failure due to insufficient vault balance, but got success");

    Ok(())
}



#[tokio::test]
async fn test_deposi_without_permission_should_fail() -> Result<(), TransportError> {

    let (mut context, _admin) = setup_test_env().await;

    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;

     let result = try_deposit_tokens(&mut context, &intruder, &intruder.pubkey(), 10_100, 0).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error when withdrawing from vault.");

    
    Ok(())
}