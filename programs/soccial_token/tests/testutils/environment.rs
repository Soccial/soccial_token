// ============================================================================
// Soccial Token – Integration Test Environment Extensions
// ----------------------------------------------------------------------------
//
// This module provides extended test utilities and helpers for Solana
// `ProgramTest`, tailored for the Soccial Token smart contract.
//
// It wraps a full test runtime (`EnvProgramTestContext`) and offers high-level
// features for:
// - Time warping (slots & seconds)
// - Token minting (users, vaults, fallback)
// - Balance checks & accounting audits
// - Utility wrappers for PDAs, ATAs, and transfers
//
// ----------------------------------------------------------------------------
// Features:
// ✔ Warp forward by slots or seconds (and patch Clock sysvar)
// ✔ Mint tokens to any user, vault, or internal fallback account
// ✔ Transfer tokens and SOL dynamically between entities
// ✔ Assert account existence and fetch SPL balances
// ✔ Log system-wide vault and token supply audit reports
//
// ----------------------------------------------------------------------------
// Key Components:
// - `EnvProgramTestContext`: Wrapper for ProgramTestContext with helpers
// - `setup_program_test`: Bootstraps the full on-chain state
// - `warp_forward_slots` / `warp_forward_seconds`: Time manipulation
// - `mint_tokens_*`: Direct minting helpers for tests
// - `log_all_balances`: Snapshot report of all token activity
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================


use anchor_lang::{solana_program, AccountDeserialize};
use soccial_token::{self, economy::*, governance::GovernanceState, utils::math::units_to_tokens, token::state::TokenState};
use solana_sdk::{program_pack::Pack, system_instruction, transaction::Transaction};
use spl_token::state::Account as SplAccount;
use solana_program_test::{BanksClient, ProgramTest, ProgramTestContext};

use solana_sdk::{
    account::ReadableAccount, msg, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transport::TransportError, hash::Hash,
    account::Account, account::AccountSharedData
};
use chrono::{Duration, TimeZone, Utc};
use spl_associated_token_account::get_associated_token_address;

use crate::testutils::basics::*;
//use thousands::Separable;
use solana_program::sysvar::clock::ID as CLOCK_SYSVAR_ID;
use solana_program::clock::Clock;
use bincode::{serialize, deserialize};
use core::convert::TryFrom;

/// Struct to hold the test context
#[allow(dead_code)]
pub struct EnvProgramTestContext {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub caller: Keypair,
    pub recent_blockhash: Hash,
    pub program_id: Pubkey,
    pub slot: u64,
    pub slot_offset: u64,
    pub original_context: ProgramTestContext,
}

/// Returns the vault PDA and its associated token account based on a given name.
///
/// # Arguments
/// * `vault_name` - Name of the vault (e.g., "liquidity", "airdrop", etc.).
/// * `seeds` - Struct containing all pre-derived PDAs and ATAs.
///
/// # Panics
/// Panics if the vault name is not recognized.
pub fn get_vault_accounts_by_name(
    vault_name: &str,
    seeds: &AccountSeeds,
) -> (Pubkey, Pubkey) {
    match vault_name {
        "offchain_reserve" => (
            seeds.offchain_reserve_vault,
            seeds.offchain_reserve_vault_token_account,
        ),
        "liquidity" => (
            seeds.liquidity_vault,
            seeds.liquidity_vault_token_account,
        ),
        "staking" => (
            seeds.staking_vault,
            seeds.staking_vault_token_account,
        ),
        "revenue" => (
            seeds.revenue_vault,
            seeds.revenue_vault_token_account,
        ),
        "rewards" => (
            seeds.rewards_vault,
            seeds.rewards_vault_token_account,
        ),
        "airdrop" => (
            seeds.airdrop_vault,
            seeds.airdrop_vault_token_account,
        ),
        "vesting" => (
            seeds.vesting_vault,
            seeds.vesting_vault_token_account,
        ),
        "insurance" => (
            seeds.insurance_vault,
            seeds.insurance_vault_token_account,
        ),
        "treasury" => (
            seeds.treasury_vault,
            seeds.treasury_vault_token_account,
        ),
        "reserved_supply" => (
            seeds.reserved_supply_vault,
            seeds.reserved_supply_vault_token_account,
        ),
        _ => panic!("❌ Unknown vault name: {}", vault_name),
    }
}


#[allow(dead_code)]
impl EnvProgramTestContext {

    /// Refreshes the `recent_blockhash` inside the test context.
    ///
    /// In Solana, transactions must include a recent blockhash to be accepted by the network.
    /// When running tests, if the blockhash becomes too old or if multiple transactions are
    /// processed, it's important to refresh it to avoid transaction replay errors or expired blockhash issues.
    ///
    /// This method updates `self.recent_blockhash` by fetching the latest one from the `BanksClient`.
    ///
    /// # Panics
    /// This function will panic if it fails to fetch the latest blockhash from the `BanksClient`.
    ///
    /// # Example
    
