// ======================================================================
/// Soccial Token – Integration Tests: Vault Withdrawals (Airdrop)
///
/// These tests validate the logic for securely withdrawing tokens 
/// from the airdrop vault in the Soccial Token contract. 
///
///
/// Author: Paulo Rodrigues  
/// Project: Soccial Token  
/// Website: https://www.soccial.com/thetoken  
/// ======================================================================


use soccial_token::utils::error::ErrorCode;
use soccial_token::vaults::VaultError;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
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
async fn test_withdraw_airdrop_vault_should_fail() -> Result<(), TransportError> {

    let (mut context, admin) = setup_test_env().await;
    
    context.mint_tokens(&admin, 10_000).await;

    let participant = Keypair::new();
    create_user_ata(&mut context, &participant).await?;
    fund_lamports(&mut context, &participant, 5_000_000).await?;
    context.mint_tokens(&participant, 10_000).await;

    // Fund source vault
    context.mint_tokens_to_vault("airdrop", 100_000_000_000).await?;

    let result = try_withdraw_airdrop_vault(&mut context, &admin, &participant, 2_000).await;

    // The vault is blocked for withdrawn, it should fail
    assert_custom_error(result, VaultError::UnknownVaultType, "Expected Unauthorized vault access. It should return a Unknown vault error because the vault is blocked for withdraw.");

    Ok(())
}

#[tokio::test]
async fn test_withdraw_airdrop_vault_invalid_authority_should_fail() -> Result<(), TransportError> {

    let (mut context, admin) = setup_test_env().await;
    let intruder = Keypair::new();

    context.mint_tokens(&admin, 500_000).await;

    // Fund source vault
    context.mint_tokens_to_vault("airdrop", 100_000_000_000).await?;
    
    try_deposit_airdrop_vault(&mut context, &admin, &admin, 500_000).await?;

    create_user_ata(&mut context, &intruder).await?;

    let result = try_withdraw_airdrop_vault(&mut context, &intruder, &intruder, 1_000).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error when withdrawing from vault.");

    
    Ok(())
}