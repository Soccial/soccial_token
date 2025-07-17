use solana_program_test::*;
use solana_sdk::msg;
use solana_sdk::transport::TransportError;

mod testutils;
mod trymethods;
use crate::testutils::environment::*;
use crate::testutils::environment::setup_test_env;

#[tokio::test]
async fn test_balance() -> Result<(), TransportError> {
  
    let (mut context, caller) = setup_test_env().await;

    msg!("✅ Rewards vault deposit successful.");
    
    log_all_balances(&mut context, &caller).await?;

    Ok(())
}