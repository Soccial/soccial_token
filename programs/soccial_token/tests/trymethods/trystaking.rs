// ============================================================================
// Soccial Token – Staking Instruction Wrappers for Integration Tests
// ----------------------------------------------------------------------------
//
// This module provides test utilities for interacting with the staking system
// of the Soccial Token contract. It supports scenarios like:
// - Buying and staking in one transaction
// - Manual staking from wallet balances
// - Claiming staking rewards and withdrawing tokens
// - Managing staking plans (APR, lockup duration, enabling/disabling)
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Simulate user staking flows end-to-end  
// ✔ Configure and update staking plans in test suites  
// ✔ Verify PDA derivation and claim logic  
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_stake_tokens`, `try_buy_and_stake_tokens`  
// - `try_claim_staking_rewards`, `try_withdraw_staked_tokens`  
// - `try_add_staking_plan`, `try_edit_staking_plan`, `try_disable_staking_plan`  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use anchor_lang::{solana_program::{self, example_mocks::solana_sdk::system_program}, AccountDeserialize};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transport::TransportError};
use crate::testutils::environment::EnvProgramTestContext;
use crate::testutils::basics::*;
use soccial_token::{accounts as soccial_accounts, instruction as soccial_instruction, staking::StakingState, token::TokenState};
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program::sysvar::clock;

/// Derives the staking account PDA for a given participant and staking ID.
pub fn derive_staking_account_pda(
    program_id: &Pubkey,
    participant: &Pubkey,
    staking_id: u64,
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"staking_account",
            participant.as_ref(),
            &staking_id.to_le_bytes(),
        ],
        program_id,
    ).0
}

// ============================================================================
/// Buys tokens and immediately stakes them in a single instruction.
///
/// This combines purchase and stake logic into a single TX for UX efficiency.
/// Staking ID is auto-incremented from the staking state.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer (admin/contract logic)
/// - `participant`: User who will receive the stake
/// - `amount`: Amount of tokens to buy and stake
/// - `plan_id`: ID of staking plan to use
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_buy_and_stake_tokens(&mut context, &admin, &user.pubkey(), 1_000_000, 1).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_buy_and_stake_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    amount: u64,
    plan_id: u8,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, participant);

    // 1. Fetch staking_state to get current staking_id
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState must exist");

    let staking_state = StakingState::try_deserialize(&mut &staking_state_account.data[..])
        .expect("Failed to deserialize staking state");

    let staking_id = staking_state.last_id;

    // 2. Derive staking_account PDA using participant and staking_id
    let staking_account_pda =
        derive_staking_account_pda(&context.program_id, participant, staking_id);

    // 3. Prepare args
    let args = vec![amount.to_string(), plan_id.to_string()];

    // 4. Build accounts metadata for instruction
    let accounts = soccial_accounts::BuyAndStakeTokens {
        staking_state: seeds.staking_state,
        staking_account: staking_account_pda,
        participant: *participant,
        token_mint: seeds.token_mint,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        staking_vault_token_account: seeds.staking_vault_token_account,
        staking_vault: seeds.staking_vault,
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        token_program: spl_token::ID,
        system_program: solana_program::system_program::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None);

    // 5. Build instruction
    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::BuyAndStakeTokens { args }.data(),
    };

    // 6. Send transaction
    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller],
        ix,
        context.recent_blockhash,
    )
    .await?;

    Ok(())
}


// ============================================================================
/// Stakes tokens from a participant’s wallet using a specific staking plan.
///
/// This requires the participant to sign and transfer their tokens to the vault.
/// Staking ID is auto-assigned from the current staking state.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Contract authority triggering the stake
/// - `participant`: The user whose tokens are being staked
/// - `amount`: Number of tokens to stake
/// - `plan_id`: Chosen staking plan
///
/// # Returns:
/// `Ok(())` if successful
///
/// # Example:
/// ```
/// try_stake_tokens(&mut context, &admin, &user, 500_000, 2).await?;
/// ```
// ============================================================================

