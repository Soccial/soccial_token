// ============================================================================
// Soccial Token – Vault Utilities for Integration Testing
// ----------------------------------------------------------------------------
//
// This module provides high-level helpers for testing token vault logic in
// the Soccial Token smart contract. It includes generic deposit/withdraw/
// transfer functions and specific wrappers for each named vault.
//
// These utilities are designed for integration testing with `ProgramTest`,
// enabling simulation of realistic vault operations.
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Deposit/withdraw from any vault with optional reason (audit/log)  
// ✔ Transfer tokens between vaults using seed-resolved addresses  
// ✔ Specialized wrappers for each vault type (airdrop, rewards, revenue, etc.)  
// ✔ Internal contract-to-vault transfers (using PDA authority)  
// ✔ Test functions to assert success or expected failure of vault interactions  
//
// ----------------------------------------------------------------------------
// Key Vault Types Covered:
// - Offchain Reserve  
// - Liquidity  
// - Staking  
// - Revenue  
// - Rewards  
// - Airdrop  
// - Vesting  
// - Insurance  
// - Treasury  
// - Reserved Supply  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================


use soccial_token::vaults::VaultError;
use solana_sdk::{msg, pubkey::Pubkey, signature::Keypair, signer::Signer, transport::TransportError};
use crate::testutils::{basics::*, environment::log_all_balances};
use crate::testutils::environment::EnvProgramTestContext;
use crate::trymethods::trygovernance::try_approve_proposal_flow;
use soccial_token::{self, instruction as soccial_instruction};

// ============================================================================
/// Attempts to deposit tokens from a participant into a specified vault.
///
/// This function sends a `VaultDeposit` instruction with the provided vault parameters.
/// It supports custom reasons for audit or logging purposes.
///
/// # Parameters:
/// - `context`: Mutable reference to the `ProgramTest` context.
/// - `caller`: Authorized signer triggering the deposit (usually admin or backend).
/// - `participant`: The user whose tokens are being deposited.
/// - `vault`: The destination vault PDA.
/// - `vault_token_account`: Token account owned by the vault.
/// - `vault_authority`: PDA authority of the vault (signer).
/// - `amount`: Number of tokens to deposit.
/// - `reason`: Optional reason string for logging or categorization.
///
/// # Returns:
/// `Ok(())` on success or `TransportError` on failure.
///
/// # Example:
/// ```
/// try_vault_deposit(&mut context, &admin, &user, treasury_vault, treasury_vault_ata, treasury_vault_auth, 1_000_000, Some("internal allocation")).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_vault_deposit(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Keypair,   // The user depositing their tokens
    vault: Pubkey,
    vault_token_account: Pubkey,
    vault_authority: Pubkey,
    amount: u64,
    reason: Option<&str>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());

    let mut args = vec![amount.to_string()];
    if let Some(r) = reason {
        args.push(r.to_string());
    }

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::VaultDepositContext {
            caller: caller.pubkey(),
            vault_token_account,
            vault,
            vault_authority,
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
            participant: participant.pubkey(),
            participant_token_account: seeds.user_token_ata,
            token_mint: seeds.token_mint,
        },
        soccial_instruction::VaultDeposit { args },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller, participant],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Attempts to withdraw tokens from a specified vault to a user account.
///
/// This function sends a `VaultWithdraw` instruction from the given vault
/// and transfers the specified amount to the recipient's token account.
/// An optional reason may be logged for audit.
///
/// # Parameters:
/// - `context`: Mutable reference to the `ProgramTest` context.
/// - `caller`: Authorized signer performing the withdrawal.
/// - `destination`: User receiving the tokens.
/// - `vault`: Vault PDA.
/// - `vault_token_account`: Token account owned by the vault.
/// - `vault_authority`: Vault PDA authority.
/// - `amount`: Amount to withdraw.
/// - `reason`: Optional reason string for logging or policy control.
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` if the transaction fails.
///
/// # Example:
/// ```
/// try_vault_withdraw(&mut context, &admin, &user, airdrop_vault, airdrop_vault_ata, airdrop_vault_auth, 5_000, Some("weekly_reward")).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_vault_withdraw(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    destination: &Keypair,   // The user that will receive the tokens from the vault
    vault: Pubkey,
    vault_token_account: Pubkey,
    vault_authority: Pubkey,
    amount: u64,
    reason: Option<&str>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &destination.pubkey());

    let mut args = vec![amount.to_string()];
    if let Some(r) = reason {
        args.push(r.to_string());
    }

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::VaultWithdrawContext {
            caller: caller.pubkey(),
            vault_token_account,
            user_token_account: seeds.user_token_ata,
            vault,
            vault_authority,
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
        },
        soccial_instruction::VaultWithdraw { args },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
