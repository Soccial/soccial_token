// ======================================================================
// Test: test_vault_transfer_between_vaults.rs
// ======================================================================

use soccial_token::vaults::VaultError;
use solana_program_test::*;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;

use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryvaults::*;

#[tokio::test]
async fn test_transfer_vesting_to_revenue() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "revenue_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_rewards() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "rewards_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_treasury() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "treasury_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_insurance() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "insurance_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_vault_to_reserved_supply() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "vesting_vault", "reserved_supply_vault", 1_000_000_000, "RecoverFromVesting").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_airdrop() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "airdrop_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_staking() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "staking_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_liquidity() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "liquidity_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_vesting_to_offchain_reserve() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "vesting_vault", "offchain_reserve_vault", 1_000_000_000, None).await;
    Ok(())
}


#[tokio::test]
async fn test_transfer_vesting_to_vesting() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    
    try_transfer_between_vaults_should_fail(&mut context, &caller, 
        "vesting_vault", 
        "vesting_vault", 
        70_000_000,
        VaultError::InvalidItselfVaultTransfer, 
        "Expected failure when transferring to the same vault",
        None
    ).await;

    Ok(())
}