#[allow(dead_code)]
pub async fn try_stake_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,        // Admin or function initiator
    participant: &Keypair,   // The user staking their tokens
    amount: u64,
    plan_id: u8,
) -> Result<(), TransportError> {
    // Step 1: Derive seeds for all relevant accounts using the participant
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());

    // Step 2: Fetch the current staking state to retrieve last_id for staking_account PDA
    let staking_state_account = context
        .banks_client
        .get_account(seeds.staking_state)
        .await?
        .expect("StakingState account must exist");

    let staking_state = StakingState::try_deserialize(
        &mut &staking_state_account.data[..],
    ).expect("Failed to deserialize StakingState");

    let staking_id = staking_state.last_id;

    // Step 3: Derive the PDA for the staking account using participant + staking_id
    let staking_account_pda = derive_staking_account_pda(
        &context.program_id,
        &participant.pubkey(),
        staking_id,
    );

    // Step 4: Prepare argument vector for the instruction
    let args = vec![amount.to_string(), plan_id.to_string()];

    // Step 5: Build accounts metadata for the CPI
    let accounts = soccial_accounts::StakeTokens {
        caller: caller.pubkey(),
        participant: participant.pubkey(),
        participant_token_account: seeds.user_token_ata,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        staking_vault_token_account: seeds.staking_vault_token_account,
        staking_vault: seeds.staking_vault,
        staking_account: staking_account_pda,
        staking_state: seeds.staking_state,
        mint_authority: seeds.mint_authority,
        token_mint: seeds.token_mint,
        destination_token_account: seeds.user_token_ata,
        user_access: None,
        token_state: seeds.token_state,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None);

    // Step 6: Build the instruction with the accounts and args
    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::StakeTokens { args }.data(),
    };

    // Step 7: Send the transaction with all required signers
    // Note: Participant MUST sign because tokens are being transferred from their ATA
    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller, participant], // Must have the participant to sign the transaction
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}

/// ============================================================================
/// Adds Additional Tokens to an Existing Stake (Reinforcement)
///
/// Simulates a participant increasing the size of an existing stake by
/// depositing more tokens. This updates the total staked amount,
/// reserves new rewards proportionally, and resets the staking cycle.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: The participant reinforcing the stake
/// - `stake_id`: Unique ID of the staking entry to reinforce
/// - `amount`: Number of additional tokens to stake
///
/// # Returns:
/// `Ok(())` if reinforcement was successful
///
/// # Example:
/// ```
/// try_add_to_stake(&mut context, &user, 1, 5_000).await?;
/// ```
/// ============================================================================
#[allow(dead_code)]
pub async fn try_add_to_stake(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Keypair,
    stake_id: u64,
    amount: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &participant.pubkey());

    let staking_account_pda = derive_staking_account_pda(
        &context.program_id,
        &participant.pubkey(),
        stake_id,
    );

    let args = vec![amount.to_string()];

    let accounts = soccial_accounts::ReinforceStake {
        caller: caller.pubkey(),
        participant: participant.pubkey().into(),
        participant_token_account: seeds.user_token_ata,
        staking_state: seeds.staking_state,
        staking_account: staking_account_pda,
        staking_vault_token_account: seeds.staking_vault_token_account,
        staking_vault: seeds.staking_vault,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        token_mint: seeds.token_mint,
        destination_token_account: seeds.user_token_ata,
        mint_authority: seeds.mint_authority,
        user_access: None,
        token_state: seeds.token_state,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::AddTokensToStake { args }.data(),
    };

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller, participant],
        ix,
        context.recent_blockhash,
    )
    .await?;

    Ok(())
}




