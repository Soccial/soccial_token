
use anchor_lang::system_program;
use soccial_token::utils::error::ErrorCode;
use solana_program_test::*;
use solana_sdk::signer::Signer;
use solana_sdk::{signature::Keypair, transport::TransportError};
use testutils::environment::*;
use soccial_token::{instruction as soccial_instruction};
mod testutils;
mod trymethods;
use crate::testutils::basics::{anchor_ix, assert_custom_error, derive_seeds, send_ix};
use crate::trymethods::trysystem::*;


#[tokio::test]
async fn test_permissions_api_authority_should_succeed() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    
    let api = Keypair::new();

    let new_api_authority = api.pubkey();

    let seeds = derive_seeds(&context.program_id, &admin.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::ManageContract {
            caller: admin.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::SetApiAuthority {
            args: vec![new_api_authority.to_string()],
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &admin],
        ix,
        context.recent_blockhash,
    ).await?;


    let args = vec!["System upgrade complete".to_string()];

    let result = try_emit_system_log(&mut context, &api, args).await;

    assert!(result.is_ok(), "Expected successful log emission");

    Ok(())
}


#[tokio::test]
async fn test_permissions_api_authority_with_intruder_should_fail() -> Result<(), TransportError> {
    let (mut context, admin) = setup_test_env().await;
    
    let api = Keypair::new();
    let intruder = Keypair::new();

    let new_api_authority = api.pubkey();

    let seeds = derive_seeds(&context.program_id, &admin.pubkey());

    let ix = anchor_ix(
        context.program_id,
        soccial_token::accounts::ManageContract {
            caller: admin.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::SetApiAuthority {
            args: vec![new_api_authority.to_string()],
        },
    );

    send_ix(
        &mut context.banks_client,
        &context.payer,
        &[&context.payer, &admin],
        ix,
        context.recent_blockhash,
    ).await?;


    let args = vec!["System upgrade complete".to_string()];

    // Try with an intruder. Don't have permission and is not an API Authority
    let result = try_emit_system_log(&mut context, &intruder, args).await;

    assert_custom_error(
        result,
        ErrorCode::Unauthorized,
        "Expected unsuccessful log emission due to permissions.",
    );


    Ok(())
}