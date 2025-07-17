// ============================================================================
// Soccial Token – System Logging Test Helper
// ----------------------------------------------------------------------------
//
// This module provides a wrapper function to trigger system-level logging
// from the Soccial Token smart contract. It allows injecting log messages
// via an Anchor instruction for testing or diagnostics.
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Emit custom logs from the program during test execution  
// ✔ Useful for debug tracing and integration test assertions  
//
// ----------------------------------------------------------------------------
// Key Function:
// - `try_emit_system_log`: Sends args to the `EmitSystemLog` instruction
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
    signature::Keypair, signer::Signer, transport::TransportError
};

use soccial_token::{instruction as soccial_instruction, accounts as soccial_accounts};

// ============================================================================
/// Emits a custom log from within the Soccial Token program for testing.
///
/// This wraps the `EmitSystemLog` instruction, which allows sending debug
/// or event-based logs to the Anchor runtime output.
///
/// # Parameters:
/// - `context`: ProgramTest environment
/// - `caller`: Authorized signer
/// - `args`: Vector of log message arguments (arbitrary strings)
///
/// # Returns:
/// `Ok(())` if the instruction is processed successfully
///
/// # Example:
/// ```
/// try_emit_system_log(&mut context, &admin, vec!["INIT".into(), "vault_check".into()]).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_emit_system_log(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    args: Vec<String>,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_accounts::EmitSystemLog {
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_instruction::EmitSystemLog { args },
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