    /// context.refresh().await;
    
    pub async fn refresh(&mut self) {
        self.recent_blockhash = self
            .banks_client
            .get_latest_blockhash()
            .await
            .expect("Failed to refresh blockhash");
    }   
    
    pub async fn get_current_unix_timestamp(&mut self) -> i64 {
        let clock_sysvar = solana_program::sysvar::clock::id();
        let clock_account = self.original_context.banks_client.get_account(clock_sysvar)
            .await.unwrap().unwrap();
        let clock: solana_program::sysvar::clock::Clock =
            bincode::deserialize(&clock_account.data).unwrap();
        clock.unix_timestamp
    }

    /// Loads and deserializes the TokenState from the on-chain PDA
    pub async fn load_token_state(&mut self) -> TokenState {
        let seeds = derive_seeds(&self.program_id, &self.payer.pubkey());

        let account = self
            .banks_client
            .get_account(seeds.token_state)
            .await
            .expect("Failed to fetch token state account")
            .expect("Token state account not found");

        TokenState::try_deserialize(&mut account.data.as_slice())
            .expect("Failed to deserialize TokenState")
    }
    

    /// Warps the blockchain forward by the specified number of slots **and** updates the Clock sysvar
    /// with an estimated timestamp based on the standard 400ms slot duration.
    ///
    /// This ensures tests depending on `unix_timestamp` behave as expected after warp.
    ///
    /// # Arguments
    /// * `slots` - The number of slots to warp forward.
    ///
    /// # Panics
    /// Will panic if the warp fails or if the Clock cannot be injected.
    pub async fn warp_forward_slots(&mut self, slots: u64) {
        // Step 1: get original clock
        let clock_before = self.get_reliable_clock().await;
        let current_slot = clock_before.slot;
        let current_ts = clock_before.unix_timestamp;

        // Step 2: calculate new slot and timestamp
        let target_slot = current_slot + slots;
        let estimated_ts = current_ts + ((slots as f64) * 0.4) as i64;

        // Step 3: warp to new slot
        self.original_context
            .warp_to_slot(target_slot)
            .expect("Failed to warp to slot");

        self.slot = target_slot;
        self.slot_offset += slots;

        // Step 4: inject custom clock with estimated timestamp
        let fake_clock = Clock {
            slot: target_slot,
            epoch_start_timestamp: estimated_ts - 10000, // Arbitrary, can refine later
            unix_timestamp: estimated_ts,
            epoch: clock_before.epoch,
            leader_schedule_epoch: clock_before.leader_schedule_epoch,
        };

        let clock_data = serialize(&fake_clock).unwrap();
        let fake_clock_account = Account {
            lamports: 1,
            data: clock_data,
            owner: CLOCK_SYSVAR_ID,
            executable: false,
            rent_epoch: 0,
        };

        let shared_account = AccountSharedData::from(fake_clock_account);
        self.original_context
            .set_account(&CLOCK_SYSVAR_ID, &shared_account);

        // Step 5: force Solana runtime to re-read clock
        let dummy_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[solana_sdk::system_instruction::transfer(
                &self.payer.pubkey(),
                &self.payer.pubkey(),
                1,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            self.recent_blockhash,
        );

        self.banks_client
            .process_transaction(dummy_tx)
            .await
            .expect("Failed to force transaction for clock update");

        self.refresh().await;
    }

    /// Returns the current runtime `Clock`
    pub async fn get_reliable_clock(&mut self) -> solana_program::clock::Clock {
        
        // read the account and update the Clock
        let clock_account = self
            .banks_client
            .get_account(CLOCK_SYSVAR_ID)
            .await
            .expect("clock sysvar account not found")
            .expect("clock sysvar is none");

        deserialize(&clock_account.data).expect("Failed to deserialize Clock")
    }


    /// Warps the blockchain forward by a specified number of **seconds**.
    ///
    /// This is useful for simulating the passage of real-world time in Solana tests,
    /// especially when testing vesting schedules, time-based releases, expirations, etc.
    ///
    /// Internally, this converts seconds to slots assuming an average slot duration of ~400 ms,
    /// and calls `warp_forward_slots` accordingly.
    ///
    /// # Arguments
    /// * `seconds` - The number of seconds to simulate forward.
    ///
    /// # Panics
    /// Panics if slot warping fails.
    ///
    /// # Example
    
    /// context.warp_forward_seconds(180).await; // Warp 3 minutes into the future
    
    pub async fn warp_forward_seconds(&mut self, seconds: u64) {
        let slots = ((seconds as f64) / 0.4).ceil() as u64;
        self.warp_forward_slots(slots).await;
    }

    /// Estimates the UTC datetime corresponding to a given slot,
    /// and returns it as a formatted string (`"%Y-%m-%d %H:%M:%S"`).
    ///
    /// # Arguments
    /// * `initial_slot` - The starting slot used as reference.
    /// * `slot` - The target slot to estimate time for.
    ///
    /// # Returns
    /// * `String` — Formatted datetime string.
    pub async fn estimate_datetime_from_slot_formatted(&mut self, target_slot: u64) -> String {
        let base_slot = self.slot;
        let base_ts = self.get_current_unix_timestamp().await;

        let slot_diff = target_slot.saturating_sub(base_slot);
        let delta_millis = (slot_diff as f64 * 400.0).round() as i64;

        let base_datetime = Utc
            .timestamp_opt(base_ts, 0)
            .single()
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap()); 

        let estimated = base_datetime
            .checked_add_signed(Duration::milliseconds(delta_millis))
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap()); 

