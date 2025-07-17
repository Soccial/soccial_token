use anchor_lang::InstructionData;
use solana_program_test::*;
use solana_sdk::{
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_program,
    transport::TransportError,
};

use soccial_token::{self, instruction as soccial_instruction};
mod testutils;
use crate::testutils::basics::*;

#[tokio::test]
async fn test_initialize_spl_token_should_succeed() -> Result<(), TransportError> {

    let program_id = soccial_token::ID;
    

    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(&mut program_test, &[&payer, &caller], 10_000_000_000, system_program::ID);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let seeds = derive_seeds(&program_id, &caller.pubkey());

     // Step 1: Initialize token
     let ix_initialize = Instruction {
        program_id,
        accounts: build_initialize_token_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeToken {}
        .data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix_initialize, recent_blockhash)
        .await
        .expect("❌ Failed to initialize token");

    // Step 2: Init Economy
    let ix_economy = Instruction {
        program_id,
        accounts: build_initialize_economy_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeEconomy {}.data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix_economy, recent_blockhash)
        .await
        .expect("❌ Failed to initialize economy");



    // 🧪 Step: Initialize SPL Token
    let ix = Instruction {
        program_id,
        accounts: build_initialize_spl_token_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeSplToken {}
        .data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix, recent_blockhash)
        .await
        .expect("❌ Failed to initialize SPL token");

    
    Ok(())
}


#[tokio::test]
async fn test_initialize_spl_token_twice_should_fail() -> Result<(), TransportError> {

    let program_id = soccial_token::ID;
    

    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(&mut program_test, &[&payer, &caller], 10_000_000_000, system_program::ID);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let seeds = derive_seeds(&program_id, &caller.pubkey());

     // Step 1: Initialize token
     let ix_initialize = Instruction {
        program_id,
        accounts: build_initialize_token_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeToken {}
        .data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix_initialize, recent_blockhash)
        .await
        .expect("❌ Failed to initialize token");

    // Step 2: Init Economy
    let ix_economy = Instruction {
        program_id,
        accounts: build_initialize_economy_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeEconomy {}.data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix_economy, recent_blockhash)
        .await
        .expect("❌ Failed to initialize economy");

        
    // Step 3: First Initialize SPL Token (should succeed)
  
    let ix_spl_token = Instruction {
        program_id,
        accounts: build_initialize_spl_token_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeSplToken {}
        .data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &caller], ix_spl_token.clone(), recent_blockhash)
        .await
        .expect("❌ First SPL token init failed");

    let new_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    // Step 4: Second Initialize SPL Token (should fail)
    send_ix_expect_failure(&mut banks_client, &payer, &[&payer, &caller], ix_spl_token, new_blockhash)
        .await?;

    
    Ok(())
}


#[tokio::test]
async fn test_initialize_spl_token_without_initialize_should_fail() -> Result<(), TransportError> {

    let program_id = soccial_token::ID;
    

    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(&mut program_test, &[&payer, &caller], 10_000_000_000, system_program::ID);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let seeds = derive_seeds(&program_id, &caller.pubkey());

    // Skip init_token + init_economy and go directly to SPL Token
    let ix_spl_token = Instruction {
        program_id,
        accounts: build_initialize_spl_token_accounts(&seeds, caller.pubkey()),
        data: soccial_instruction::InitializeSplToken {}
        .data(),
    };

    send_ix_expect_failure(&mut banks_client, &payer, &[&payer, &caller], ix_spl_token, recent_blockhash)
        .await?;

    
    Ok(())
}

#[tokio::test]
async fn test_initialize_spl_token_without_permission_should_fail() -> Result<(), TransportError> {
  
    let program_id = soccial_token::ID;
    

    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let owner = Keypair::new();    // legítimo
    let intruder = Keypair::new(); // não autorizado

    fund_test_accounts(&mut program_test, &[&payer, &owner, &intruder], 10_000_000_000, system_program::ID);
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let seeds = derive_seeds(&program_id, &owner.pubkey());

    // Step 1: Initialize by owner
    let ix_initialize = Instruction {
        program_id,
        accounts: build_initialize_token_accounts(&seeds, owner.pubkey()),
        data: soccial_instruction::InitializeToken {}
        .data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &owner], ix_initialize, recent_blockhash)
        .await
        .expect("❌ Failed to initialize token");

    let ix_economy = Instruction {
        program_id,
        accounts: build_initialize_economy_accounts(&seeds, owner.pubkey()),
        data: soccial_instruction::InitializeEconomy {}.data(),
    };

    send_ix(&mut banks_client, &payer, &[&payer, &owner], ix_economy, recent_blockhash)
        .await
        .expect("❌ Failed to initialize economy");

    // Step 2: Intruder tries to initialize SPL token
    let ix_spl_token = Instruction {
        program_id,
        accounts: build_initialize_spl_token_accounts(&seeds, intruder.pubkey()),
        data: soccial_instruction::InitializeSplToken {}
        .data(),
    };

    let new_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    send_ix_expect_failure(&mut banks_client, &payer, &[&payer, &intruder], ix_spl_token, new_blockhash)
        .await?;

    
    Ok(())
}
