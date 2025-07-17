use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;
use crate::testutils::basics::{assert_custom_error, derive_user_ata};
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::trymarket::*;

#[tokio::test]
async fn test_transfer_tokens_success() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let program_id = context.program_id;

    let sender = Keypair::new();
    let recipient = Keypair::new();

    create_user_ata(&mut context, &sender).await?;
    create_user_ata(&mut context, &recipient).await?;

    let recipient_ata = derive_user_ata(&program_id, &recipient.pubkey());

    let initial_deposit = 20_000_000_000;
    let transfer_amount = 15_000_000_000;
    let fee_bps = 20u64; // 0.2%

    context.mint_tokens_to_user(&sender.pubkey(), initial_deposit).await;
    
    let state = context.load_token_state().await;
    let rewards_fee_bps = state.fee.rewards_fee_bps as u64;
    let airdrop_fee_bps = state.fee.airdrop_fee_bps as u64;

    let total_fee = transfer_amount * fee_bps / 10_000;
    let net_to_recipient = transfer_amount - total_fee;
    let to_rewards = total_fee * rewards_fee_bps / 10_000;
    let to_airdrop = total_fee * airdrop_fee_bps / 10_000;
    let to_revenue = total_fee - to_rewards - to_airdrop;

    let sender_before = context.get_user_balance(&sender.pubkey()).await;
    let recipient_before = context.get_user_balance(&recipient.pubkey()).await;
    let rewards_before = context.get_vault_balance("rewards").await;
    let airdrop_before = context.get_vault_balance("airdrop").await;
    let revenue_before = context.get_vault_balance("revenue").await;
    
    try_transfer_tokens(
        &mut context,
        &admin,
        &sender,
        recipient_ata,
        transfer_amount,
        fee_bps as u16,
    )
    .await?;

    context.refresh().await;

    let sender_after = context.get_user_balance(&sender.pubkey()).await;
    let recipient_after = context.get_user_balance(&recipient.pubkey()).await;
    let rewards_after = context.get_vault_balance("rewards").await;
    let airdrop_after = context.get_vault_balance("airdrop").await;
    let revenue_after = context.get_vault_balance("revenue").await;

    assert_eq!(
        sender_before - sender_after,
        transfer_amount,
        "❌ Sender should have lost exactly {} tokens, but didn't.",
        transfer_amount
    );

    assert_eq!(
        recipient_after - recipient_before,
        net_to_recipient,
        "❌ Recipient should have received exactly {} tokens, but didn’t.",
        net_to_recipient
    );

    assert_eq!(
        rewards_after - rewards_before,
        to_rewards,
        "❌ Rewards vault should receive {} tokens, but received {}",
        to_rewards,
        rewards_after - rewards_before
    );

    assert_eq!(
        airdrop_after - airdrop_before,
        to_airdrop,
        "❌ Airdrop vault should receive {} tokens, but received {}",
        to_airdrop,
        airdrop_after - airdrop_before
    );

    assert_eq!(
        revenue_after - revenue_before,
        to_revenue,
        "❌ Revenue vault should receive {} tokens, but received {}",
        to_revenue,
        revenue_after - revenue_before
    );

    log_all_balances(&mut context, &admin).await?;

    Ok(())
}

#[tokio::test]
async fn test_transfer_tokens_insufficient_balance_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    let program_id = context.program_id;

    let sender = Keypair::new();
    let recipient = Keypair::new();

    create_user_ata(&mut context, &sender).await?;
    create_user_ata(&mut context, &recipient).await?;

    let excessive_amount = 100_000_000_000;
    let fee_bps = 20;

    let result = try_transfer_tokens(
        &mut context,
        &admin,
        &sender,
        derive_user_ata(&program_id, &recipient.pubkey()),
        excessive_amount,
        fee_bps,
    )
    .await;

    assert!(
        result.is_err(),
        "Expected failure due to insufficient balance, but got success"
    );

    Ok(())
}


#[tokio::test]
async fn test_transfer_tokens_without_permissions_should_fail() -> Result<(), TransportError> {
    let (mut context, _admin) = setup_test_env().await;
    let program_id = context.program_id;

    let sender = Keypair::new();
    let recipient = Keypair::new();
    let intruder = Keypair::new();

    create_user_ata(&mut context, &sender).await?;
    create_user_ata(&mut context, &recipient).await?;
    create_user_ata(&mut context, &intruder).await?;

    let excessive_amount = 100_000_000_000;
    let fee_bps = 200;

    let result = try_transfer_tokens(
        &mut context,
        &intruder,
        &sender,
        derive_user_ata(&program_id, &recipient.pubkey()),
        excessive_amount,
        fee_bps,
    )
    .await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error when transferirg tokens.");

    Ok(())
}
