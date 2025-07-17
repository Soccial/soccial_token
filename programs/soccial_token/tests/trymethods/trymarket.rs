// ============================================================================
// Soccial Token – Token Operations (Buy / Deposit / Transfer) Test Helpers
// ----------------------------------------------------------------------------
//
// This module provides helper functions to simulate core token flows such as:
// - Buying tokens from the liquidity vault
// - Depositing tokens into the off-chain reserve
// - Transferring tokens between users (with fee allocation)
//
// These are integration-level wrappers used during testing to ensure
// token economy mechanisms behave as expected.
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Simulate token purchases and fee distribution across vaults  
// ✔ Test deposits into the offchain reserve (for centralized exchanges)  
// ✔ Transfer tokens between accounts with dynamic fee calculations  
//
// ----------------------------------------------------------------------------
// Key Functions:
// - `try_buy_tokens`: Buys tokens from the liquidity vault  
// - `try_deposit_tokens`: Deposits tokens into the off-chain reserve  
// - `try_transfer_tokens`: Transfers tokens between users with fee logic  
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transport::TransportError};
use crate::testutils::{basics::*};
use crate::testutils::environment::EnvProgramTestContext;
use soccial_token::{self, instruction as soccial_instruction};

// ============================================================================
/// Buys tokens from the liquidity vault, applying fee-based distribution.
///
/// This simulates a user buying SCTK tokens, with fees allocated across
/// the rewards, revenue, and airdrop vaults.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Authorized contract signer
/// - `buyer`: Buyer of the tokens
/// - `amount`: Amount of tokens to purchase
/// - `fee_bps`: Fee in basis points (1 BPS = 0.01%)
///
/// # Returns:
/// `Ok(())` if transaction succeeded, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_buy_tokens(&mut context, &admin, &buyer, 1_000_000, 500).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_buy_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    buyer: &Keypair,
    amount: u64,
    fee_bps: u16,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &buyer.pubkey());
    let args = vec![
        amount.to_string(),
        fee_bps.to_string(),
    ];

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::BuyTokensContext {
            caller: caller.pubkey(),
            liquidity_vault: seeds.liquidity_vault,
            liquidity_vault_token_account: seeds.liquidity_vault_token_account,
            buyer_token_account: seeds.user_token_ata,

            rewards_vault: seeds.rewards_vault,
            rewards_vault_token_account: seeds.rewards_vault_token_account,
            revenue_vault: seeds.revenue_vault,
            revenue_vault_token_account: seeds.revenue_vault_token_account,
            airdrop_vault: seeds.airdrop_vault,
            airdrop_vault_token_account: seeds.airdrop_vault_token_account,

            token_mint: seeds.token_mint,
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
           
        },
        soccial_instruction::BuyTokens { args },
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
/// Deposits tokens into the off-chain reserve (e.g., for CEX custody).
///
/// Simulates moving tokens to the `offchain_reserve_vault`, deducting
/// fees and distributing them across vaults.
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Authorized signer
/// - `recipient`: User depositing the tokens
/// - `amount`: Amount to deposit (raw units)
/// - `fee_bps`: Fee in basis points
///
/// # Returns:
/// `Ok(())` if transaction succeeded, or `TransportError` on failure
///
/// # Example:
/// ```
/// try_deposit_tokens(&mut context, &admin, &user.pubkey(), 5_000_000, 300).await?;
/// ```
// ============================================================================
#[allow(dead_code)]
pub async fn try_deposit_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    recipient: &Pubkey,
    amount: u64,
    fee_bps: u16,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, recipient);
    let args = vec![
        amount.to_string(),
        fee_bps.to_string(),
    ];
    
    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::DepositTokensContext {
            caller: caller.pubkey(),
            offchain_reserve_vault: seeds.offchain_reserve_vault,
            offchain_reserve_vault_token_account: seeds.offchain_reserve_vault_token_account,
            destination_authority: *recipient,
            destination_token_account: seeds.user_token_ata,

            rewards_vault: seeds.rewards_vault,
            rewards_vault_token_account: seeds.rewards_vault_token_account,
            revenue_vault: seeds.revenue_vault,
            revenue_vault_token_account: seeds.revenue_vault_token_account,
            airdrop_vault: seeds.airdrop_vault,
            airdrop_vault_token_account: seeds.airdrop_vault_token_account,

            token_mint: seeds.token_mint,
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
        },
        soccial_instruction::DepositTokens { args },
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
/// Transfers tokens between users with fee distribution to vaults.
///
/// Simulates a token transfer with protocol-level fees, supporting
/// vault-based allocation (e.g., rewards, revenue, airdrop).
///
/// # Parameters:
/// - `context`: Test context
/// - `caller`: Signer triggering the transfer
/// - `sender`: Token sender account
/// - `recipient_ata`: Associated token account of the recipient
/// - `amount`: Token amount to send
/// - `fee_bps`: Fee in basis points to apply
///
/// # Returns:
/// `Ok(())` if transfer succeeded, or error on failure
///
/// # Example:
/// ```
/// try_transfer_tokens(&mut context, &admin, &user, recipient_ata, 10_000_000, 250).await?;
/// ```
// ============================================================================

#[allow(dead_code)]
pub async fn try_transfer_tokens(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
    sender: &Keypair,
    recipient_ata: Pubkey,
    amount: u64,
    fee_bps: u16,
) -> Result<(), TransportError> {
    let seeds = derive_seeds(&context.program_id, &sender.pubkey());
    let args = vec![
        amount.to_string(),
        fee_bps.to_string(),
    ];

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::TransferTokensContext {
            caller: caller.pubkey(),
            sender: sender.pubkey(),
            sender_token_account: seeds.user_token_ata,
            recipient_token_account: recipient_ata,

            rewards_vault: seeds.rewards_vault,
            rewards_vault_token_account: seeds.rewards_vault_token_account,
            revenue_vault: seeds.revenue_vault,
            revenue_vault_token_account: seeds.revenue_vault_token_account,
            airdrop_vault: seeds.airdrop_vault,
            airdrop_vault_token_account: seeds.airdrop_vault_token_account,

            token_mint: seeds.token_mint,
            user_access: None,
            token_state: seeds.token_state,
            token_program: spl_token::ID,
        },
        soccial_instruction::TransferTokens { args },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, caller, sender],
        ix,
        context.recent_blockhash,
    ).await?;
    
    Ok(())
}
