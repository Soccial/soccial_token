// ======================================================================
// Refactored Test File: test_airdrop.rs using try_ helpers
// ======================================================================

use solana_program_test::*;
use solana_sdk::{
    program_pack::Pack, signature::{Keypair, Signer}, transport::TransportError
};
use soccial_token::{airdrop::AirdropError, utils::error::ErrorCode, economy::MAX_AIRDROP_AMOUNT};
mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryairdrop::*;

#[tokio::test]
async fn test_airdrop_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let recipient = Keypair::new();

    let recipient_ata = create_user_ata(&mut context, &recipient).await?;

    try_airdrop(&mut context, &owner, &recipient.pubkey(), 1000, Some("test")).await?;

    // Verify
    let recipient_ata_account = context.banks_client.get_account(recipient_ata).await?.unwrap();
    let token_account = spl_token::state::Account::unpack(&recipient_ata_account.data).unwrap();
    assert_eq!(token_account.amount, 1000);

    log_all_balances(&mut context, &owner).await?;

    Ok(())
}

#[tokio::test]
async fn test_airdrop_invalid_amount_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let recipient = Keypair::new();

    // Ensure the recipient's associated token account exists
    create_user_ata(&mut context, &recipient).await?;

    let result = try_airdrop(&mut context, &owner, &recipient.pubkey(), 0, Some("test")).await;
    assert_custom_error(result, AirdropError::InvalidAmount, "Invalid amount (negative or overflow)");
    Ok(())
}

#[tokio::test]
async fn test_airdrop_unauthorized_should_fail() -> Result<(), TransportError> {
    // Setup environment and owner
    let (mut context, _owner) = setup_test_env().await;

    // Create hacker and recipient keypairs
    let hacker = Keypair::new();
    let recipient = Keypair::new();

    // Fund hacker and recipient accounts with SOL
    fund_test_accounts_manual(
        &mut context.banks_client,
        &[&hacker, &recipient],
        1_000_000_000,
        &context.payer,
        &context.recent_blockhash,
    ).await;

    // Create token accounts (ATAs) for hacker and recipient
    create_user_ata(&mut context, &hacker).await?;
    create_user_ata(&mut context, &recipient).await?;

    // Mint tokens to the hacker's token account, using the authorized owner
    context.mint_tokens(&hacker, 10_000_000).await;
    //context.transfer_tokens_to_user(&owner, &hacker.pubkey(), 10_000_000).await?;
  

    // Attempt to airdrop from the hacker (unauthorized)
    let result = try_airdrop(&mut context, &hacker, &recipient.pubkey(), 1_000, Some("test")).await;
  
    // Assert that the operation fails with Unauthorized error
    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error during airdrop.");

    Ok(())
}


#[tokio::test]
async fn test_airdrop_exceeds_max_limit_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let recipient = Keypair::new();

    // Ensure the recipient's associated token account exists
    create_user_ata(&mut context, &recipient).await?;

    let result = try_airdrop(&mut context, &owner, &recipient.pubkey(), MAX_AIRDROP_AMOUNT + 1,  Some("test")).await;
    assert_custom_error(result, AirdropError::ExceedsPerAirdropLimit, "Exceeds per-airdrop limit should fail");
    Ok(())
}

#[tokio::test]
async fn test_airdrop_zero_amount_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;
    let recipient = Keypair::new();

    // Ensure the recipient's associated token account exists
    create_user_ata(&mut context, &recipient).await?;

    let result = try_airdrop(&mut context, &owner, &recipient.pubkey(),  0, Some("test")).await;
    assert_custom_error(result, AirdropError::InvalidAmount, "Zero amount should fail");
    Ok(())
}
