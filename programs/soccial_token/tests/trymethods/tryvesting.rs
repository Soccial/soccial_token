// ============================================================================
// Soccial Token – Vesting Schedule Test Helpers
// ----------------------------------------------------------------------------
//
// This module provides integration test helpers for managing vesting logic
// in the Soccial Token smart contract, designed for use with Solana `ProgramTest`.
//
// These high-level wrappers simulate real contract flows such as:
// - Creating and updating vesting schedules
// - Canceling or locking vesting plans (immutability)
// - Claiming vested tokens over time
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Create schedules with cliffs, cycles, and durations
// ✔ Update or cancel schedules (if not immutable)
// ✔ Mark vesting entries as immutable
// ✔ Claim unlocked tokens after warp
// ✔ Fully test vesting flows with on-chain state
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_create_vesting_schedule`: Create a new schedule for a user
// - `try_update_vesting_schedule`: Modify an existing schedule
// - `try_cancel_vesting_schedule`: Cancel and recover unclaimed tokens
// - `try_set_vesting_immutable`: Lock schedule from edits
// - `try_claim_vested_tokens`: Claim unlocked tokens
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use crate::testutils::environment::EnvProgramTestContext;
use crate::testutils::basics::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transport::TransportError};
use spl_associated_token_account::{get_associated_token_address, ID as ASSOCIATED_TOKEN_PROGRAM_ID};
use spl_token::ID as TOKEN_PROGRAM_ID;
use soccial_token::{accounts as soccial_accounts, instruction as soccial_instruction, token::TokenState};
use anchor_lang::{solana_program, AccountDeserialize, InstructionData, ToAccountMetas};
use solana_program::sysvar::clock;

/// Derives seeds for a vesting schedule given a participant and a vesting ID (u64).
pub fn derive_vesting_schedule_pda(
    program_id: &Pubkey,
    participant: &Pubkey,
    vesting_id: u64,
) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"vesting_schedule",
            participant.as_ref(),
            &vesting_id.to_le_bytes(),
        ],
        program_id,
    ).0
}

// ============================================================================
/// Attempts to create a new vesting schedule for a participant.
///
/// This helper wraps the `CreateVestingSchedule` instruction, handling:
/// - PDA derivation (vesting state & schedule)
/// - Parameter encoding for vesting logic (start time, cliffs, cycles, tokens)
///
/// # Parameters:
/// - `context`: Test environment instance
/// - `caller`: Authorized signer
/// - `participant`: The beneficiary of the vesting schedule
/// - `start_time`: Timestamp when vesting starts
/// - `cliff_duration`: Time before any tokens unlock
/// - `cycles`: Number of release cycles
/// - `vesting_duration`: Total duration of vesting
/// - `initial_tokens`: Immediate unlocked tokens
/// - `total_tokens`: Full amount to be vested
/// - `immutable`: If true, schedule cannot be updated or cancelled
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_create_vesting_schedule(&mut context, &owner, &user.pubkey(), now, 60, 3, 600, 0, 1000000, false).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_create_vesting_schedule(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    start_time: i64,
    cliff_duration: i64,
    cycles: i64,
    vesting_duration: i64,
    initial_tokens: u64,
    total_tokens: u64,
    immutable: bool
) -> Result<(), TransportError> {

    let seeds = derive_seeds(&context.program_id, participant);


    // Get vesting_id
    let vesting_state_account = context
        .banks_client
        .get_account(seeds.vesting_state)
        .await?
        .expect("VestingState should exist");

    let vesting_state_data = soccial_token::vesting::state::VestingState::try_deserialize(
        &mut &vesting_state_account.data[..]
    ).expect("Deserialization failed");

    let vesting_id = vesting_state_data.last_id;

    let vesting_seeds = derive_vesting_schedule_pda(&context.program_id, participant, vesting_id);

    let args = vec![
        participant.to_string(),
        start_time.to_string(),
        cliff_duration.to_string(),
        cycles.to_string(),
        vesting_duration.to_string(),
        initial_tokens.to_string(),
        total_tokens.to_string(),
        immutable.to_string()
    ];

    let accounts = soccial_accounts::ManageVesting {
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        participant: *participant,
        vesting_schedule: vesting_seeds,  
        vesting_state: seeds.vesting_state,
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        destination_token_account: seeds.user_token_ata,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        vesting_vault: seeds.vesting_vault,
        vesting_vault_token_account: seeds.vesting_vault_token_account,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
      
    }
    .to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::CreateVestingSchedule { args }.data(),
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
/// Attempts to cancel an existing vesting schedule.
///
/// This helper wraps the `CancelVestingSchedule` instruction, removing
/// a vesting entry and optionally returning unclaimed tokens to the liquidity vault.
///
/// # Parameters:
/// - `context`: Test environment instance
/// - `caller`: Authorized signer
/// - `participant`: Beneficiary of the vesting schedule
/// - `vesting_id`: Unique identifier for the schedule
///
/// # Returns:
/// `Ok(())` if cancelled, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_cancel_vesting_schedule(&mut context, &owner, &user.pubkey(), 0).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_cancel_vesting_schedule(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    vesting_id: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &participant);
    let vesting_schedule_pda =
        derive_vesting_schedule_pda(&context.program_id, participant, vesting_id);

    let args = vec![vesting_id.to_string()];

    let accounts = soccial_accounts::EditVestingSchedule {
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        participant: *participant,
        vesting_schedule: vesting_schedule_pda,
        vesting_state: seeds.vesting_state,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        vesting_vault: seeds.vesting_vault,
        vesting_vault_token_account: seeds.vesting_vault_token_account,
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        destination_token_account: seeds.user_token_ata,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::CancelVestingSchedule { args }.data(),
    };

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
/// Attempts to update the parameters of an existing vesting schedule.
///
/// This allows changing schedule values **unless the schedule is marked immutable**.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Signer with permission
/// - `participant`: User whose schedule is being edited
/// - `vesting_id`: Schedule ID
/// - `start_time`, `cliff_duration`, `cycles`, `vesting_duration`: Updated time parameters
/// - `initial_tokens`, `total_tokens`: Updated token values
/// - `immutable`: Whether to lock the schedule from future edits
///
/// # Returns:
/// `Ok(())` if updated, or error if invalid or unauthorized
///
/// # Example:
/// ```
/// try_update_vesting_schedule(&mut context, &admin, &user.pubkey(), 0, now + 60, 60, 3, 600, 0, 2000000, true).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_update_vesting_schedule(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    vesting_id: u64,
    start_time: i64,
    cliff_duration: i64,
    cycles: i64,
    vesting_duration: i64,
    initial_tokens: u64,
    total_tokens: u64,
    immutable: bool
) -> Result<(), TransportError> {
    
    let seeds = derive_seeds(&context.program_id, participant);

    let vesting_schedule_pda =
        derive_vesting_schedule_pda(&context.program_id, participant, vesting_id);

    let args = vec![
        vesting_id.to_string(),
        start_time.to_string(),
        cliff_duration.to_string(),
        cycles.to_string(),
        vesting_duration.to_string(),
        initial_tokens.to_string(),
        total_tokens.to_string(),
        immutable.to_string()
    ];

    let accounts = soccial_accounts::EditVestingSchedule {
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        participant: *participant,
        vesting_schedule: vesting_schedule_pda,
        vesting_state: seeds.vesting_state,
        liquidity_vault: seeds.liquidity_vault,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        vesting_vault: seeds.vesting_vault,
        vesting_vault_token_account: seeds.vesting_vault_token_account,
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        destination_token_account: seeds.user_token_ata,
        associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);
    
    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::UpdateVestingSchedule { args }.data(),
    };

   
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
/// Marks a vesting schedule as immutable, preventing future edits or cancellations.
///
/// Useful for securing schedules after final review.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer
/// - `participant`: The user whose schedule is targeted
/// - `vesting_id`: The ID of the schedule to lock
///
/// # Returns:
/// `Ok(())` if immutability set, or `TransportError` otherwise
///
/// # Example:
/// ```
/// try_set_vesting_immutable(&mut context, &admin, &user.pubkey(), 0).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_set_vesting_immutable(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    vesting_id: u64
) -> Result<(), TransportError> {
    
    let seeds = derive_seeds(&context.program_id, participant);

    let vesting_schedule_pda =
        derive_vesting_schedule_pda(&context.program_id, participant, vesting_id);

    let args = vec![
        vesting_id.to_string()
    ];

    let accounts = soccial_accounts::ImmutableVestingSchedule {
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        participant: *participant,
        vesting_schedule: vesting_schedule_pda,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    
    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::SetVestingImmutable { args }.data(),
    };
    
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
/// Claims tokens from a vesting schedule that are currently unlocked.
///
/// This helper wraps the `ClaimVestedTokens` instruction, transferring
/// vested tokens to the participant’s associated token account.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Participant or authorized signer
/// - `participant`: Address of the user claiming vested tokens
/// - `vesting_id`: ID of the vesting schedule
///
/// # Returns:
/// `Ok(())` if successful, or error if vesting not available yet
///
/// # Example:
/// ```
/// try_claim_vested_tokens(&mut context, &user, &user.pubkey(), 0).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_claim_vested_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    participant: &Pubkey,
    vesting_id: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, participant);
    let vesting_schedule_pda = derive_vesting_schedule_pda(&context.program_id, participant, vesting_id);

     let (vesting_vault, _vault_bump) = Pubkey::find_program_address(
        &[b"vesting_vault"],
        &context.program_id,
    );

    let token_state_data = context
        .banks_client
        .get_account(seeds.token_state)
        .await?
        .expect("token_state should exist");

    let token_state = TokenState::try_deserialize(&mut &token_state_data.data[..])
     .expect("Failed to deserialize token_state");

    // Derive the vesting_vault_token_account with authority = mint_authority
    let vesting_vault_token_account = get_associated_token_address(&vesting_vault, &seeds.token_mint);
    
    let accounts = soccial_accounts::ReleaseVestedTokens {
        caller: caller.pubkey(),
        user_access: None,
        token_state: seeds.token_state,
        vesting_schedule: vesting_schedule_pda,
        mint_authority: seeds.mint_authority,
        mint: seeds.token_mint,
        vesting_vault,
        vesting_vault_token_account,
        destination_token_account: seeds.user_token_ata,
        token_program: TOKEN_PROGRAM_ID,
        system_program: system_program::ID,
        clock: clock::ID,
        recipient_of_lamports: token_state.core.owner,
    }
    .to_account_metas(None);

    let ix = solana_sdk::instruction::Instruction {
        program_id: context.program_id,
        accounts,
        data: soccial_instruction::ClaimVestedTokens { }.data(),
    };

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