        estimated.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Fetches the SPL token balance of the user's associated token account
    ///
    /// # Arguments
    /// * `user` – The user's Pubkey
    ///
    /// # Returns
    /// The raw token balance (base units)
    #[allow(dead_code)]
    pub async fn get_user_balance(
        &mut self,
        user: &Pubkey,
    ) -> u64 {
        let ata = get_associated_token_address(user, &derive_seeds(&self.program_id, user).token_mint);
        fetch_token_balance(&mut self.banks_client, &ata).await
    }

    #[allow(dead_code)]
    /// Fetches the SPL token balance of a vault's token account
    ///
    /// # Arguments
    /// * `vault_name` – The name of the vault (e.g., "liquidity", "airdrop", etc.)
    ///
    /// # Returns
    /// The raw token balance (in base units)
    pub async fn get_vault_balance(
        &mut self,
        vault_name: &str,
    ) -> u64 {
        let seeds = derive_seeds(&self.program_id, &self.payer.pubkey());
        let (_vault_pda, vault_token_account) = get_vault_accounts_by_name(vault_name, &seeds);

        fetch_token_balance(&mut self.banks_client, &vault_token_account).await
    }

    #[allow(dead_code)]
    pub async fn transfer_tokens_to_user(
        &mut self,
        sender: &Keypair,
        recipient: &Pubkey,
        amount: u64,
    ) -> Result<(), TransportError> {
        let seeds = derive_seeds(&self.program_id, &sender.pubkey());

        let source_token_account = seeds.user_token_ata;
        let destination_token_account = get_associated_token_address(recipient, &seeds.token_mint);

        create_ata_if_missing(
            &mut self.banks_client,
            &self.payer,
            sender,
            &seeds.token_mint,
            &destination_token_account,
            self.recent_blockhash,
        ).await?;

        let ix = spl_token::instruction::transfer(
            &spl_token::ID,
            &source_token_account,
            &destination_token_account,
            &sender.pubkey(),
            &[],
            amount,
        ).map_err(|e| TransportError::Custom(format!("Token transfer instruction failed: {:?}", e)))?;

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.payer.pubkey()),
            &[&self.payer, sender],
            self.recent_blockhash,
        );

        self.banks_client.process_transaction(tx).await?;

        Ok(())
    }

    /// Mints tokens directly using the SPL Token CPI (bypassing the main program logic).
    ///
    /// This function is intended for test scenarios only and allows minting tokens
    /// directly to a user's associated token account (ATA), without going through the Soccial program.
    ///
    /// # Arguments
    /// * `recipient` - The public key of the user receiving the minted tokens.
    /// * `amount` - Amount of tokens (in base units) to mint.
    ///
    /// # Panics
    /// Panics if the mint operation fails.
    ///
    /// # Example
    
    /// let recipient = Keypair::new();
    /// context.create_user_ata(&recipient).await?;
    /// context.mint_tokens_to_user(&recipient.pubkey(), 1_000_000).await;
    
    #[allow(dead_code)]
    pub async fn mint_tokens_to_user(
        &mut self, 
        recipient: &Pubkey, 
        amount: u64
    ) {
        let seeds = derive_seeds(&self.program_id, recipient);
        let ix = anchor_ix(
            self.program_id,
            soccial_token::accounts::MintTokens {
                caller: self.caller.pubkey(),
                mint_authority: seeds.mint_authority,
                mint: seeds.token_mint,
                recipient: *recipient,
                recipient_token_account: seeds.user_token_ata,
                token_program: spl_token::ID,
                associated_token_program: spl_associated_token_account::ID,
                system_program: system_program::ID,
            },
            soccial_token::instruction::Mint {
                args: vec![amount.to_string()],
            },
        );
        
        send_ix(
            &mut self.banks_client,
            &self.payer,
              &[&self.payer, &self.caller],
            ix,
            self.recent_blockhash,
        )

        .await
        .expect("Failed to mint tokens to user");
    }

    #[allow(dead_code)]
    /// Mints tokens directly to a vault's associated token account (ATA) using SPL Token CPI.
    ///
    /// This is a testing utility that allows minting tokens directly to a vault (like "liquidity", "airdrop", etc.)
    /// bypassing all program-level logic.
    ///
    /// # Arguments
    /// * `vault_name` - The name of the vault ("liquidity", "airdrop", "staking", etc.).
    /// * `amount` - The number of tokens to mint (in base units).
    ///
    /// # Example
    
    /// context.mint_tokens_to_vault("liquidity", 100_000_000).await?;
    
    pub async fn mint_tokens_to_vault(
        &mut self,
        vault_name: &str,
        amount: u64,
    ) -> Result<(), TransportError> {
        let seeds = derive_seeds(&self.program_id, &self.caller.pubkey());

        // Use helper to resolve vault PDA and associated token account
        let (vault_pda, vault_token_account) = get_vault_accounts_by_name(vault_name, &seeds);

        let ix = anchor_ix(
            self.program_id,
            soccial_token::accounts::MintTokens {
                caller: self.caller.pubkey(),
                mint_authority: seeds.mint_authority,
                mint: seeds.token_mint,
                recipient: vault_pda,
                recipient_token_account: vault_token_account,
                token_program: spl_token::ID,
                associated_token_program: spl_associated_token_account::ID,
                system_program: system_program::ID,
            },
            soccial_token::instruction::Mint {
                args: vec![amount.to_string()],
            },
        );

        send_ix(
            &mut self.banks_client,
            &self.payer,
            &[&self.payer, &self.caller],
            ix,
            self.recent_blockhash,
        )
        .await?;

        Ok(())
    }


    /// Mints tokens directly to the contract's internal token account.
    ///
    /// This is a testing-only utility that allows minting tokens to the program-owned fallback account
    /// (`contract_token_owner`) used for recovery or internal operations. It bypasses all on-chain logic.
    ///
    /// The recipient of the mint is the `contract_token_owner` PDA, and the destination is its
    /// associated token account (ATA).
    ///
    /// # Arguments
    /// * `amount` - The number of tokens to mint (in base units, e.g., 1_000_000 for 1 token if decimals = 6)
    ///
    /// # Example
    
    /// context.mint_tokens_to_contract_account(1_000_000).await?;
    
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(TransportError)` if the transaction fails
   #[allow(dead_code)]
    pub async fn mint_tokens_to_contract_account(
        &mut self,
        amount: u64,
    ) -> Result<(), TransportError> {
        let seeds = derive_seeds(&self.program_id, &self.caller.pubkey());

        let mint = seeds.token_mint;
        let mint_authority = seeds.mint_authority;
        let contract_token_account = seeds.contract_token_account;
        let contract_token_owner = seeds.contract_token_owner;

        let ix = anchor_ix(
            self.program_id,
            soccial_token::accounts::MintTokens {
                caller: self.caller.pubkey(),
                mint_authority,
                mint,
                recipient: contract_token_owner,
                recipient_token_account: contract_token_account,
                token_program: spl_token::ID,
                associated_token_program: spl_associated_token_account::ID,
                system_program: system_program::ID,
            },
            soccial_token::instruction::Mint {
                args: vec![amount.to_string()],
            },
        );

        send_ix(
            &mut self.banks_client,
            &self.payer,
            &[&self.payer, &self.caller],
            ix,
            self.recent_blockhash,
        )
        .await
        .map_err(|e| TransportError::Custom(e.to_string()))
        
    }



    /// Wrapper for `mint_tokens_to_user` that accepts a `Keypair` directly.
    ///
    /// Allows calling `context.mint_tokens(&user, amount)` instead of needing to extract the pubkey manually.
    ///
    /// # Arguments
    /// * `user` - The Keypair representing the recipient.
    /// * `amount` - Amount of tokens (base units) to mint.
    ///
    /// # Example
    
    /// let user = Keypair::new();
    /// context.create_user_ata(&user).await?;
    /// context.mint_tokens(&user, 1_000_000).await;
    
    #[allow(dead_code)]
    pub async fn mint_tokens(&mut self, user: &Keypair, amount: u64) {
        self.mint_tokens_to_user(&user.pubkey(), amount).await;
    }

}

/// Sets up a generic `ProgramTest` environment for Solana integration tests.
///
/// This function creates a basic testing environment with a payer and a caller,
/// funds them with lamports, starts the test environment, and initializes the
/// Soccial Token environment by calling the `initialize_token_env` function.
///
/// # Arguments
/// * `program_name` - The name of the program under test.
/// * `program_id` - The program's public key.
///
/// # Returns
/// * `Ok((EnvProgramTestContext, Keypair))` - A tuple containing the test context and the authorized caller.
/// * `Err(TransportError)` - If environment setup or initialization fails.
///
/// # Example

/// let (context, owner) = setup_program_test("soccial_token", soccial_token::ID).await.unwrap();

///uarn test
/// # Panics
/// This function does not panic, but if any transaction fails during setup, it will return an error.
/// Sets up the Solana ProgramTest environment with a test mint.
#[allow(dead_code)]
pub async fn setup_program_test(
    program_name: &'static str,
    program_id: Pubkey,
) -> Result<(EnvProgramTestContext, Keypair), TransportError> {
    let mut program_test = ProgramTest::default();
    program_test.add_program(program_name, program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(&mut program_test, &[&payer, &caller], 10_000_000_000, system_program::ID);

    let program_context = program_test.start_with_context().await;

    let current_slot = program_context.banks_client.get_root_slot().await?;

    let context = EnvProgramTestContext {
        banks_client: program_context.banks_client.clone(),
        payer: clone_keypair(&program_context.payer),
        caller: clone_keypair(&caller),

        recent_blockhash: program_context.last_blockhash,
        program_id,
        original_context: program_context,
        slot: current_slot,
        slot_offset: 0,
    };

    initialize_token_env(
        context.program_id,
        &mut context.banks_client.clone(),
        &context.payer,
        &caller,
        context.recent_blockhash,
    )
    .await?;

    Ok((context, caller))
}

/// Clones a `Keypair` by serializing it to bytes and reconstructing it safely.
///
/// This is useful in testing contexts where `Keypair::clone()` is not available or
/// when you need an owned copy of a keypair derived from an existing one.
///
/// Internally, it uses `to_bytes()` (which returns `[u8; 64]`) and converts it
/// to a slice (`&[u8]`) to match the expected input of `Keypair::try_from(...)`.
///
/// Panics with a clear error message if the byte conversion fails, which should
/// never happen unless the keypair was corrupted.
fn clone_keypair(original: &Keypair) -> Keypair {
    Keypair::try_from(original.to_bytes().as_ref())
        .expect("Invalid keypair bytes")
}


/// Shortcut for quickly setting up the Soccial Token test environment.
///
/// This function wraps `setup_program_test` with pre-filled program name and ID
/// for the Soccial Token project, reducing boilerplate in tests.
///
/// # Returns
/// * `(EnvProgramTestContext, Keypair)` - The test environment context and the authorized caller.
///
/// # Example

/// let (context, owner) = setup_test_env().await;

#[allow(dead_code)]
pub async fn setup_test_env() -> (EnvProgramTestContext, Keypair) {
    setup_program_test("soccial_token", soccial_token::ID).await.unwrap()
}

/// Transfers lamports (SOL) from the test payer to a recipient account.
///
/// This is used to ensure that a given test account has enough SOL to pay for transaction fees.
///
/// # Arguments
/// * `context` - The current ProgramTest environment.
/// * `recipient` - The Keypair of the account to receive the lamports.
/// * `amount` - The amount of lamports to transfer.
///
/// # Returns
/// * `Ok(())` on success.
/// * `Err(TransportError)` if the transaction fails.
///
/// # Example

/// fund_lamports(&mut context, &user, 10_000_000).await?;

#[allow(dead_code)]
pub async fn fund_lamports(
    context: &mut EnvProgramTestContext,
    recipient: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let transfer_ix = solana_sdk::system_instruction::transfer(
        &context.payer.pubkey(),
        &recipient.pubkey(),
        amount,
    );

    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.recent_blockhash,
    );

    context.banks_client.process_transaction(tx).await?;
    Ok(())
}

/// Creates an Associated Token Account (ATA) for an intruder account.
///
/// This ensures that the intruder has a valid token account for a given SPL mint,
/// which can then be used in tests involving token transfers or interactions.
///
/// # Arguments
/// * `context` - The current ProgramTest environment.
/// * `intruder` - The Keypair representing the intruder.
/// * `mint` - The Pubkey of the token mint.
///
/// # Returns
/// * `Ok(())` if the ATA was created or already exists.
/// * `Err(TransportError)` if the ATA creation fails.
///
/// # Example

/// create_intruder_token_account(&mut context, &intruder, &mint).await?;

#[allow(dead_code)]
pub async fn create_intruder_token_account(
    context: &mut EnvProgramTestContext,
    intruder: &Keypair,
    mint: &Pubkey,
) -> Result<(), TransportError> {
    let ata = get_associated_token_address(&intruder.pubkey(), mint);

    create_ata_if_missing(
        &mut context.banks_client,
        &context.payer,
        intruder,
        mint,
        &ata,
        context.recent_blockhash,
    ).await
}

/// Creates the intruder's Associated Token Account (ATA) and funds their account with lamports.
///
/// This function ensures that the given intruder account has both:
/// - A valid Associated Token Account (for token operations).
/// - Enough SOL (lamports) to cover transaction fees during tests.
///
/// # Arguments
/// * `context` - The current test environment.
/// * `intruder` - The Keypair representing the unauthorized user to fund.
///
/// # Returns
/// * `Ok(())` if successful.
/// * `Err(TransportError)` if any step fails.
///
/// # Example

/// let intruder = Keypair::new();
/// create_and_fund_intruder(&mut context, &intruder).await?;

#[allow(dead_code)]
pub async fn create_and_fund_intruder(
    context: &mut EnvProgramTestContext,
    intruder: &Keypair,
) -> Result<(), TransportError> {
    create_user_ata(context, intruder).await?;
    fund_lamports(context, intruder, 10_000_000).await?;
    Ok(())
}

/// Ensures that the user's Associated Token Account (ATA) for the main token mint exists.
/// Creates it if missing and returns the ATA address.
///
/// # Arguments
/// * `context` - The current test environment.
/// * `owner` - The account that will own the created ATA.
///
/// # Returns
/// * `Ok(Pubkey)` - The address of the user's ATA.
/// * `Err(TransportError)` - If the creation fails.
#[allow(dead_code)]
pub async fn create_user_ata(
    context: &mut EnvProgramTestContext,
    owner: &Keypair,
) -> Result<Pubkey, TransportError> {
    // Derive the seeds based on the owner's public key
    let seeds = derive_seeds(&context.program_id, &owner.pubkey());

    // Mint comes from the seeds
    let mint = seeds.token_mint;

    // Derive the Associated Token Account (ATA) for the owner
    let user_ata = get_associated_token_address(&owner.pubkey(), &mint);

    // Ensure the ATA exists
    create_ata_if_missing(
        &mut context.banks_client,
        &context.payer,
        owner,
        &mint,
        &user_ata,
        context.recent_blockhash,
    ).await?;

    Ok(user_ata)
}

/// ======================================================================
/// Manually creates a Token Account with a custom owner.
/// Unlike create_user_ata (which uses the Associated Token Program),
/// this method allows for more flexibility, including the creation of
/// intentionally invalid or non-standard accounts for edge case testing,
/// such as simulating transfers to unauthorized vaults.
/// 
/// # Parameters:
/// - `context`: Test environment with access to `BanksClient`.
/// - `funder`: Signer who will fund and sign the transaction.
/// - `mint`: The SPL token mint associated with this account.
/// - `owner`: The intended owner of the account. This can be any Pubkey,
///            including invalid vault authorities for negative test cases.
///
/// # Returns:
/// - `Pubkey`: The public key of the newly created Token Account.
///
/// # Use cases:
/// - Create a token account with an owner that is *not* a known vault PDA.
/// - Simulate unauthorized transfers or vault misconfigurations.
/// - Bypass standard PDA validation to test edge-case security behavior.
///
/// # Example:

/// let fake_vault = Pubkey::new_unique(); // not a valid vault
/// let bogus_account = create_token_account(&mut context, &caller, &mint, &fake_vault).await;

///
/// # Notes:
/// - The account is initialized using the SPL Token `initialize_account`
///   instruction, and is owned by the SPL Token Program.
/// - If the `owner` does not match any known vault authority, the contract
///   logic is expected to reject operations against it.
/// ======================================================================

#[allow(dead_code)]
pub async fn create_token_account(
    context: &mut EnvProgramTestContext,
    funder: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey, // pode ser um owner inválido (para o teste)
) -> Pubkey {

    let token_account = Keypair::new();
    let rent = context.banks_client.get_rent().await.unwrap();
    let space = SplAccount::LEN;
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &funder.pubkey(),
        &token_account.pubkey(),
        lamports,
        space as u64,
        &spl_token::ID,
    );

    let initialize_ix = spl_token::instruction::initialize_account(
        &spl_token::ID,
        &token_account.pubkey(),
        mint,
        owner,
    ).unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, initialize_ix],
        Some(&funder.pubkey()),
        &[funder, &token_account],
        context.recent_blockhash,
    );

    context.banks_client.process_transaction(tx).await.unwrap();

    token_account.pubkey()
}


/// Fetch the balance of a token account (in base units, not in human tokens)
#[allow(dead_code)]
pub async fn fetch_token_balance(
    client: &mut BanksClient,
    token_account: &Pubkey,
) -> u64 {
    match client.get_account(*token_account).await.unwrap() {
        Some(account) => {
            let data = account.data();
            if data.len() >= 64 {
                let amount_bytes = &data[64..72];
                u64::from_le_bytes(amount_bytes.try_into().unwrap())
            } else {
                0
            }
        }
        None => 0,
    }
}

#[allow(dead_code)]
pub(crate) async fn get_proposal_last_id(context: &mut EnvProgramTestContext, governance_state: Pubkey) -> Result<u64, TransportError> {
    let account = context
        .banks_client
        .get_account(governance_state)
        .await?
        .ok_or(TransportError::Custom("GovernanceState account not found".to_string()))?;

    let governance_data = GovernanceState::try_deserialize(&mut account.data.as_slice())
        .map_err(|_| TransportError::Custom("Failed to deserialize GovernanceState".to_string()))?;

    Ok(governance_data.last_id)
}

/// ======================================================================
/// Logs and audits all major Soccial Token balances across vaults,
/// the mint, the mint authority, and the contract's fallback account.
///
/// This function verifies if all minted tokens are properly distributed
/// and accounted for in the system by checking SPL token balances and
/// PDA lamport balances.
///
/// # How it works:
/// - Fetches the lamport balance of important PDA accounts.
/// - Fetches the SPL Token balance of all vault token accounts, the mint authority,
///   and the contract fallback account.
/// - Fetches the total supply directly from the Mint account.
/// - Calculates the total tracked tokens across all major token accounts.
/// - Compares the tracked total with the known TOTAL_SUPPLY.
/// - Displays a ✅ success message if everything matches, or 🚨 a warning if discrepancies exist.
///
/// # Arguments
/// * `context` - The current ProgramTest environment.
/// * `caller` - The keypair of the authorized user (usually the contract owner).
///
/// # Output
/// Structured logs of all vault balances, fallback balances, mint supply, and audit result.
///
/// # Example

/// log_all_balances(&mut context, &caller).await?;

///
/// # Warning
/// - If any tokens are missing or unaccounted for, a warning will be displayed.
/// - Ensure that `TOTAL_SUPPLY` matches your actual intended token supply.
/// ======================================================================
#[allow(dead_code)]
pub async fn log_all_balances(
    context: &mut EnvProgramTestContext,
    caller: &Keypair,
) -> Result<(), TransportError> {
    use thousands::Separable;

    let seeds = derive_seeds(&context.program_id, &caller.pubkey());

    async fn fetch_balance(client: &mut BanksClient, pubkey: &Pubkey) -> u64 {
        match client.get_account(*pubkey).await.unwrap() {
            Some(account) => account.lamports(),
            None => 0,
        }
    }

    async fn fetch_token_balance(client: &mut BanksClient, pubkey: &Pubkey) -> u128 {
        match client.get_account(*pubkey).await.unwrap() {
            Some(account) => {
                let data = account.data();
                if data.len() >= 64 {
                    let amount_bytes = &data[64..72];
                    u64::from_le_bytes(amount_bytes.try_into().unwrap()) as u128
                } else {
                    0
                }
            },
            None => 0,
        }
    }

    async fn fetch_mint_supply(client: &mut BanksClient, pubkey: &Pubkey) -> u128 {
        match client.get_account(*pubkey).await.unwrap() {
            Some(account) => {
                let data = account.data();
                if data.len() >= 44 {
                    let supply_bytes = &data[36..44];
                    u64::from_le_bytes(supply_bytes.try_into().unwrap()) as u128
                } else {
                    0
                }
            },
            None => 0,
        }
    }

    msg!("════════════════════════════════════════════════════════════════════════════════════════════");
    msg!("📊 Soccial Token System Balances (Tokens | Lamports | % of Supply):");
    msg!("════════════════════════════════════════════════════════════════════════════════════════════");

    let token_state_balance = fetch_balance(&mut context.banks_client, &seeds.token_state).await;
    let staking_state_balance = fetch_balance(&mut context.banks_client, &seeds.staking_state).await;

    let mint_authority_token_balance = fetch_token_balance(&mut context.banks_client, &seeds.authority_token_account).await;
    let contract_token_balance = fetch_token_balance(&mut context.banks_client, &seeds.contract_token_account).await;
    let mint_supply = fetch_mint_supply(&mut context.banks_client, &seeds.token_mint).await;

    // Convert raw amounts to human-readable tokens
    let mint_authority_token_balance_in_tokens = units_to_tokens(mint_authority_token_balance as u64).unwrap_or(0);
    let contract_token_balance_in_tokens = units_to_tokens(contract_token_balance as u64).unwrap_or(0);
    let mint_supply_in_tokens = units_to_tokens(mint_supply as u64).unwrap_or(0);
    let total_supply_in_tokens = units_to_tokens(TOTAL_SUPPLY).unwrap_or(0);

    // Header
    msg!("{:<30} {:>20} {:>20} {:>10}", "Account", "Tokens", "Lamports", "% Supply");
    msg!("{:<30} {:>20} {:>20} {:>10}", "------------------------------", "--------------------", "--------------------", "----------");

    // Core PDA balances
    msg!("{:<30} {:>20} {:>20} {:>10}", "TokenState PDA", "-", token_state_balance.separate_with_commas(), "-");
    msg!("{:<30} {:>20} {:>20} {:>10}", "StakingState PDA", "-", staking_state_balance.separate_with_commas(), "-");

    // Contract-related token accounts
    msg!("{:<30} {:>20} {:>20} {:>10}",
        "Mint Authority Token Account",
        mint_authority_token_balance_in_tokens.separate_with_commas(),
        "-",
        format!("{:.2}%", (mint_authority_token_balance_in_tokens as f64 / total_supply_in_tokens as f64) * 100.0),
    );

    msg!("{:<30} {:>20} {:>20} {:>10}",
        "Contract Fallback SCTK Account",
        contract_token_balance_in_tokens.separate_with_commas(),
        "-",
        format!("{:.2}%", (contract_token_balance_in_tokens as f64 / total_supply_in_tokens as f64) * 100.0),
    );

    msg!("{:<30} {:>20} {:>20} {:>10}",
        "SPL Mint Total Supply",
        mint_supply_in_tokens.separate_with_commas(),
        "-",
        format!("{:.2}%", (mint_supply_in_tokens as f64 / total_supply_in_tokens as f64) * 100.0),
    );

    // Vault balances
    let vaults = vec![
        ("Offchain Reserve Vault", &seeds.offchain_reserve_vault, &seeds.offchain_reserve_vault_token_account),
        ("Liquidity Vault", &seeds.liquidity_vault, &seeds.liquidity_vault_token_account),
        ("Revenue Vault", &seeds.revenue_vault, &seeds.revenue_vault_token_account),
        ("Rewards Vault", &seeds.rewards_vault, &seeds.rewards_vault_token_account),
        ("Airdrop Vault", &seeds.airdrop_vault, &seeds.airdrop_vault_token_account),
        ("Vesting Vault", &seeds.vesting_vault, &seeds.vesting_vault_token_account),
        ("Insurance Vault", &seeds.insurance_vault, &seeds.insurance_vault_token_account),
        ("Treasury Vault", &seeds.treasury_vault, &seeds.treasury_vault_token_account),
        ("Reserved Supply Vault", &seeds.reserved_supply_vault, &seeds.reserved_supply_vault_token_account),
    ];

    let mut total_tracked_tokens =
        mint_authority_token_balance_in_tokens + contract_token_balance_in_tokens;
    let mut total_lamports: u64 = token_state_balance + staking_state_balance;

    for (name, vault_pda, vault_token_account) in vaults {
        let lamports = fetch_balance(&mut context.banks_client, vault_pda).await;
        let tokens = fetch_token_balance(&mut context.banks_client, vault_token_account).await;

        let tokens_in_human = units_to_tokens(tokens as u64).unwrap_or(0);

        let percentage = if total_supply_in_tokens > 0 {
            format!("{:.2}%", (tokens_in_human as f64 / total_supply_in_tokens as f64) * 100.0)
        } else {
            "-".to_string()
        };

        msg!(
            "{:<30} {:>20} {:>20} {:>10}",
            name,
            tokens_in_human.separate_with_commas(),
            lamports.separate_with_commas(),
            percentage
        );

        total_tracked_tokens += tokens_in_human;
        total_lamports += lamports;
    }

    // Summary
    msg!("════════════════════════════════════════════════════════════════════════════════════════════");

    let missing_tokens = total_supply_in_tokens.saturating_sub(total_tracked_tokens);

    msg!("{:<62} {:>20}", "🏛️ TOTAL SUPPLY:", total_supply_in_tokens.separate_with_commas());
    msg!(
        "{:<61} {:>20}  ({:.2}%)",
        "📋 TOTAL TRACKED:",
        total_tracked_tokens.separate_with_commas(),
        (total_tracked_tokens as f64 / total_supply_in_tokens as f64) * 100.0
    );
    msg!(
        "{:<62} {:>20}",
        "💰 TOTAL LAMPORTS:",
        total_lamports.separate_with_commas()
    );

    if missing_tokens == 0 {
        msg!("✅ All tokens are properly distributed and accounted for.");
    } else {
        msg!("🚨 WARNING: {} tokens are missing or unaccounted for!", missing_tokens.separate_with_commas());
    }

    msg!("════════════════════════════════════════════════════════════════════════════════════════════");

    Ok(())
}