/// Transfers tokens between two vaults (source → destination).
///
/// This helper abstracts the `TransferBetweenVaults` instruction,
/// allowing the caller to move tokens across any two vaults,
/// as long as they have the proper permissions.
///
/// # Parameters:
/// - `context`: Mutable reference to the test environment.
/// - `caller`: Signer executing the transfer.
/// - `source_vault`: Source vault PDA.
/// - `source_vault_token_account`: Token account belonging to the source vault.
/// - `source_vault_authority`: PDA authority for the source vault.
/// - `destination_vault_token_account`: Token account of the receiving vault.
/// - `amount`: Tokens to transfer.
/// - `reason`: Optional string used for categorization or audit logging.
///
/// # Returns:
/// `Ok(())` if transfer succeeds, or `TransportError` if it fails.
///
/// # Example:
/// ```
/// try_vault_transfer(&mut context, &admin, vesting_vault, vesting_vault_ata, vesting_auth, treasury_vault_ata, 50_000, Some("buyback redistribution")).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_vault_transfer(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: Pubkey,
    source_vault_token_account: Pubkey,
    source_vault_authority: Pubkey,
    destination_vault_token_account: Pubkey,
    amount: u64,
    reason: Option<&str>,
    proposal_id: Option<u64>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let mut args = vec![amount.to_string()];
    if let Some(r) = reason {
        args.push(r.to_string());
    }

    // Derive proposal if proposal_id is provided
    let proposal = proposal_id.map(|id| {
        derive_proposal_account(&context.program_id, id).0
    });
    
    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::VaultTransferContext {
            source_vault,
            source_vault_token_account,
            destination_vault_token_account,
            source_vault_authority,
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            governance_state: seeds.governance_state,
            proposal,
            token_program: spl_token::ID,
        },
        soccial_token::instruction::TransferBetweenVaults { args },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}


// ============================================================================
// Vault Wrappers: Deposit & Withdraw Helpers per Named Vault
// ----------------------------------------------------------------------------
//
// These functions provide high-level helpers for interacting with each named
// vault in the Soccial Token contract. They are thin wrappers around the
// generic `try_vault_deposit` and `try_vault_withdraw` functions,
// with vault-specific seeds automatically resolved per vault type.
//
// Each function handles a specific vault (e.g. `liquidity_vault`,
// `revenue_vault`, `staking_vault`, etc.) and simplifies test readability
// when writing focused vault interaction tests.
//
// ----------------------------------------------------------------------------
// Vaults Supported:
// - Offchain Reserve Vault
// - Liquidity Vault
// - Staking Vault
// - Revenue Vault
// - Rewards Vault
// - Airdrop Vault
// - Vesting Vault
// - Insurance Vault
// - Treasury Vault
// - Reserved Supply Vault
//
// ----------------------------------------------------------------------------
// Usage Example:
//
// try_deposit_rewards_vault(&mut context, &admin, &user, 10_000).await?;
// try_withdraw_airdrop_vault(&mut context, &admin, &user, 5_000).await?;
//
// ============================================================================

// ▸ OffchainReserve
#[allow(dead_code)]
pub async fn try_deposit_offchain_reserve_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.offchain_reserve_vault, seeds.offchain_reserve_vault_token_account, seeds.offchain_reserve_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_offchain_reserve_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.offchain_reserve_vault, seeds.offchain_reserve_vault_token_account, seeds.offchain_reserve_vault_authority, amount, Some("teste")).await
}

