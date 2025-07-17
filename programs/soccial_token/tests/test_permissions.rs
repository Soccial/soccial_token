
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
};
use soccial_token::utils::error::ErrorCode;
mod testutils;
mod trymethods;
use crate::testutils::basics::*;
use crate::testutils::environment::setup_test_env;
use crate::trymethods::tryuser::*;

#[tokio::test]
async fn test_emit_log_with_permission_should_succeed() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;


    let logger_user = Keypair::new();
    context.mint_tokens(&owner, 1_000_000).await;

    // Grant the "emit_log" permission (this will create user_access account)
    try_assign_permission(
        &mut context,
        &owner,
        &logger_user.pubkey(),
        vec!["emit_log".to_string()],
    ).await.unwrap();

    context.refresh().await;

    let seeds = derive_seeds(&context.program_id, &logger_user.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::EmitSystemLog {
            caller: logger_user.pubkey(),
            user_access: Some(seeds.user_access),
            token_state: seeds.token_state,
        },
        soccial_token::instruction::EmitSystemLog {
            args: vec!["Hello from test!".to_string()],
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &logger_user],
        ix,
        context.recent_blockhash,
    ).await?;

    Ok(())
}

#[tokio::test]
async fn test_emit_log_without_permission_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new();
    context.mint_tokens( &owner, 1_000_000).await;

    // Give the user a different permission, so the account exists but lacks "emit_log"
    try_assign_permission(
        &mut context,
        &owner,
        &user.pubkey(),
        vec!["manage_contract".to_string()],
    ).await.unwrap();

    context.refresh().await;

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::EmitSystemLog {
            caller: user.pubkey(),
            user_access: Some(derive_seeds(&context.program_id, &user.pubkey()).user_access),
            token_state: derive_seeds(&context.program_id, &user.pubkey()).token_state,
        },
        soccial_token::instruction::EmitSystemLog {
            args: vec!["This should fail.".to_string()],
        },
    );

    let result = send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &user],
        ix,
        context.recent_blockhash,
    ).await.map_err(Into::<TransportError>::into);

    assert_custom_error(result, ErrorCode::Unauthorized, "User without emit_log permission should be blocked");

    Ok(())
}


#[tokio::test]
async fn test_emit_log_user_dont_exist_should_fail() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let user = Keypair::new(); // Nunca recebe nenhuma permissão, nem sequer cria user_access
    context.mint_tokens( &owner, 1_000_000).await;

    context.refresh().await;

    let seeds = derive_seeds(&context.program_id, &user.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::EmitSystemLog {
            caller: user.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_token::instruction::EmitSystemLog {
            args: vec!["This should fail – no user_access.".to_string()],
        },
    );

    let result = send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &user],
        ix,
        context.recent_blockhash,
    )
    .await
    .map_err(Into::<TransportError>::into);

    assert_custom_error(result, ErrorCode::Unauthorized, "User without user_access should fail to emit log");

    Ok(())
}

#[tokio::test]
async fn test_emit_log_with_admin_flag_should_succeed() -> Result<(), TransportError> {
    // Initialize the full test environment
    let (mut context, owner) = setup_test_env().await;

    // Create an admin user (will only have the admin flag, no permissions)
    let admin = Keypair::new();
    context.mint_tokens( &owner, 1_000_000).await;

    // Grant the admin flag (no permissions assigned)
    try_add_admin(&mut context, &owner, &admin.pubkey()).await.unwrap();
    context.refresh().await;

    // Derive user_access and token_state accounts
    let admin_seeds = derive_seeds(&context.program_id, &admin.pubkey());

    // Attempt to emit a system log as an admin
    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::EmitSystemLog {
            caller: admin.pubkey(),
            user_access: Some(admin_seeds.user_access),
            token_state: admin_seeds.token_state,
        },
        soccial_token::instruction::EmitSystemLog {
            args: vec!["Admin executed log.".to_string()],
        },
    );

    // Should succeed because admin has implicit permissions
    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &admin],
        ix,
        context.recent_blockhash,
    )
    .await?;

    Ok(())
}

#[tokio::test]
async fn test_emit_log_with_owner_should_succeed() -> Result<(), TransportError> {
    // Initialize the test environment
    let (mut context, owner) = setup_test_env().await;

    // The owner should already have user_access account set by initialize_token_env
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());

    // Try to emit a log directly as the contract owner
    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::EmitSystemLog {
            caller: owner.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
        },
        soccial_token::instruction::EmitSystemLog {
            args: vec!["Owner emitted this.".to_string()],
        },
    );
    
    // Owner should be allowed even without explicit permissions
    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &owner],
        ix,
        context.recent_blockhash,
    )
    .await?;

    Ok(())
}

