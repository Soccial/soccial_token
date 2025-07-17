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


use soccial_token::utils::error::ErrorCode;
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
async fn test_transfer_airdrop_to_revenue_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "revenue_vault", 1_000_000_000, None).await;
    
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_rewards_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "rewards_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_treasury_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "treasury_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_insurance_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "insurance_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_vesting_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "vesting_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_reserved_supply_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;

    try_transfer_between_vaults_test_with_governance(&mut context, &admin, "airdrop_vault", "reserved_supply_vault", 1_000_000_000, "RecoverFromAidrop").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_staking_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "staking_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_liquidity_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "liquidity_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_offchain_reserve_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &admin, "airdrop_vault", "offchain_reserve_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_airdrop_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    
    try_transfer_between_vaults_should_fail(&mut context, &admin, 
        "airdrop_vault", 
        "airdrop_vault", 
        70_000_000,
        VaultError::InvalidItselfVaultTransfer, 
        "Expected failure when transferring to the same vault",
        None
    ).await;

    Ok(())
}

#[tokio::test]
async fn test_transfer_airdrop_to_revenue_without_permissions_should_fail() -> Result<(), TransportError> {
    let (mut context, _admin) = setup_test_env().await;

    let intruder = Keypair::new();
    
    try_transfer_between_vaults_should_fail(&mut context, &intruder, 
        "airdrop_vault", 
        "revenue_vault", 
        70_000_000,
        ErrorCode::Unauthorized, 
        "Expected Unauthorized error transferring from unauthorized user",
        None
    ).await;

    Ok(())
}