// ▸ Liquidity
#[allow(dead_code)]
pub async fn try_deposit_liquidity_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.liquidity_vault, seeds.liquidity_vault_token_account, seeds.liquidity_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_liquidity_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.liquidity_vault, seeds.liquidity_vault_token_account, seeds.liquidity_vault_authority, amount, Some("teste")).await
}

// ▸ Staking
#[allow(dead_code)]
pub async fn try_deposit_staking_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.staking_vault, seeds.staking_vault_token_account, seeds.staking_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_staking_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.staking_vault, seeds.staking_vault_token_account, seeds.staking_vault_authority, amount, Some("teste")).await
}

// ▸ Revenue
#[allow(dead_code)]
pub async fn try_deposit_revenue_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.revenue_vault, seeds.revenue_vault_token_account, seeds.revenue_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_revenue_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.revenue_vault, seeds.revenue_vault_token_account, seeds.revenue_vault_authority, amount, Some("teste")).await
}

// ▸ Rewards
#[allow(dead_code)]
pub async fn try_deposit_rewards_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.rewards_vault, seeds.rewards_vault_token_account, seeds.rewards_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_rewards_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.rewards_vault, seeds.rewards_vault_token_account, seeds.rewards_vault_authority, amount, Some("teste")).await
}

// ▸ Airdrop
#[allow(dead_code)]
pub async fn try_deposit_airdrop_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.airdrop_vault, seeds.airdrop_vault_token_account, seeds.airdrop_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_airdrop_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.airdrop_vault, seeds.airdrop_vault_token_account, seeds.airdrop_vault_authority, amount, Some("teste")).await
}

// ▸ Vesting
#[allow(dead_code)]
pub async fn try_deposit_vesting_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.vesting_vault, seeds.vesting_vault_token_account, seeds.vesting_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_vesting_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.vesting_vault, seeds.vesting_vault_token_account, seeds.vesting_vault_authority, amount, Some("teste")).await
}

// ▸ Insurance
#[allow(dead_code)]
pub async fn try_deposit_insurance_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.insurance_vault, seeds.insurance_vault_token_account, seeds.insurance_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_insurance_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.insurance_vault, seeds.insurance_vault_token_account, seeds.insurance_vault_authority, amount, Some("teste")).await
}

// ▸ Treasury
#[allow(dead_code)]
pub async fn try_deposit_treasury_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.treasury_vault, seeds.treasury_vault_token_account, seeds.treasury_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_treasury_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context, caller, destination, seeds.treasury_vault, seeds.treasury_vault_token_account, seeds.treasury_vault_authority, amount, Some("teste")).await
}

// ▸ Reserved Supply
#[allow(dead_code)]
pub async fn try_deposit_reserved_supply_vault(context: &mut EnvProgramTestContext, caller: &Keypair, participant: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_deposit(context, caller, participant, seeds.reserved_supply_vault, seeds.reserved_supply_vault_token_account, seeds.reserved_supply_vault_authority, amount, Some("teste")).await
}