// ============================================================================
/// Withdraws staked tokens after the lockup period has ended.
///
/// This releases the principal from the staking vault to the participant’s wallet.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `participant`: Staking account owner
/// - `stake_id`: Unique ID of the staking record
///
/// # Returns:
/// `Ok(())` if withdrawn successfully
///
/// # Example:
/// ```
/// try_withdraw_staked_tokens(&mut context, &admin, &user.pubkey(), 0).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_withdraw_staked_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    stake_id: u64,
) -> Result<(), TransportError> {
    let staking_account_pda = derive_staking_account_pda(
        &context.program_id,
        participant,
        stake_id,
    );

    let seeds = derive_seeds(&context.program_id, participant);

     let token_state_data = context
        .banks_client
        .get_account(seeds.token_state)
        .await?
        .expect("token_state should exist");

    let token_state = TokenState::try_deserialize(&mut &token_state_data.data[..])
     .expect("Failed to deserialize token_state");
    
    let accounts = soccial_accounts::WithdrawStaked {
        caller: caller.pubkey(),
        token_state: seeds.token_state,
        user_access: None,
        staking_account: staking_account_pda,
        recipient_of_lamports: token_state.core.owner, 
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        staking_vault_token_account: seeds.staking_vault_token_account,
        staking_vault: seeds.staking_vault,
        destination_token_account: seeds.user_token_ata,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        clock: clock::ID,
    }.to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::WithdrawStakedTokens {}.data(),
    };

    send_ix(&mut context.banks_client, &context.payer, &[&context.payer, caller], ix, context.recent_blockhash).await?;

    Ok(())
}

// ============================================================================
/// Claims staking rewards accrued over time for a given stake.
///
/// This transfers earned rewards to the participant’s associated token account.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Function initiator (typically admin or participant)
/// - `participant`: Wallet address of user
/// - `stake_id`: ID of staking entry
///
/// # Returns:
/// `Ok(())` if rewards were claimed successfully
///
/// # Example:
/// ```
/// try_claim_staking_rewards(&mut context, &user, &user.pubkey(), 0).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_claim_staking_rewards(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    stake_id: u64,
) -> Result<(), TransportError> {
    // Derive staking PDA with the stake_id
    let staking_account_pda = derive_staking_account_pda(
        &context.program_id,
        participant,
        stake_id,
    );

    let seeds = derive_seeds(&context.program_id, participant);

    let accounts = soccial_accounts::ReleaseStaked {
        caller: caller.pubkey(),
        token_state: seeds.token_state,
        user_access: None,
        staking_account: staking_account_pda,
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        staking_vault_token_account: seeds.staking_vault_token_account,
        staking_vault: seeds.staking_vault,
        destination_token_account: seeds.user_token_ata,
        token_program: spl_token::ID,
        system_program: system_program::ID,
        clock: clock::ID,
    }.to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::ClaimStakingRewards {}.data(),
    };

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
/// Adds a new staking plan with lockup duration and APR.
///
/// Useful for registering new tiers of staking for users to choose from.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Admin signer
/// - `plan_id`: Unique numeric ID of the plan
/// - `lockup_duration`: Duration in seconds before tokens can be withdrawn
/// - `apr_bps`: APR in basis points (100 = 1%)
///
/// # Returns:
/// `Ok(())` if added, or error otherwise
///
/// # Example:
/// ```
/// try_add_staking_plan(&mut context, &admin, 1, 3600 * 30, 700).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_add_staking_plan(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    plan_id: u8,
    lockup_duration: i64,
    apr_bps: u16,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let args = vec![
        plan_id.to_string(),
        lockup_duration.to_string(),
        apr_bps.to_string(),
    ];

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageStaking {
            caller: caller.pubkey(),
            staking_state: seeds.staking_state,
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_instruction::AddStakingPlan { args },
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

#[allow(dead_code)]
pub async fn try_edit_staking_plan(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    plan_id: u8,
    lockup_duration: i64,
    apr_bps: u16,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let args = vec![
        plan_id.to_string(),
        lockup_duration.to_string(),
        apr_bps.to_string(),
    ];

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageStaking {
            caller: caller.pubkey(),
            staking_state: seeds.staking_state,
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_instruction::EditStakingPlan { args },
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
/// Edits an existing staking plan’s parameters (lockup and APR).
///
/// Allows adjusting plan conditions while preserving the plan ID.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Admin keypair
/// - `plan_id`: ID of the plan to update
/// - `lockup_duration`: New lockup time (in seconds)
/// - `apr_bps`: New APR (in basis points)
///
/// # Returns:
/// `Ok(())` if update succeeded
///
/// # Example:
/// ```
/// try_edit_staking_plan(&mut context, &admin, 1, 86400 * 90, 1200).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_disable_staking_plan(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    plan_id: u8,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let args = vec![plan_id.to_string()];

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageStaking {
            caller: caller.pubkey(),
            staking_state: seeds.staking_state,
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_instruction::DisableStakingPlan { args },
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
