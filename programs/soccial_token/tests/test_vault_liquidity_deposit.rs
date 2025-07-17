
/// ======================================================================
/// Soccial Token – Integration Tests: Liquidity Vault Deposits
///
/// These integration tests validate the logic for securely depositing
/// tokens into the Liquidity Vault. The tests ensure that balances are
/// updated correctly, unauthorized access is prevented, and invalid
/// input values (like zero tokens) are rejected.
///
/// Covered scenarios:
/// - ✅ Successful deposit by a valid user
/// - ✅ Accurate balance changes after deposit
/// - ❌ Rejection of deposits with insufficient balance
/// - ❌ Rejection of zero-value deposits
/// - ❌ Rejection of unauthorized callers
///
/// Author: Paulo Rodrigues  
/// Project: Soccial Token  
/// Website: https://www.soccial.com/thetoken  
/// ======================================================================

use soccial_token::utils::error::ErrorCode;
use soccial_token::vaults::VaultError;
use solana_program_test::*;
use solana_sdk::msg;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryvaults::*;

// ======================================================================
// TESTS
// ======================================================================

#[tokio::test]
async fn test_deposit_liquidity_vault_should_succeed() -> Result<(), TransportError> {
  
    let (mut context, admin) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;


    try_deposit_liquidity_vault(&mut context, &admin, &participant, 1_000).await?;

    msg!("✅ Rewards vault deposit successful.");
    
    log_all_balances(&mut context, &admin).await?;

    Ok(())
}


#[tokio::test]
async fn test_deposit_liquidity_balances_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;

    let deposit_amount = 1_000;

    // Balances before the deposit
    let liquidity_before = context.get_vault_balance("liquidity").await;
    let participant_before = context.get_user_balance(&participant.pubkey()).await;

    // Execute the deposit
    try_deposit_liquidity_vault(&mut context, &admin, &participant, deposit_amount).await?;

    context.refresh().await;

    // Balances after the deposit
    let liquidity_after = context.get_vault_balance("liquidity").await;
    let participant_after = context.get_user_balance(&participant.pubkey()).await;

    // Assert: vault should increase by the deposited amount
    assert_eq!(
        liquidity_after - liquidity_before,
        deposit_amount,
        "❌ Liquidity vault should increase by {} tokens, but increased by {}",
        deposit_amount,
        liquidity_after - liquidity_before
    );

    // Assert: participant should lose the exact amount
    assert_eq!(
        participant_before - participant_after,
        deposit_amount,
        "❌ Participant should lose {} tokens, but lost {}",
        deposit_amount,
        participant_before - participant_after
    );

    msg!("✅ Liquidity vault deposit successful and balances verified.");
    log_all_balances(&mut context, &admin).await?;

    Ok(())
}

#[tokio::test]
async fn test_deposit_liquidity_vault_without_tokens_should_fail() -> Result<(), TransportError> {
  
    let (mut context, admin) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;

    let result = try_deposit_liquidity_vault(&mut context, &admin, &participant, 1_000).await;

    assert!(
        result.is_err(),
        "🚨 Expected failure when depositing liquidity without tokens."
    );
    
    msg!("✅ Correctly failed deposit without owning tokens.");
    
    Ok(())
}

#[tokio::test]
async fn test_deposit_liquidity_vault_zero_tokens_should_fail() -> Result<(), TransportError> {
 

    let (mut context, admin) = setup_test_env().await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;

    let result = try_deposit_liquidity_vault(&mut context, &admin, &participant, 0).await;
    
    assert!(
        result.is_err(),
        "🚨 Expected failure when depositing liquidity without tokens."
    );

    assert_custom_error(
        result,
        VaultError::InvalidVaultAmount,
        "Expected failure when depositing 0 tokens.",
    );

    msg!("✅ Correctly failed deposit of zero tokens.");
    
    Ok(())
}

#[tokio::test]
async fn test_deposit_liquidity_vault_unauthorized_should_fail() -> Result<(), TransportError> {
  

    let (mut context, legit_admin) = setup_test_env().await;
    
    context.mint_tokens(&legit_admin, 10_000).await;

    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let result = try_deposit_liquidity_vault(&mut context, &intruder, &intruder, 1_000).await;
    
    assert_custom_error(
        result,
        ErrorCode::Unauthorized,
        "Expected failure when unauthorized user tried to deposit liquidity.",
    );

    msg!("✅ Correctly failed unauthorized liquidity deposit.");
    
    Ok(())
}