#[allow(dead_code)]
pub async fn try_withdraw_reserved_supply_vault(context: &mut EnvProgramTestContext, caller: &Keypair, destination: &Keypair, amount: u64) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());
    try_vault_withdraw(context,  caller, destination, seeds.reserved_supply_vault, seeds.reserved_supply_vault_token_account, seeds.reserved_supply_vault_authority, amount, Some("teste")).await
}
// ============================================================================
/// Transfers tokens from the contract's internal fallback token account
/// (PDA-owned) into a named vault by seed.
///
/// This simulates an internal system operation (e.g., moving leftover funds,
/// fees, or allocations into a managed vault) executed by the contract itself.
///
/// # Parameters:
/// - `context`: Mutable reference to the `EnvProgramTestContext`.
/// - `caller`: Signer account initiating the operation (usually admin or system PDA).
/// - `vault_seed`: The string name of the target vault (e.g. "treasury_vault").
/// - `amount`: Amount of tokens to transfer from the contract fallback account.
///
/// # Returns:
/// `Ok(())` if the transfer succeeds, or `TransportError` if the transaction fails.
///
/// # Behavior:
/// Uses the `MoveFromContractToVault` instruction with PDA signer seeds.
///
/// # Example:
/// ```
/// try_transfer_contract_to_vault(&mut context, &admin, "revenue_vault", 1_000_000).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_transfer_contract_to_vault(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    vault_seed: &str,
    amount: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &context.caller.pubkey());

    let contract_token_account = seeds.contract_token_account;

    let vault_seed_bytes = vault_seed.as_bytes();
    let (destination_vault, _) = Pubkey::find_program_address(&[vault_seed_bytes], &context.program_id);
    let destination_vault_token_account = get_vault_token_account(&seeds, vault_seed);

    let (contract_token_owner, bump) =
        Pubkey::find_program_address(&[b"contract_token_owner"], &context.program_id);

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::ContractToVaultContext {
            source_authority: contract_token_owner,
            source_token_account: contract_token_account,
            destination_vault,
            destination_vault_token_account,
            caller: context.caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
        },
        soccial_token::instruction::MoveFromContractToVault {
            args: vec![amount.to_string()],
        },
    );

    send_ix_with_signer_seeds(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &context.caller],
        ix,
        context.recent_blockhash,
        &[&[b"contract_token_owner", &[bump]]],
    )
    .await?;


    context.refresh().await;

    log_all_balances(context, caller)
        .await
        .expect("Failed to log balances");

    Ok(())
}



// ============================================================================
/// Transfers tokens from one vault to another using their vault seed strings.
///
/// This high-level helper simplifies internal reallocations between system
/// vaults (e.g. transferring fees from `reserved_supply_vault` to `revenue_vault`).
/// Vault names are passed as string identifiers resolved to their PDAs.
///
/// # Parameters:
/// - `context`: Mutable reference to the test context.
/// - `caller`: Signer initiating the transfer.
/// - `from_vault_seed`: Name of the source vault (e.g., "revenue_vault").
/// - `to_vault_seed`: Name of the destination vault (e.g., "airdrop_vault").
/// - `amount`: Amount to transfer.
///
/// # Returns:
/// `Ok(())` on success or `TransportError` on failure.
///
/// # Example:
/// ```
/// try_transfer_between_vaults(&mut context, &admin, "reserved_supply_vault", "revenue_vault", 500_000).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_transfer_between_vaults(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    from_vault_seed: &str,
    to_vault_seed: &str,
    amount: u64,
    proposal_id: Option<u64>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    // Resolve source vault addresses
    let source_vault_seed = from_vault_seed.as_bytes();
    let (source_vault, _) = Pubkey::find_program_address(&[source_vault_seed], &context.program_id);
    let (source_vault_authority, _) = Pubkey::find_program_address(&[source_vault_seed], &context.program_id);
    let source_vault_token_account = get_vault_token_account(&seeds, from_vault_seed);

    // Resolve destination vault token account
    let destination_vault_token_account = get_vault_token_account(&seeds, to_vault_seed);

    // Build and send instruction
    try_vault_transfer(
        context,
        caller,
        source_vault,
        source_vault_token_account,
        source_vault_authority,
        destination_vault_token_account,
        amount,
        Some("We need to transfer this!"),
        proposal_id
    ).await
}


