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
async fn test_transfer_contract_to_reserved_supply() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;

    context.mint_tokens_to_contract_account(100_000_000_000_000).await?;

    // Transfer 1 SCTK (10_000_000_000 = 10 tokens with 9 decimals)
    try_transfer_contract_to_vault(&mut context, &caller, "reserved_supply_vault", 50_000_000_000_000) .await?;
    
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_revenue() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "reserved_supply_vault", "revenue_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_rewards() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "reserved_supply_vault", "rewards_vault", 1_000_000_000, "RewardsAllocation").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_treasury() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "reserved_supply_vault", "treasury_vault", 1_000_000_000, "TreasuryAllocation").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_insurance() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test(&mut context, &caller, "reserved_supply_vault", "insurance_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_vesting() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "reserved_supply_vault", "vesting_vault", 1_000_000_000, "VestingRecover").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_airdrop() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "reserved_supply_vault", "airdrop_vault", 1_000_000_000, "AidropAllocation").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_staking() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_with_governance(&mut context, &caller, "reserved_supply_vault", "staking_vault", 1_000_000_000, "StakingRecover").await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_liquidity() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test(&mut context, &caller, "reserved_supply_vault", "liquidity_vault", 1_000_000_000, None).await;
    Ok(())
}

#[tokio::test]
async fn test_transfer_reserved_supply_to_offchain_reserve() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    try_transfer_between_vaults_test_vault_fail(&mut context, &caller, "reserved_supply_vault", "offchain_reserve_vault", 1_000_000_000, None).await;
    Ok(())
}


#[tokio::test]
async fn test_transfer_reserved_supply_to_reserved_supply() -> Result<(), TransportError> {
    let (mut context, caller) = setup_test_env().await;
    
    try_transfer_between_vaults_should_fail(&mut context, &caller, 
        "reserved_supply_vault", 
        "reserved_supply_vault", 
        70_000_000,
        VaultError::InvalidItselfVaultTransfer, 
        "Expected failure when transferring to the same vault",
        None
    ).await;

    Ok(())
}