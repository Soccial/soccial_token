// ============================================================================
// File: trymethods.rs
// Soccial Token – Instruction Invocation Helpers for Tests
// ----------------------------------------------------------------------------
//
// This module provides **helper functions** for executing Soccial Token
// instructions in integration tests. These functions abstract the process
// of building, signing, and sending transactions, allowing test code to
// focus on logic rather than boilerplate.
//
// ----------------------------------------------------------------------------
// Features:
// - Encapsulates logic to call program instructions with minimal repetition
// - Supports automatic seed derivation and PDA resolution
// - Works seamlessly with `EnvProgramTestContext`
// - Designed for integration testing of custom instructions
//
// ----------------------------------------------------------------------------
// Naming Convention:
// All functions use the `try_*` prefix (e.g., `try_airdrop`, `try_stake`) to
// indicate they return a `Result<(), TransportError>` and are used in tests.
//
// ----------------------------------------------------------------------------
// Example:
// try_airdrop(&mut context, &owner, &recipient.pubkey(), 1_000_000, None).await?;
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use crate::testutils::environment::EnvProgramTestContext;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use solana_sdk::transport::TransportError;
use soccial_token::{instruction as soccial_instruction, accounts as soccial_accounts};
use crate::testutils::basics::*;

// ============================================================================
/// Attempts to execute an airdrop transfer from the airdrop vault to a recipient.
///
/// This helper wraps the `Airdrop` instruction of the Soccial Token program and:
/// - Derives the necessary PDAs (vault, ATA, token state)
/// - Optionally includes a string `reason` for audit/tracking
/// - Submits the transaction with required signers
///
/// # Parameters:
/// - `context`: The mutable reference to the test environment context
/// - `caller`: The authorized signer initiating the airdrop (e.g. owner or admin)
/// - `recipient`: The public key of the user receiving the airdrop
/// - `amount`: Token amount (in base units) to send
/// - `reason`: Optional string reason (e.g., `"referral_bonus"`, `"welcome"`)
///
/// # Returns:
/// - `Ok(())` on success
/// - `Err(TransportError)` if the transaction fails
///
/// # Example:
/// ```
/// try_airdrop(&mut context, &owner, &user.pubkey(), 500_000, Some("airdrop campaign")).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_airdrop(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    recipient: &Pubkey,
    amount: u64,
    reason: Option<&str>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, recipient); 

    let mut args = vec![amount.to_string()];
    if let Some(r) = reason {
        args.push(r.to_string());
    }

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::ManageAirdrop {
            caller: caller.pubkey(),
            token_state: seeds.token_state,
            airdrop_vault: seeds.airdrop_vault,
            airdrop_vault_token_account: seeds.airdrop_vault_token_account,
            mint: seeds.token_mint,
            recipient_token_account: seeds.user_token_ata,
            user_access: None,
            token_program: spl_token::ID,
        },
        soccial_instruction::Airdrop {args },
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