// ============================================================================
/// Test utility that executes a successful vault-to-vault transfer and validates balances.
///
/// This function is meant to be used in integration tests to assert:
/// - The source vault balance decreased by `amount`
/// - The destination vault balance increased by `amount`
///
/// # Parameters:
/// - `context`: Program test environment.
/// - `caller`: Admin or signer executing the transfer.
/// - `source_vault`: Source vault name (string).
/// - `destination_vault`: Destination vault name (string).
/// - `amount`: Token amount to move between vaults.
///
/// # Panics:
/// Fails test if the transfer does not complete successfully or balances are incorrect.
///
/// # Example:
/// ```
/// try_transfer_between_vaults_test(&mut context, &admin, "airdrop_vault", "treasury_vault", 100_000).await;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_transfer_between_vaults_test(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: &str,
    destination_vault: &str,
    amount: u64,
    proposal_id: Option<u64>,
) {
    let source_key = normalize_vault_name(source_vault);
    let dest_key = normalize_vault_name(destination_vault);

    // Fund source vault
    let _=context.mint_tokens_to_vault(source_key, 100_000_000_000).await;

    // Balances before transfer
    let source_before = context.get_vault_balance(source_key).await;
    let dest_before = context.get_vault_balance(dest_key).await;

    // Execute the transfer
    let result = try_transfer_between_vaults(
        context,
        caller,
        source_vault,
        destination_vault,
        amount,
        proposal_id
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "❌ Transfer from {} to {} failed unexpectedly: {:?}",
        source_vault,
        destination_vault,
        result.err()
    );

    context.refresh().await;

    // Balances after transfer
    let source_after = context.get_vault_balance(source_key).await;
    let dest_after = context.get_vault_balance(dest_key).await;

    // Assert source vault decreased
    assert_eq!(
        source_before - source_after,
        amount,
        "❌ Source vault '{}' should decrease by {} tokens, but decreased by {}",
        source_vault,
        amount,
        source_before - source_after
    );

    // Assert destination vault increased
    assert_eq!(
        dest_after - dest_before,
        amount,
        "❌ Destination vault '{}' should increase by {} tokens, but increased by {}",
        destination_vault,
        amount,
        dest_after - dest_before
    );

    msg!(
        "✅ Successfully transferred {} tokens from '{}' to '{}' vault.",
        amount,
        source_vault,
        destination_vault
    );


}


// ============================================================================
/// Test utility that performs a vault-to-vault transfer and validates balances,
/// with governance approval enforced.
///
/// This test ensures that:
/// - A proposal of the specified type is approved through the governance system
/// - The transfer respects governance restrictions
/// - Token balances are updated correctly for both the source and destination vaults
///
/// Intended for integration testing of vault movement logic tied to governance.
///
/// # Parameters:
/// - `context`: Program test environment
/// - `caller`: Admin or signer executing the transfer
/// - `source_vault`: Name of the source vault
/// - `destination_vault`: Name of the destination vault
/// - `amount`: Token amount to transfer
/// - `proposal_type`: Type of governance proposal required to authorize the transfer
///
/// # Panics:
/// Fails the test if governance approval fails or balances are incorrect
///
/// # Example:
/// ```
/// try_transfer_between_vaults_test_with_governance(
///     &mut context,
///     &admin,
///     "airdrop_vault",
///     "treasury_vault",
///     100_000,
///     "UpdateAirdropFee",
/// ).await;
/// ```
// ============================================================================

#[allow(dead_code)]
pub async fn try_transfer_between_vaults_test_with_governance(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: &str,
    destination_vault: &str,
    amount: u64,
    proposal_type: &str,
) {
    // Create a governance proposal that successfully passes
    let proposal_types = vec![proposal_type.to_string()];
    let proposal_id = try_approve_proposal_flow(
        context,
        caller,
        format!("Testing vault transfer with governance: {}", proposal_type),
        proposal_types,
    )
    .await
    .expect("Governance approval should succeed");

    // Execute the actual vault-to-vault transfer test with governance approval
    try_transfer_between_vaults_test(
        context,
        caller,
        source_vault,
        destination_vault,
        amount,
        Some(proposal_id),
    )
    .await;
}

// ============================================================================
/// Executes a vault-to-vault transfer test **without pre-funding** the source.
///
/// This version is used for edge-case tests, where vaults might start with zero
/// balance or require testing failure behavior based on lack of liquidity.
///
/// # Parameters:
/// - `context`: Program test environment.
/// - `caller`: Admin or signer executing the transfer.
/// - `source_vault`: Vault name to debit from.
/// - `destination_vault`: Vault name to credit to.
/// - `amount`: Token amount to attempt transferring.
///
/// # Panics:
/// Panics if the transfer fails unexpectedly.
///
/// # Example:
/// ```
/// try_transfer_between_vaults_without_funding_test(&mut context, &admin, "treasury_vault", "insurance_vault", 50_000).await;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_transfer_between_vaults_without_funding_test(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: &str,
    destination_vault: &str,
    amount: u64,
    proposal_id: Option<u64>,
) {

    // Execute the transfer
    let result = try_transfer_between_vaults(
        context,
        caller,
        source_vault,
        destination_vault,
        amount,
        proposal_id
    )
    .await;

    // Assert success
    assert!(
        result.is_ok(),
        "❌ Transfer from {} to {} failed unexpectedly: {:?}",
        source_vault,
        destination_vault,
        result.err()
    );

    context.refresh().await;

    msg!(
        "✅ Successfully transferred {} tokens from '{}' to '{}' vault.",
        amount,
        source_vault,
        destination_vault
    );


}

