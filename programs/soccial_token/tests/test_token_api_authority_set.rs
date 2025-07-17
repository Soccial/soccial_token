// ======================================================================
// Test file: test_set_api_authority.rs
// ======================================================================


use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport::TransportError,
    system_program,
};

use soccial_token::{
    self,
    instruction as soccial_instruction, token::TokenState,
};

mod testutils;
use crate::testutils::basics::{
    anchor_ix, derive_seeds, fund_test_accounts,
    initialize_token_env, send_ix, send_ix_expect_failure,
};

#[tokio::test]
async fn test_set_api_authority_should_succeed() -> Result<(), TransportError> {
  

    let program_id = soccial_token::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new(); // Owner

    fund_test_accounts(&mut program_test, &[&payer, &caller], 10_000_000_000, system_program::ID);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    initialize_token_env(program_id, &mut banks_client, &payer, &caller, recent_blockhash).await?;

    let seeds = derive_seeds(&program_id, &caller.pubkey());

    let new_api_authority = Keypair::new().pubkey();

    let ix = anchor_ix(
        program_id,
        soccial_token::accounts::ManageContract {
            caller: caller.pubkey(),
            user_access: None,
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::SetApiAuthority {
            args: vec![new_api_authority.to_string()],
        },
    );

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix, recent_blockhash).await?;

    let settings_account = banks_client
        .get_account(seeds.token_state)
        .await?
        .expect("❌ Contract Settings account should exist");

    let control: TokenState = anchor_lang::AccountDeserialize::try_deserialize(&mut &settings_account.data[..])
        .expect("❌ Failed to deserialize Contract Settings");

    assert_eq!(
        control.core.api_authority,
        new_api_authority,
        "🚨 API authority should match the new public key"
    );

    
    Ok(())
}

#[tokio::test]
async fn test_set_api_authority_unauthorized() -> Result<(), TransportError> {
  
    let program_id = soccial_token::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let owner = Keypair::new();
    let non_admin = Keypair::new();

    fund_test_accounts(
        &mut program_test,
        &[&payer, &owner, &non_admin],
        10_000_000_000,
        system_program::ID,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    initialize_token_env(program_id, &mut banks_client, &payer, &owner, recent_blockhash).await?;

    let seeds = derive_seeds(&program_id, &non_admin.pubkey());

    let new_api_authority = Keypair::new().pubkey();

    let ix = anchor_ix(
        program_id,
        soccial_token::accounts::ManageContract {
            caller: non_admin.pubkey(),
            user_access: None,
            
            token_state: seeds.token_state,
            system_program: system_program::ID,
        },
        soccial_instruction::SetApiAuthority {
            args: vec![new_api_authority.to_string()],
        },
    );

    send_ix_expect_failure(&mut banks_client, &payer, &[&payer, &non_admin], ix, recent_blockhash).await?;

    
    Ok(())
}
