use soccial_token::utils::error::ErrorCode;
/// ======================================================================
/// Soccial Token – Integration Tests: Vault-to-Vault Transfers
///
/// These tests validate internal token transfers between vaults 
/// in the Soccial Token contract. They ensure that only allowed 
/// transfers are permitted, unauthorized access is blocked, and 
/// vault balances are correctly updated.
///
/// Covered scenarios:
/// - ✅ Valid transfers from airdrop vault to all supported destination vaults
/// - ❌ Rejection of transfers to the same vault
/// - ❌ Rejection of transfers from unauthorized callers
///
/// Author: Paulo Rodrigues  
/// Project: Soccial Token  
/// Website: https://www.soccial.com/thetoken  
/// ======================================================================

use soccial_token::vaults::VaultError;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;

use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryvaults::*;

// ======================================================================
// TESTS
// ======================================================================

#[tokio::test]
async fn test_transfer_contract_to_rewards_vault() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;

    context.mint_tokens_to_contract_account(100_000_000_000_000).await?;

    // Transfer 1 SCTK (10_000_000_000 = 10 tokens with 9 decimals)
    try_transfer_contract_to_vault(&mut context, &caller, "rewards_vault", 50_000_000_000_000) .await?;
    
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_revenue() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "revenue_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_reserved_supply_vault() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "reserved_supply_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_treasury() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "treasury_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_insurance() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "insurance_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_vesting() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "vesting_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_airdrop() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "airdrop_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_staking() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "staking_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_liquidity() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "liquidity_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_offchain_reserve() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "rewards_vault", "offchain_reserve_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_rewards() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    
    try_transfer_between_vaults_should_fail(&mut context, &caller, 
        "rewards_vault", 
        "rewards_vault", 
        70_000_000,
        VaultError::InvalidItselfVaultTransfer, 
        "Expected failure when transferring to the same vault",
        None
    ).await;

    Ok(())
}

#[tokio::test]
async fn test_transfer_rewards_to_revenue_without_permissions_should_fail() -> Result<(), TransportError> {
    let (mut context, _admin) = setup_test_env().await;

    let intruder = Keypair::new();
    
    try_transfer_between_vaults_should_fail(&mut context, &intruder, 
        "rewards_vault", 
        "revenue_vault", 
        70_000_000,
        ErrorCode::Unauthorized, 
        "Expected Unauthorized error transferring from unauthorized user",
        None
    ).await;

    Ok(())
}