/// Converts "airdrop_vault" → "airdrop"
fn normalize_vault_name(name: &str) -> &str {
    name.strip_suffix("_vault").unwrap_or(name)
}

// ============================================================================
/// Negative test utility to assert a **vault-to-vault transfer fails** as expected.
///
/// This is used to verify that certain transfers are blocked due to logic errors,
/// policy restrictions, or permission issues. It checks that the error code matches
/// the expected one and logs success on correct failure.
///
/// # Parameters:
/// - `context`: Mutable program test context.
/// - `caller`: Signer initiating the (expected to fail) transfer.
/// - `source_vault`: Vault seed string to debit from.
/// - `destination_vault`: Vault seed string to credit to.
/// - `amount`: Amount to transfer.
/// - `expected_error`: Expected error code (as u32 or enum).
/// - `description`: Reason or expectation context for logging clarity.
///
/// # Example:
/// ```
/// try_transfer_between_vaults_should_fail(
///     &mut context,
///     &admin,
///     "revenue_vault",
///     "revenue_vault",
///     10_000,
///     VaultError::InvalidVaultTransfer as u32,
///     "Should fail because source and destination are the same"
/// ).await;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_transfer_between_vaults_should_fail<E>(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: &str,
    destination_vault: &str,
    amount: u64,
    expected_error: E,
    description: &str,
    proposal_id: Option<u64>,
)
where
    E: Into<u32> + core::fmt::Debug,
{
    let result = try_transfer_between_vaults(
        context,
        caller,
        source_vault,
        destination_vault,
        amount,
        proposal_id
    )
    .await;

    assert_custom_error(result, expected_error, description);

    msg!(
        "✅ Correctly failed transfer from {} to {} vault → {}",
        source_vault,
        destination_vault,
        description
    );

    context.refresh().await;

    log_all_balances(context, caller)
        .await
        .expect("Failed to log balances");
}


/// ===========================================================================
/// Test: try_transfer_between_vaults_test_fail
/// 
/// Validates that a transfer between vaults fails under invalid conditions.
/// 
/// ## Purpose:
/// - Ensures that transfers from a vault with a blocked or unknown type
///   correctly fail with a `VaultError::UnknownVaultType`.
///
/// ## Behavior:
/// - Mints tokens to the source vault.
/// - Attempts to transfer tokens to another vault.
/// - Expects failure with specific custom error code.
///
/// ## Parameters:
/// - `context`: Mutable test context for program execution
/// - `caller`: Signer attempting the transfer
/// - `source_vault`: Name of the source vault
/// - `destination_vault`: Name of the target vault
/// - `amount`: Amount of tokens to transfer
///
/// ## Expected Outcome:
/// - The transfer should **fail** due to restricted withdrawal on the vault.
///
/// ## Error Checked:
/// - `VaultError::UnknownVaultType`
///
/// Author: Paulo Rodrigues  
/// ===========================================================================
#[allow(dead_code)]
pub async fn try_transfer_between_vaults_test_vault_fail(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    source_vault: &str,
    destination_vault: &str,
    amount: u64,
    proposal_id: Option<u64>,
) {
    let source_key = normalize_vault_name(source_vault);

    // Fund source vault
    let _=context.mint_tokens_to_vault(source_key, 100_000_000_000).await;
    

    // Execute the transfer
    let result = try_transfer_between_vaults(
        context,
        caller,
        source_vault,
        destination_vault,
        amount,
        proposal_id
    )
    .await;

    // The vault is blocked for withdrawn, it should fail
    assert_custom_error(
        result, 
        VaultError::UnauthorizedVaultTransfer, 
        "Expected Unauthorized vault access. It should return a Unknown vault error because the vault is blocked for transfers."
    );

}


