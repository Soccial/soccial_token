
// ============================================================================
// Soccial Token – Governance Instruction Helpers for Tests
// ----------------------------------------------------------------------------
//
// This module provides test wrappers for interacting with the governance
// system of the Soccial Token program during integration testing.
//
// These helpers simplify the flow of:
// - Creating proposals
// - Voting and finalizing them
// - Updating governance parameters dynamically
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Create time-based or immediate proposals with types and metadata  
// ✔ Vote on proposals and simulate majority outcomes  
// ✔ Finalize proposals after quorum or duration ends  
// ✔ Update governance rules via contract-level parameters  
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_create_proposal`: Propose changes with description and type  
// - `try_vote`: Cast a vote (support or reject)  
// - `try_finalize_proposal`: Close and evaluate proposals  
// - `try_update_governance_settings`: Change config values (quorum, durations, etc.)  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use anchor_lang::{solana_program, system_program};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transport::TransportError};
use crate::testutils::environment::{create_user_ata, fund_lamports, EnvProgramTestContext};
use crate::testutils::basics::*;
use crate::testutils::environment::get_proposal_last_id;
use soccial_token::{instruction as soccial_instruction, accounts as soccial_accounts};
use solana_program::sysvar::clock;

// ============================================================================
/// Submits a new governance proposal with optional start time and voting duration.
///
/// # Parameters:
/// - `context`: Current test context
/// - `caller`: Signer proposing the change
/// - `description`: Human-readable proposal summary
/// - `proposal_types`: Vector of encoded types (e.g., "SystemUpgrade")
/// - `start_time`: Optional UNIX timestamp to delay voting start
/// - `duration`: Optional voting period in seconds
///
/// # Returns:
/// `Ok(())` if successfully submitted, or error if transaction fails
///
/// # Example:
/// ```
/// try_create_proposal(&mut context, &admin, "Enable buyback".into(), vec!["SystemUpgrade".into()], None, Some(600)).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_create_proposal(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    description: String,
    proposal_types: Vec<String>,
    start_time: Option<i64>,
    duration: Option<i64>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let current_count = get_proposal_last_id(context, seeds.governance_state).await?;
    let (_proposal_account, _) = derive_proposal_account(&context.program_id, current_count);

    // Prepare args for instruction
    let mut args = vec![description];
    args.extend(proposal_types);

    // Optional values (casted to string)
    if let Some(start) = start_time {
        args.push(start.to_string());
    }

    if let Some(dur) = duration {
        args.push(dur.to_string());
    }

    // Prepare instruction
    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::CreateProposal {
            caller: caller.pubkey(),
            governance_state: seeds.governance_state,
            proposal_account: _proposal_account,
            user_access: None,
            token_state: seeds.token_state,
            system_program: solana_sdk::system_program::ID,
        },
        soccial_instruction::CreateProposal { args },
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
/// Finalizes a governance proposal after voting has ended.
///
/// This evaluates the votes, checks quorum, and finalizes the result.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Signer finalizing the proposal
/// - `proposal_account`: The proposal's PDA
///
/// # Returns:
/// `Ok(())` on success, error if not finalized or still pending
///
/// # Example:
/// ```
/// try_finalize_proposal(&mut context, &admin, proposal_pda).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_finalize_proposal(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    proposal_account: Pubkey,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::FinalizeProposal {
            caller: caller.pubkey(),
            governance_state: seeds.governance_state,
            proposal_account, 
            user_access: None,
            token_state: seeds.token_state,
            system_program: solana_sdk::system_program::ID,
            clock: clock::ID,
        },
        soccial_instruction::FinalizeProposal {
            //args: vec![],
        },
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
/// Casts a vote (support or reject) on a governance proposal.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Voting user
/// - `proposal_account`: Target proposal
/// - `support`: `true` to support, `false` to reject
///
/// # Returns:
/// `Ok(())` on vote cast, or error on failure
///
/// # Example:
/// ```
/// try_vote(&mut context, &user, proposal_pda, true).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_vote(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    proposal_account: Pubkey, 
    support: bool,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let vote_receipt = Pubkey::find_program_address(
        &[
            b"vote",
            &proposal_account.to_bytes(),
            &caller.pubkey().to_bytes(),
        ],
        &context.program_id,
    ).0;

    let total_offchain: u64 = 0;
    let total_staking: u64 = 0;

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::VoteOnProposal {
            caller: caller.pubkey(),                            
            governance_state: seeds.governance_state,
            proposal_account,
            user_access: None,
            token_mint: seeds.token_mint,
            vote_receipt, 
            user_token_account: seeds.user_token_ata,
            token_state: seeds.token_state,
            token_program: anchor_spl::token::ID,
            system_program: solana_sdk::system_program::ID,
            clock: clock::ID,
        },
        soccial_instruction::Vote { 
            args: vec![
                support.to_string(),
                total_offchain.to_string(),    
                total_staking.to_string(),  
            ],
        },
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
/// Updates governance configuration such as quorum, duration, or thresholds.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Authorized signer (e.g., owner)
/// - `updates`: Vector of key=value string settings (e.g., "quorum_percent=51")
///
/// # Returns:
/// `Ok(())` if applied, or error if unauthorized or invalid
///
/// # Example:
/// ```
/// try_update_governance_settings(&mut context, &owner, vec!["quorum_percent=66".into()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_update_governance_settings(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    updates: Vec<String>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageGovernance {
            caller: caller.pubkey(),
            governance_state: seeds.governance_state,
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::UpdateGovernanceSettings { args: updates },
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

/// Runs a full governance flow:
/// - Creates a proposal
/// - Distributes tokens to 5 voters
/// - All vote in favor
/// - Simulates passage of time
/// - Finalizes the proposal as approved
///
/// # Parameters:
/// - `context`: Test context
/// - `admin`: Owner or authority keypair
///
/// # Returns:
/// `Ok(())` on success, or error if any step fails
#[allow(dead_code)]
pub async fn try_approve_proposal_flow(
    context: &mut EnvProgramTestContext,
    admin: &Keypair,
    description: String,
    proposal_types: Vec<String>,
) -> Result<u64, TransportError> {
   
    let now = context.get_current_unix_timestamp().await;
   
    let seeds = derive_seeds(&context.program_id, &admin.pubkey()); 

    // Step 1: Create proposal
    try_create_proposal(context, admin, description, proposal_types, Some(now), Some(300)).await?;
    
    let last_id = get_proposal_last_id(context, seeds.governance_state).await?;


    let proposal_id = last_id - 1;
    
    let (proposal_pda, _) = derive_proposal_account(&context.program_id, proposal_id);
    
    // Step 2: Create 5 voters
    let mut voters = vec![];
    for _ in 0..5 {
        let user = Keypair::new();
        fund_lamports(context, &user, 10_000_000).await?;
        create_user_ata(context, &user).await?;
        context.mint_tokens_to_user(&user.pubkey(), 1_500_000_000_000_0000).await;
        voters.push(user);
    }

    // Step 3: Vote in favor
    for user in &voters {
        try_vote(context, user, proposal_pda, true).await?;
    }

    // Step 4: Warp time
    context.warp_forward_slots(1000).await;

    // Step 5: Finalize
    try_finalize_proposal(context, admin, proposal_pda).await?;

    Ok(proposal_id)
}
