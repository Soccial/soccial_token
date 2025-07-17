// ============================================================================
// Soccial Token – Minting & Contract Management Helpers
// ----------------------------------------------------------------------------
//
// This module contains helper functions to test mint-related and global
// contract control instructions (like pausing, resuming, fee updates, etc.).
//
// These helpers wrap key instructions such as `PauseContract`, `SetApiAuthority`,
// and various economy configuration methods.
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Set the API authority for external integrations  
// ✔ Pause or resume the contract globally  
// ✔ Update tokenomics (fees for rewards and airdrops)  
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_set_api_authority`: Assigns new API signer  
// - `try_pause_contract` / `try_resume_contract`: Toggle pause state  
// - `try_update_rewards_fee` / `try_update_airdrop_fee`: Modify system fees  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use crate::testutils::environment::EnvProgramTestContext;
use crate::testutils::basics::*;
use solana_sdk::{

    pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transport::TransportError
};

use soccial_token::{instruction as soccial_instruction, accounts as soccial_accounts};


// ============================================================================
/// Attempts to set the API authority for the contract.
///
/// Used to update the signer allowed to perform API-controlled instructions.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Owner or authorized signer
/// - `new_api_pubkey`: New public key to assign as API authority
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_set_api_authority(&mut context, &admin, &new_api).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_set_api_authority(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    new_api_pubkey: &Pubkey,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageContract {
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::SetApiAuthority {
            args: vec![new_api_pubkey.to_string()],
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
/// Attempts to pause the contract using a valid admin or owner.
///
/// Pauses all on-chain activity gated by the pause check flag.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Signer with authority (owner or admin)
///
/// # Returns:
/// `Ok(())` if successful, or `TransportError` if rejected
///
/// # Example:
/// ```
/// try_pause_contract(&mut context, &owner).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_pause_contract(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageContract {
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::PauseContract {},
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
/// Attempts to resume the contract using a valid admin or owner.
///
/// Resumes activity after the contract has been paused.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Signer with authority (owner or admin)
///
/// # Returns:
/// `Ok(())` if resumed, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_resume_contract(&mut context, &owner).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_resume_contract(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageContract {
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::ResumeContract {},
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
/// Attempts to update the rewards fee in the contract settings.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer (must have economy permissions)
/// - `args`: Vector with new fee in BPS (e.g., `vec!["500"]`)
///
/// # Returns:
/// `Ok(())` if fee is updated, or `TransportError` otherwise
///
/// # Example:
/// ```
/// try_update_rewards_fee(&mut context, &admin, vec!["400"], 1).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_update_rewards_fee(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    args: Vec<String>,
    proposal_id: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    // Derive proposal PDA a partir do ID
    let (proposal, _) = Pubkey::find_program_address(
        &[b"proposal", &proposal_id.to_le_bytes()],
        &context.program_id,
    );

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageContractGovernance {
            caller: caller.pubkey(),
            token_state: seeds.token_state,
            governance_state: seeds.governance_state,
            proposal: proposal,
            user_access: None,
            system_program: system_program::ID,
        },
        soccial_instruction::UpdateRewardsFee {
            args,
        },
    );

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
/// Attempts to update the airdrop fee in the contract settings.
///
/// # Parameters:
/// - `context`: Test environment
/// - `caller`: Authorized signer (must have economy permissions)
/// - `args`: Vector with new fee in BPS (e.g., `vec!["250"]`)
///
/// # Returns:
/// `Ok(())` if updated, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_update_airdrop_fee(&mut context, &admin, vec!["200"], 1).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_update_airdrop_fee(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    args: Vec<String>,
    proposal_id: u64,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    // Derive proposal PDA using ID
    let (proposal, _) = Pubkey::find_program_address(
        &[b"proposal", &proposal_id.to_le_bytes()],
        &context.program_id,
    );

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::ManageContractGovernance {
            caller: caller.pubkey(),
            token_state: seeds.token_state,
            governance_state: seeds.governance_state,
            proposal,
            user_access: None,
            system_program: system_program::ID,
        },
        soccial_token::instruction::UpdateAirdropFee { args },
    );

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
