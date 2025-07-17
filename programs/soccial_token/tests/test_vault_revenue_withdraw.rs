// ======================================================================
/// Soccial Token – Integration Tests: Vault Withdrawals (Revenue)
///
/// These tests validate the logic for securely withdrawing tokens 
/// from the revenue vault in the Soccial Token contract. 
///
/// Covered scenarios:
/// - ✅ Successful withdrawals to participant accounts
/// - ✅ Balance changes verified after withdrawal
/// - ❌ Unauthorized withdrawals are correctly rejected
///
/// Author: Paulo Rodrigues  
/// Project: Soccial Token  
/// Website: https://www.soccial.com/thetoken  
/// ======================================================================


use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;
use solana_sdk::msg;

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
async fn test_withdraw_revenue_vault_should_succeed() -> Result<(), TransportError> {

    let (mut context, admin) = setup_test_env().await;
    
    context.mint_tokens(&admin, 10_000).await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;

    // Fund source vault
    context.mint_tokens_to_vault("revenue", 100_000_000_000).await?;

    try_withdraw_revenue_vault(&mut context, &admin, &participant, 2_000).await?;

    msg!("✅ Rewards vault withdrawal successful.");

    log_all_balances(&mut context, &admin).await?;

    Ok(())
}

#[tokio::test]
async fn test_withdraw_revenue_balances_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    context.mint_tokens(&admin, 10_000).await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;

    // Fund source vault
    context.mint_tokens_to_vault("revenue", 100_000_000_000).await?;

    let withdraw_amount = 2_000;

    // Balances before the withdrawal
    let revenue_before = context.get_vault_balance("revenue").await;
    let participant_before = context.get_user_balance(&participant.pubkey()).await;

    // Execute the withdrawal
    try_withdraw_revenue_vault(&mut context, &admin, &participant, withdraw_amount).await?;

    context.refresh().await;

    // Balances after the withdrawal
    let revenue_after = context.get_vault_balance("revenue").await;
    let participant_after = context.get_user_balance(&participant.pubkey()).await;

    // Assert: vault should decrease by the withdrawal amount
    assert_eq!(
        revenue_before - revenue_after,
        withdraw_amount,
        "❌ Revenue vault should decrease by {} tokens, but decreased by {}",
        withdraw_amount,
        revenue_before - revenue_after
    );

    // Assert: participant should receive the exact amount
    assert_eq!(
        participant_after - participant_before,
        withdraw_amount,
        "❌ Participant should receive {} tokens, but received {}",
        withdraw_amount,
        participant_after - participant_before
    );

    msg!("✅ Revenue vault withdrawal successful and balances verified.");
    log_all_balances(&mut context, &admin).await?;

    Ok(())
}


#[tokio::test]
async fn test_withdraw_revenue_vault_invalid_authority_should_fail() -> Result<(), TransportError> {

    let (mut context, admin) = setup_test_env().await;
    let intruder = Keypair::new();

    context.mint_tokens(&admin, 500_000).await;

    // Fund source vault
    context.mint_tokens_to_vault("revenue", 100_000_000_000).await?;
    
    try_deposit_revenue_vault(&mut context, &admin, &admin, 500_000).await?;

    create_user_ata(&mut context, &intruder).await?;

    let result = try_withdraw_revenue_vault(&mut context, &intruder, &intruder, 1_000).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error when withdrawing from vault.");

    
    Ok(())
}
