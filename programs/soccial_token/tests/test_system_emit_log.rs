// ======================================================================
// Tests: Emit System Log
// ======================================================================

use solana_program_test::*;
use solana_sdk::{signature::Keypair, transport::TransportError};
use testutils::environment::*;
use testutils::basics::assert_custom_error;
use soccial_token::utils::error::ErrorCode;

mod testutils;
mod trymethods;
use crate::trymethods::trysystem::*;


#[tokio::test]
async fn test_emit_system_log_success() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let args = vec!["System upgrade complete".to_string()];

    let result = try_emit_system_log(&mut context, &owner, args).await;

    assert!(result.is_ok(), "Expected successful log emission");

    Ok(())
}

#[tokio::test]
async fn test_emit_system_log_missing_arg() -> Result<(), TransportError> {
    let (mut context, owner) = setup_test_env().await;

    let args = vec![]; // missing message

    let result = try_emit_system_log(&mut context, &owner, args).await;

    assert_custom_error(result, ErrorCode::NotEnoughArguments, "Expected MissingArgs");

    Ok(())
}

#[tokio::test]
async fn test_emit_system_log_unauthorized_user() -> Result<(), TransportError> {
    let (mut context, _owner) = setup_test_env().await;

    let intruder = Keypair::new();
    create_user_ata(&mut context, &intruder).await?;
    fund_lamports(&mut context, &intruder, 5_000_000).await?;

    let args = vec!["Hacked log".to_string()];

    let result = try_emit_system_log(&mut context, &intruder, args).await;

    assert_custom_error(result, ErrorCode::Unauthorized, "Expected Unauthorized error");

    Ok(())
}
