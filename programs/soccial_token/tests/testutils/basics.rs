// ============================================================================
// Soccial Token – Test Utilities & Environment Setup
// ----------------------------------------------------------------------------
//
// This module provides a full testing environment for the Soccial Token smart
// contract, including utilities for:
// - PDA derivation
// - On-chain account initialization
// - Instruction building
// - Integration test helpers and assertions
//
// ----------------------------------------------------------------------------
// Features:
// - Deterministic PDA generation via `derive_seeds()`
// - Test bootstrap with `initialize_token_env()` and `setup_test_env()`
// - Instruction builders and transaction senders with logging
// - Utilities for asserting account existence and expected failures
//
// ----------------------------------------------------------------------------
// Key Components:
// - `AccountSeeds`: Struct with all PDAs used by the contract
// - `TestEnv`: Struct encapsulating the full test setup (banks client, signer, etc.)
// - `initialize_token_env`: Creates and initializes all contract accounts
// - `send_tx`, `send_ix`: Utilities to submit instructions and handle blockhash
// - `assert_all_pdas_exist`: Verifies all critical PDAs were created
//
// ----------------------------------------------------------------------------
// Usage:
// Use these utilities in Rust integration tests to simulate contract behavior
// and ensure deterministic behavior in testnet/mainnet deployments.
//
// ----------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ============================================================================

use anchor_lang::{solana_program::example_mocks::solana_sdk::sysvar, InstructionData, ToAccountMetas};
use anchor_spl::associated_token::spl_associated_token_account;
//use anchor_spl::associated_token::spl_associated_token_account;
use solana_program_test::{BanksClient, BanksClientError, ProgramTest};
use solana_sdk::{
    account::Account, clock::Clock, hash::Hash, instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, Signer}, system_instruction, system_program, transaction::{Transaction, TransactionError}, transport::TransportError  
};
use soccial_token::{accounts as soccial_accounts, initialize::initialize::team_vesting, instruction as soccial_instruction};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};
use solana_sdk::instruction::InstructionError;
use core::str::FromStr;


// Holds all Program Derived Addresses (PDAs) required by the Soccial Token program.
///
/// These addresses are deterministically derived using known seeds (e.g. "token_state", "user", etc.)
/// combined with the caller’s public key where needed. This struct is used throughout testing and
/// initialization flows to provide easy access to every core account in the contract.

#[allow(dead_code)]
pub struct AccountSeeds {

    pub user_access: Pubkey,
    pub vesting_state: Pubkey,  
    pub vesting_schedule: Pubkey,         // Default: b"vesting_schedule" + None
    pub vesting_schedule_pda: Pubkey,     // Custom: b"vesting_schedule" + caller
    pub user_adopter_info: Pubkey,
    pub staking_account: Pubkey,
    pub staking_state: Pubkey,
    pub governance_state: Pubkey,
    
    pub token_state: Pubkey,
    pub token_mint: Pubkey,
    pub user_token_ata: Pubkey,
    pub mint_authority: Pubkey,
    pub authority_token_account: Pubkey,
    pub mint_authority_bump: u8,
    pub contract_token_account: Pubkey, // The token account that hold tokens directly sent to the contract id
    pub contract_token_owner: Pubkey,

    
    pub offchain_reserve_vault: Pubkey,
    pub revenue_vault: Pubkey,
    pub rewards_vault: Pubkey,
    pub airdrop_vault: Pubkey,
    pub liquidity_vault: Pubkey,
    pub staking_vault: Pubkey,
    pub reserved_supply_vault: Pubkey,
    pub vesting_vault: Pubkey,
    pub insurance_vault: Pubkey,
    pub treasury_vault: Pubkey,
    
    pub offchain_reserve_vault_token_account: Pubkey,
    pub revenue_vault_token_account: Pubkey,
    pub rewards_vault_token_account: Pubkey,
    pub airdrop_vault_token_account: Pubkey,
    pub liquidity_vault_token_account: Pubkey,
    pub reserved_supply_vault_token_account: Pubkey,
    pub vesting_vault_token_account: Pubkey,
    pub staking_vault_token_account: Pubkey,
    pub insurance_vault_token_account: Pubkey,
    pub treasury_vault_token_account: Pubkey,

    pub offchain_reserve_vault_authority: Pubkey,
    pub revenue_vault_authority: Pubkey,
    pub rewards_vault_authority: Pubkey, 
    pub airdrop_vault_authority: Pubkey, 
    pub liquidity_vault_authority: Pubkey,
    pub reserved_supply_vault_authority: Pubkey,
    pub vesting_vault_authority: Pubkey,
    pub staking_vault_authority: Pubkey,
    pub insurance_vault_authority: Pubkey,
    pub treasury_vault_authority: Pubkey,

    pub team1_vesting_schedule: Pubkey,
    pub team2_vesting_schedule: Pubkey,

}

#[allow(dead_code)]
pub fn derive_proposal_account(program_id: &Pubkey, proposal_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"proposal", &proposal_id.to_le_bytes()],
        program_id,
    )
}

/// Helper to get the correct vault token account by vault seed name.
#[allow(dead_code)]
pub fn get_vault_token_account(seeds: &AccountSeeds, seed_name: &str) -> Pubkey {
    match seed_name {
        "offchain_reserve_vault" => seeds.offchain_reserve_vault_token_account,
        "liquidity_vault" => seeds.liquidity_vault_token_account,
        "staking_vault" => seeds.staking_vault_token_account,
        "revenue_vault" => seeds.revenue_vault_token_account,
        "rewards_vault" => seeds.rewards_vault_token_account,
        "airdrop_vault" => seeds.airdrop_vault_token_account,
        "vesting_vault" => seeds.vesting_vault_token_account,
        "insurance_vault" => seeds.insurance_vault_token_account,
        "treasury_vault" => seeds.treasury_vault_token_account,
        "reserved_supply_vault" => seeds.reserved_supply_vault_token_account,
        _ => panic!("Invalid vault seed name: {}", seed_name),
    }
}

/// Derives all the required Program Derived Addresses (PDAs) for the Soccial Token program
/// using fixed seeds and the caller’s public key where necessary.
///
/// This utility function ensures consistent and deterministic account addressing for
/// all contract modules, which is essential for tests, deployments, and instructions that
/// depend on PDAs being derived exactly as they are in the on-chain logic.
///
/// # Arguments
/// * program_id - The ID of the deployed on-chain program (used in PDA derivation).
/// * caller - The public key of the user or test actor invoking the contract (used for PDAs that require personalization).
///
/// # Returns
/// An AccountSeeds struct containing all relevant PDAs for the contract.
#[allow(dead_code)]
pub fn derive_seeds(program_id: &Pubkey, caller: &Pubkey) -> AccountSeeds {
    fn derive(name: &'static [u8], extra: Option<&Pubkey>, program_id: &Pubkey) -> (Pubkey, u8) {
        match extra {
            Some(p) => Pubkey::find_program_address(&[name, p.as_ref()], program_id),
            None => Pubkey::find_program_address(&[name], program_id),
        }
    }

    // Derive all PDAs
    let (user_access, _token_state_bump) = derive(b"user_access", Some(caller), program_id);
    let (token_state, _token_state_bump) = derive(b"token_state", None, program_id);
    let (vesting_state, _vesting_state_bump) = derive(b"vesting_state", None, program_id);
    let (vesting_schedule, _vesting_schedule_bump) = derive(b"vesting_schedule", None, program_id);
    let (vesting_schedule_pda, _vesting_schedule_pda_bump) = derive(b"vesting_schedule", Some(caller), program_id);
    let (user_adopter_info, _user_adopter_info_bump) = derive(b"user_adopter", Some(caller), program_id);
    let (staking_account, _staking_account_bump) = derive(b"staking_account", Some(caller), program_id);
    let (staking_state, _staking_state_bump) = derive(b"staking_state", None, program_id);
    let (governance_state, _governance_state_bump) = derive(b"governance_state", None, program_id);

    let (token_mint, _token_mint_bump) = derive(b"token_mint", None, program_id);
    let (mint_authority, mint_authority_bump) = derive(b"mint_authority", None, program_id);
    
    let (contract_token_owner, _contract_token_owner_bump) = derive(b"contract_token_owner", None, program_id); 
    let contract_token_account = get_associated_token_address(&contract_token_owner, &token_mint);


    let (offchain_reserve_vault, _offchain_reserve_vault_bump) = derive(b"offchain_reserve_vault", None, program_id);
    let (revenue_vault, _revenue_vault_bump) = derive(b"revenue_vault", None, program_id);
    let (rewards_vault, _rewards_vault_bump) = derive(b"rewards_vault", None, program_id);
    let (airdrop_vault, _airdrop_vault_bump) = derive(b"airdrop_vault", None, program_id);
    let (liquidity_vault, _liquidity_vault_bump) = derive(b"liquidity_vault", None, program_id);
    let (staking_vault, _staking_vault_bump) = derive(b"staking_vault", None, program_id);
    let (reserved_supply_vault, _reserved_supply_vault_bump) = derive(b"reserved_supply_vault", None, program_id);
    let (vesting_vault, _vesting_vault_bump) = derive(b"vesting_vault", None, program_id);
    let (insurance_vault, _insurance_vault_bump) = derive(b"insurance_vault", None, program_id);
    let (treasury_vault, _treasury_vault_bump) = derive(b"treasury_vault", None, program_id);
    

    // Derive Associated Token Accounts (ATAs)
    let user_token_ata = get_associated_token_address(caller, &token_mint);
    let authority_token_account = get_associated_token_address(&mint_authority, &token_mint);
    

    let offchain_reserve_vault_token_account = get_associated_token_address(&offchain_reserve_vault, &token_mint);
    let revenue_vault_token_account = get_associated_token_address(&revenue_vault, &token_mint);
    let rewards_vault_token_account = get_associated_token_address(&rewards_vault, &token_mint);
    let airdrop_vault_token_account = get_associated_token_address(&airdrop_vault, &token_mint);
    let liquidity_vault_token_account = get_associated_token_address(&liquidity_vault, &token_mint);
    let staking_vault_token_account = get_associated_token_address(&staking_vault, &token_mint);
    let reserved_supply_vault_token_account = get_associated_token_address(&reserved_supply_vault, &token_mint);
    let vesting_vault_token_account = get_associated_token_address(&vesting_vault, &token_mint);
    let insurance_vault_token_account = get_associated_token_address(&insurance_vault, &token_mint);
    let treasury_vault_token_account = get_associated_token_address(&treasury_vault, &token_mint);
  
    
    let (offchain_reserve_vault_authority, _bump) = Pubkey::find_program_address(&[b"offchain_reserve_vault"], &program_id);
    let (revenue_vault_authority, _bump) = Pubkey::find_program_address(&[b"revenue_vault"], &program_id);
    let (rewards_vault_authority, _bump) = Pubkey::find_program_address(&[b"rewards_vault"], &program_id);
    let (airdrop_vault_authority, _bump) = Pubkey::find_program_address(&[b"airdrop_vault"], &program_id);
    let (liquidity_vault_authority, _bump) = Pubkey::find_program_address(&[b"liquidity_vault"], &program_id);
    let (staking_vault_authority, _bump) = Pubkey::find_program_address(&[b"staking_vault"], &program_id);
    let (reserved_supply_vault_authority, _bump) = Pubkey::find_program_address(&[b"reserved_supply_vault"], &program_id);
    let (vesting_vault_authority, _bump) = Pubkey::find_program_address(&[b"vesting_vault"], &program_id);
    let (insurance_vault_authority, _bump) = Pubkey::find_program_address(&[b"insurance_vault"], &program_id);
    let (treasury_vault_authority, _bump) = Pubkey::find_program_address(&[b"treasury_vault"], &program_id);

    let team1_pubkey = Pubkey::from_str(team_vesting::TEAM1_STR).expect("Invalid TEAM1_STR");
    let team2_pubkey = Pubkey::from_str(team_vesting::TEAM2_STR).expect("Invalid TEAM2_STR");

    let (team1_vesting_schedule, _) = Pubkey::find_program_address(
        &[b"vesting_schedule", team1_pubkey.as_ref(), &1u64.to_le_bytes()],
        program_id,
    );

    let (team2_vesting_schedule, _) = Pubkey::find_program_address(
        &[b"vesting_schedule", team2_pubkey.as_ref(), &2u64.to_le_bytes()],
        program_id,
    );

    AccountSeeds {
        token_state,
        user_access,
        vesting_state,
        vesting_schedule,
        vesting_schedule_pda,
        user_adopter_info,
        staking_account,
        staking_state,
        governance_state,

        token_mint,
        mint_authority,
        mint_authority_bump,
        user_token_ata,
        authority_token_account,
        contract_token_owner,
        contract_token_account,

        
        offchain_reserve_vault,
        liquidity_vault,
        staking_vault,
        revenue_vault,
        rewards_vault,
        airdrop_vault,
        reserved_supply_vault,
        vesting_vault,
        insurance_vault,
        treasury_vault,
        
        offchain_reserve_vault_token_account,
        liquidity_vault_token_account,
        staking_vault_token_account,
        revenue_vault_token_account,
        rewards_vault_token_account,
        airdrop_vault_token_account,
        reserved_supply_vault_token_account,
        vesting_vault_token_account,
        insurance_vault_token_account,
        treasury_vault_token_account,
        
        offchain_reserve_vault_authority,
        liquidity_vault_authority,
        staking_vault_authority,
        revenue_vault_authority,
        rewards_vault_authority,
        airdrop_vault_authority,
        reserved_supply_vault_authority,
        vesting_vault_authority,
        insurance_vault_authority,
        treasury_vault_authority,

        team1_vesting_schedule,
        team2_vesting_schedule
    }
    
}


/// Builds a complete vector of AccountMeta objects for use in lightweight or diagnostic test instructions.
///
/// This function is designed to provide the minimal set of accounts typically used by generic, non-state-changing
/// instructions like debug_log, internal diagnostics, or function dispatchers that depend on full context access
/// (e.g., the former GenericCall handler).
///
/// It includes all core PDAs expected to exist after a full initialization, mimicking the layout expected
/// by most instructions without requiring a full Anchor context struct.
///
/// # Arguments
/// * caller - The public key of the transaction initiator (used as the signer).
/// * seeds - A reference to the AccountSeeds struct containing all precomputed PDAs.
///
/// # Returns
/// A vector of AccountMeta entries to be passed directly into a Solana Instruction.
///
/// # Notes
/// - The caller is marked as is_signer = true, since it must sign the transaction.
/// - All other accounts are treated as writable (is_signer = false) except the system program.
/// - This vector should only be used for instructions that **do not require strict account constraints**,
///   such as internal testing or contract introspection logic.
///
/// # Example Usage
/// 
/// let ix = Instruction {
///     program_id,
///     accounts: build_test_call_accounts(&caller.pubkey(), &seeds),
///     data: soccial_instruction::DebugLog {}.data(),
/// };
///


///
/// This approach is useful when testing instruction routing or debugging generic access patterns.
#[allow(dead_code)]
pub fn build_test_call_accounts(caller: &Pubkey, seeds: &AccountSeeds) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(*caller, true),                        // Signer / caller
        AccountMeta::new(seeds.user_access, false),             // User
        AccountMeta::new(seeds.token_state, false),             // Core token config
        AccountMeta::new(seeds.vesting_schedule, false),        // Vesting configuration
        AccountMeta::new(seeds.user_adopter_info, false),       // Early adopters & whitelist
        AccountMeta::new(seeds.staking_account, false),         // User staking data
        AccountMeta::new(seeds.staking_state, false),           // Staking global config
        AccountMeta::new(seeds.governance_state, false),      
        AccountMeta::new(seeds.token_mint, false),              // Token Mint
        AccountMeta::new(seeds.mint_authority, false),         // Mint Authority
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false), // System program (read-only)
    ]
}



/// Verifies the existence and proper initialization of all critical PDAs.
///
/// This helper function is typically used in integration tests immediately after 
/// running the initialize_token_env() procedure to ensure all required PDAs 
/// (Program Derived Addresses) have been correctly created and are present on-chain.
///
/// It checks the presence of each account by attempting to fetch its on-chain data using 
/// the BanksClient. It also asserts that each account holds a positive lamport balance, 
/// which is a strong indication that the account was initialized properly.
///
/// # Parameters
/// - banks_client: A mutable reference to the BanksClient, which simulates 
///   interaction with the Solana runtime in the test environment.
/// - program_id: The program ID of the deployed Soccial Token contract, used to derive PDAs.
/// - caller: The test caller’s public key, also used in PDA derivation for caller-specific accounts.
///
/// # Behavior
/// - For each PDA used by the Soccial Token contract, this function fetches the account.
/// - It asserts that the account exists and has lamports > 0, which indicates successful initialization.
/// - If any account is missing or empty, the test will panic with an informative message.
///
/// # Use Case
/// This is a defensive test to catch potential misconfigurations or missing initializations early.
///
/// # Example
/// 
/// assert_all_pdas_exist(&mut banks_client, &program_id, &caller).await;
///


///
/// # Panics
/// - If the account cannot be fetched.
/// - If the account does not exist.
/// - If the account has 0 lamports (i.e., is likely uninitialized).

/// Returns the list of PDAs that must exist after initialization
#[allow(dead_code)]
pub fn expected_pda_addresses(seeds: &AccountSeeds) -> Vec<Pubkey> {
    vec![
        seeds.token_state,
        seeds.governance_state,
        seeds.vesting_state,
        seeds.staking_state,
        seeds.token_mint,
        seeds.authority_token_account,
        
        // Vaults
        seeds.offchain_reserve_vault,
        seeds.liquidity_vault,
        seeds.staking_vault,
        seeds.revenue_vault,
        seeds.rewards_vault,
        seeds.airdrop_vault,
        seeds.reserved_supply_vault,
        seeds.vesting_vault,
        seeds.insurance_vault,
        seeds.treasury_vault,
    ]
}

/// Derives the PDA (Program Derived Address) for a specific governance proposal.
///
/// # Arguments
/// * `program_id` - The program ID of the smart contract.
/// * `caller` - The public key of the user initiating the proposal (used as part of the seed).
/// * `proposal_id` - The unique identifier of the proposal.
///
/// # Returns
/// * `Pubkey` - The derived PDA for the specified proposal.
///
/// # Seed Format
/// The seed is constructed as:
///     ["proposal_{proposal_id}", caller_pubkey]
/// This ensures that each proposal PDA is unique per user and proposal ID.
///
/// # Notes
/// This PDA must match the derivation logic used in the on-chain program to validate proposal ownership and context.
#[allow(dead_code)]
pub fn derive_proposal_pda(program_id: &Pubkey, caller: &Pubkey, proposal_id: u64) -> Pubkey {
    let seed = format!("proposal_{}", proposal_id);
    Pubkey::find_program_address(&[seed.as_bytes(), caller.as_ref()], program_id).0
}

/// Asserts that all expected PDAs exist given a program_id and caller
#[allow(dead_code)]
pub async fn assert_all_pdas_exist(
    banks_client: &mut BanksClient,
    program_id: &Pubkey,
    caller: &Pubkey,
) {
    let seeds = derive_seeds(program_id, caller);
    assert_all_pdas_exist_from_seeds(banks_client, &seeds).await;
}

/// Asserts that all expected PDAs exist given pre-derived seeds
#[allow(dead_code)]
pub async fn assert_all_pdas_exist_from_seeds(
    banks_client: &mut BanksClient,
    seeds: &AccountSeeds,
) {
    for address in expected_pda_addresses(seeds) {
        match banks_client.get_account(address).await {
            Ok(Some(account)) => {
                assert!(
                    account.lamports > 0,
                    "Account {} exists but has 0 lamports (likely not initialized properly)",
                    address
                );
            },
            Ok(None) => panic!("❌ Account {} not found on-chain", address),
            Err(e) => panic!("❌ Failed to fetch account {}: {:?}", address, e),
        }
    }
}

/// Initializes the full environment required to interact with the Soccial Token contract.
///
/// This function executes the core setup instructions sequentially, including:
/// - InitializeToken: Sets up core state (token state, user account, permissions, governance, etc.)
/// - InitEconomy: Initializes remaining economic accounts (staking, vesting, liquidity).
///
/// It returns the re-derived PDAs after the instructions, which may change due to
/// on-chain constraints or updates.
///
/// # Arguments
/// * program_id - The deployed program's public key.
/// * banks_client - Test environment's bank client for sending transactions.
/// * payer - The main account funding the setup transactions.
/// * caller - The account acting as the contract initializer (i.e., the future token owner).
/// * recent_blockhash - A mutable reference to the last known blockhash.
///
/// # Returns
/// A fully populated AccountSeeds struct with all updated PDAs.
///
/// # Errors
/// Returns TransportError if any transaction fails or is rejected by the runtime.
#[allow(dead_code)]
pub async fn initialize_token_env(
    program_id: Pubkey,
    banks_client: &mut BanksClient,
    payer: &Keypair,
    caller: &Keypair,
    mut recent_blockhash: Hash,
) -> Result<AccountSeeds, TransportError> {
    // Derive all PDA addresses based on caller + program_id before starting.
    let mut seeds = derive_seeds(&program_id, &caller.pubkey());
    
    
    // ================================
    // STEP 1: Initialize Token
    // - Creates token_state
    // - Links user_account to caller
    // - Sets up permissions and governance
    // - Assigns initial supply arguments
    // ================================
    {
        let instruction = Instruction {
            program_id,
            accounts: build_initialize_token_accounts(&seeds, caller.pubkey()),
            data: soccial_instruction::InitializeToken {}
            
            .data(),
        };

        send_tx_refresh_seeds(
            "Initialize Token",         // Step label for logging
            instruction,                      // Anchor-compatible instruction
            banks_client,
            payer,
            caller,
            &mut recent_blockhash,
            &mut seeds,
            program_id,
        ).await?;
    }

    // ================================
    // STEP 2: Initialize Economy
    // - Creates staking and vesting structures
    // - Sets vaults management accounts
    // ================================
    {
        let instruction = Instruction {
            program_id,
            accounts: build_initialize_economy_accounts(&seeds, caller.pubkey()),
            data: soccial_instruction::InitializeEconomy {}
            .data(),
        };

        send_tx_refresh_seeds(
            "Initialize Economy",        
            instruction,
            banks_client,
            payer,
            caller,
            &mut recent_blockhash,
            &mut seeds,
            program_id,
        ).await?;
    }

    // ================================
    // STEP 3: Initialize SPL Token Mint
    // - Creates ATA for each vault
    // - Mints initial token allocations
    // - Transfers vesting tokens to vesting vault
    // ================================
    {
        let instruction = Instruction {
            program_id,
            accounts: build_initialize_spl_token_accounts(&seeds, caller.pubkey()),
            data: soccial_instruction::InitializeSplToken {}
            .data(),
        };
        
        send_tx_refresh_seeds(
            "Initialize SPL Token",
            instruction,
            banks_client,
            payer,
            caller,
            &mut recent_blockhash,
            &mut seeds,
            program_id,
        ).await?;
    }

    // ================================
    // STEP 4: Initialize Founders Vesting
    // - Creates vesting schedules for founders
    // - Transfers tokens from liquidity_vault to vesting_vault
    // ================================
    {
        let instruction = Instruction {
            program_id,
            accounts: build_initialize_founders_vesting_accounts(&seeds, caller.pubkey()),
            data: soccial_instruction::InitializeFoundersVesting {}.data(),
        };

        send_tx_refresh_seeds(
            "Initialize Founders Vesting",
            instruction,
            banks_client,
            payer,
            caller,
            &mut recent_blockhash,
            &mut seeds,
            program_id,
        ).await?;
    }
    
    // Return all re-derived account addresses for use in further tests or assertions
    Ok(seeds)
}

#[allow(dead_code)]
pub fn build_initialize_token_accounts(
    seeds: &AccountSeeds,
    caller: Pubkey,
) -> Vec<AccountMeta> {
    soccial_accounts::InitializeToken {
        caller,
        token_state: seeds.token_state,
        mint: seeds.token_mint,
        mint_authority: seeds.mint_authority,
        authority_token_account: seeds.authority_token_account,
        governance_state: seeds.governance_state,
        staking_state: seeds.staking_state,
        vesting_state: seeds.vesting_state,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
        token_program: spl_token::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None)
}

#[allow(dead_code)]
pub fn build_initialize_economy_accounts(
    seeds: &AccountSeeds,
    caller: Pubkey,
) -> Vec<AccountMeta> {
    soccial_accounts::InitializeEconomy {
        caller,
        token_state: seeds.token_state,
        staking_state: seeds.staking_state,

        offchain_reserve_vault: seeds.offchain_reserve_vault,
        liquidity_vault: seeds.liquidity_vault,
        staking_vault: seeds.staking_vault,
        revenue_vault: seeds.revenue_vault,
        rewards_vault: seeds.rewards_vault,
        airdrop_vault: seeds.airdrop_vault,
        reserved_supply_vault: seeds.reserved_supply_vault,
        vesting_vault: seeds.vesting_vault,
        insurance_vault: seeds.insurance_vault,
        treasury_vault: seeds.treasury_vault,

        token_mint: seeds.token_mint,
        system_program: system_program::ID,
        token_program: spl_token::ID,
        associated_token_program: spl_associated_token_account::ID,
    }
    .to_account_metas(None)
}

#[allow(dead_code)]
pub fn build_initialize_spl_token_accounts(
    seeds: &AccountSeeds,
    caller: Pubkey,
) -> Vec<AccountMeta> {
    
    soccial_accounts::InitializeSplToken {
        caller,
        mint: seeds.token_mint,
        mint_authority: seeds.mint_authority,
        token_state: seeds.token_state,
        system_program: system_program::ID,
        rent: sysvar::rent::ID,
        token_program: spl_token::ID,
        associated_token_program: spl_associated_token_account::ID,
        authority_token_account: seeds.authority_token_account,

        contract_token_owner: seeds.contract_token_owner,
        contract_token_account: seeds.contract_token_account,
       
        offchain_reserve_vault: seeds.offchain_reserve_vault,
        liquidity_vault: seeds.liquidity_vault,
        staking_vault: seeds.staking_vault,
        revenue_vault: seeds.revenue_vault,
        rewards_vault: seeds.rewards_vault,
        airdrop_vault: seeds.airdrop_vault,
        reserved_supply_vault: seeds.reserved_supply_vault,
        vesting_vault: seeds.vesting_vault,
        insurance_vault: seeds.insurance_vault,
        treasury_vault: seeds.treasury_vault,

        offchain_reserve_vault_token_account: seeds.offchain_reserve_vault_token_account,
        liquidity_vault_token_account: seeds.liquidity_vault_token_account,
        staking_vault_token_account: seeds.staking_vault_token_account,
        revenue_vault_token_account: seeds.revenue_vault_token_account,
        rewards_vault_token_account: seeds.rewards_vault_token_account,
        airdrop_vault_token_account: seeds.airdrop_vault_token_account,
        reserved_supply_vault_token_account: seeds.reserved_supply_vault_token_account,
        vesting_vault_token_account: seeds.vesting_vault_token_account,
        insurance_vault_token_account: seeds.insurance_vault_token_account,
        treasury_vault_token_account: seeds.treasury_vault_token_account,
    }
    .to_account_metas(None)
}

#[allow(dead_code)]
pub fn build_initialize_founders_vesting_accounts(
    seeds: &AccountSeeds,
    caller: Pubkey,
) -> Vec<AccountMeta> {
    soccial_accounts::InitializeFoundersVesting {
        caller,
        vesting_state: seeds.vesting_state,
        team1_vesting_schedule: seeds.team1_vesting_schedule,
        team2_vesting_schedule: seeds.team2_vesting_schedule,
        token_state: seeds.token_state,
        system_program: system_program::ID,
        token_program: spl_token::ID,
    }
    .to_account_metas(None)
}

/// Sends a Solana instruction with custom signer seeds (for PDA-based signing).
///
/// Useful when invoking instructions where one or more signers are PDAs authorized via `invoke_signed`.
///
/// # Arguments
/// * `client` - The Solana test client.
/// * `payer` - The transaction fee payer.
/// * `signers` - A list of Keypairs that must sign the transaction (e.g., the payer).
/// * `ix` - The instruction to be executed.
/// * `blockhash` - The most recent blockhash.
/// * `signer_seeds` - A slice of seed arrays for PDA signing.
///
/// # Returns
/// Result indicating whether the transaction succeeded or failed.
#[allow(dead_code)]
pub async fn send_ix_with_signer_seeds(
    client: &mut BanksClient,
    payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
    blockhash: Hash,
    _signer_seeds: &[&[&[u8]]], // 👈 opcional, só para manter a assinatura uniforme
) -> Result<(), BanksClientError> {
    // Build transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));

    // Only the actual signers are used (no PDA signing in tests)
    tx.sign(signers, blockhash);

    // Submit transaction
    client.process_transaction(tx).await
}



/// Sends a transaction and rederives all PDA seeds afterward to ensure consistency.
///
/// This helper wraps the full instruction lifecycle with logging, seed refreshing,
/// and centralized error handling.
///
/// # Arguments
/// * label - A short identifier for debug logging (e.g. "InitEconomy").
/// * instruction - The fully formed Solana instruction (account metas + data).
/// * banks_client - The Solana test client for transaction submission.
/// * payer - Account funding the transaction.
/// * caller - The main signer / initiator of the instruction.
/// * recent_blockhash - Mutable ref to the current blockhash (refreshed automatically).
/// * seeds - Mutable ref to the account seed struct to be refreshed after execution.
/// * program_id - The program ID used in PDA derivation.
///
/// # Returns
/// Ok(()) if the transaction succeeded and seeds were updated, or TransportError.
#[allow(dead_code)]
pub async fn send_tx_refresh_seeds(
    label: &str,
    instruction: Instruction,
    banks_client: &mut BanksClient,
    payer: &Keypair,
    caller: &Keypair,
    recent_blockhash: &mut Hash,
    seeds: &mut AccountSeeds,
    program_id: Pubkey,
) -> Result<(), TransportError> {
    // Submit the transaction and propagate error if it fails
    send_tx(
        label,
        instruction,
        banks_client,
        payer,
        caller,
        recent_blockhash,
    ).await?;

    // Recompute all seed PDAs after execution (in case state affects derivation)
    *seeds = derive_seeds(&program_id, &caller.pubkey());
    Ok(())
}

/// Sends a Solana transaction with debug output and error reporting.
///
/// This is a low-level helper to handle transaction creation, signing,
/// submission, and error handling for integration tests.
///
/// # Arguments
/// * label - Human-readable label for console logging.
/// * instruction - The instruction to send (already includes all accounts and data).
/// * banks_client - The testing client that simulates a Solana runtime.
/// * payer - The funding account for transaction fees.
/// * caller - The signing account invoking the instruction.
/// * recent_blockhash - A mutable reference to the current blockhash (will be updated).
///
/// # Returns
/// Ok(()) if the transaction succeeded; otherwise TransportError.
#[allow(dead_code)]
pub async fn send_tx(
    label: &str,
    instruction: Instruction,
    banks_client: &mut BanksClient,
    payer: &Keypair,
    caller: &Keypair,
    recent_blockhash: &mut Hash,
) -> Result<(), TransportError> {
    // Always get a fresh blockhash to avoid transaction expiry
    *recent_blockhash = banks_client.get_latest_blockhash().await?;

    // Construct and sign the transaction
    let mut tx = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    tx.sign(&[payer, caller], *recent_blockhash);

    // Submit and log outcome
    if let Err(err) = banks_client.process_transaction(tx).await {
        eprintln!("❌ Step '{}' failed: {:?}", label, err);
        return Err(err.into());
    }

    println!("✅ Step '{}' succeeded.", label);
    Ok(())
}

/// Sends a single Solana instruction as a transaction to the test validator (BanksClient).
///
/// This helper wraps the full lifecycle of a test transaction, including:
/// - Transaction construction using the provided instruction and payer
/// - Signing the transaction with one or more signers
/// - Submitting the transaction to the BanksClient for processing
///
/// This is typically used for **custom instructions** or test calls outside of the
/// Anchor instruction flow. It's especially useful when building instructions manually
/// (e.g., via Instruction struct) or when bypassing Anchor’s procedural macros.
///
/// # Arguments
/// * client - The mutable reference to the Solana BanksClient (test validator backend).
/// * payer - The main fee payer for the transaction (must sign the transaction).
/// * signers - Slice of Keypair references used to sign the transaction (includes payer and others).
/// * ix - The Solana Instruction to be executed (with accounts, program ID, and data).
/// * blockhash - A recent blockhash fetched from the test validator (required for signing).
///
/// # Returns
/// * Ok(()) if the transaction was successfully processed.
/// * Err(BanksClientError) if the transaction failed (e.g., account constraint violation, runtime panic).
///
/// # Notes
/// - This function logs a failure with eprintln! to help debugging during test runs.
/// - It does **not** panic, allowing graceful error handling inside the test logic.
///
/// # Example
/// 
///
/// # Dev Tips
/// - Use [send_ix_expect_success] or [send_ix_expect_failure] for assertion-based testing.
#[allow(dead_code)]
pub async fn send_ix(
    client: &mut BanksClient,
    payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
    blockhash: Hash,
) -> Result<(), BanksClientError> {
    // Create a transaction with the given instruction and payer
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));

    // Sign the transaction using the specified keypairs
    tx.sign(signers, blockhash);

    // Submit the transaction to the test validator and handle the result
    match client.process_transaction(tx).await {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("❌  send_ix_debug failed: {:?}", e); // Helpful log for debugging test failures
            Err(e)
        }
    }
}

/// Ensures that the Associated Token Account (ATA) exists for a given wallet and token mint.
/// If the ATA is missing, it will be created using the payer's lamports.
///
/// # Arguments
/// * banks_client - Mutable reference to the test banks client.
/// * payer - The account that will pay for the creation of the ATA (usually the test payer).
/// * owner - The wallet address that will own the resulting ATA.
/// * mint - The token mint for which the ATA is associated.
/// * ata - The expected address of the associated token account to check/create.
/// * recent_blockhash - Latest blockhash required to sign the transaction.
///
/// # Behavior
/// - Checks if the given ATA already exists on-chain.
/// - If it does not exist, constructs and sends a create_associated_token_account instruction.
/// - If it already exists, no action is taken.
///
/// # Returns
/// * Ok(()) if the ATA exists or is successfully created.
/// * Err(TransportError) if something goes wrong while sending the transaction.
#[allow(dead_code)]
pub async fn create_ata_if_missing(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    owner: &Keypair,
    mint: &Pubkey,
    ata: &Pubkey,
    recent_blockhash: solana_sdk::hash::Hash,
) -> Result<(), TransportError> {
    // Query the chain to see if the ATA already exists
    let maybe_ata_account = banks_client.get_account(*ata).await?;

    // If the account does not exist, we create it
    if maybe_ata_account.is_none() {
        // Build the instruction to create the ATA using the SPL helper
        let create_ix: Instruction = create_associated_token_account(
            &payer.pubkey(),  // Payer of the ATA creation
            &owner.pubkey(),  // Owner of the token account
            mint,             // Token mint
            &spl_token::ID,   // SPL Token Program ID
        );

        // Create and sign the transaction
        let tx = Transaction::new_signed_with_payer(
            &[create_ix],             // Only 1 instruction: create ATA
            Some(&payer.pubkey()),    // The payer of fees
            &[payer],                 // Signers
            recent_blockhash,         // Latest blockhash
        );

        // Send the transaction to the test client
        banks_client.process_transaction(tx).await?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn derive_token_mint(program_id: &Pubkey) -> Pubkey {
    let (mint, _) = Pubkey::find_program_address(&[b"token_mint"], program_id);
    mint
}

#[allow(dead_code)]
pub fn derive_user_ata(program_id: &Pubkey, user: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(user, &derive_token_mint(program_id))
}

/// Sends a transaction and asserts that it **succeeds**.
///
/// This function is used in integration tests where a transaction
/// is expected to succeed. If the transaction fails (e.g., due to constraint violations,
/// missing accounts, or logic errors), the test will panic with an error message.
///
/// # Arguments
/// * client - A mutable reference to the test BanksClient (validator simulator).
/// * payer - The fee payer (must sign the transaction).
/// * signers - Slice of Keypair references used to sign the transaction (payer and others).
/// * ix - The Solana Instruction to execute.
/// * blockhash - A recent blockhash for signing.
///
/// # Panics
/// If the transaction fails, it logs the error and **panics** the test with a descriptive message.
///
/// # Example
/// 
#[allow(dead_code)]
pub async fn send_ix_expect_success(
    client: &mut BanksClient,
    payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
    blockhash: Hash,
) {
    // Build and sign the transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(signers, blockhash);

    // Submit and assert success
    let result = client.process_transaction(tx).await;

    assert!(
        result.is_ok(),
        "❌  Transaction failed when success was expected: {:?}",
        result
    );
}


/// Sends a transaction and asserts that it **fails** as expected.
///
/// This function is used in negative testing scenarios where a failure is
/// expected (e.g., unauthorized access, uninitialized accounts, invalid data).
///
/// If the transaction **succeeds**, it logs an error and returns a dummy error
/// to fail the test gracefully.
///
/// # Arguments
/// * client - A mutable reference to the test BanksClient.
/// * payer - The fee payer (must sign the transaction).
/// * signers - Slice of Keypair references used to sign the transaction.
/// * ix - The Solana Instruction expected to fail.
/// * blockhash - A recent blockhash for signing.
///
/// # Returns
/// * Ok(()) if the failure was successful (i.e., as expected).
/// * Err(BanksClientError) if the transaction unexpectedly succeeds.
///
/// # Behavior
/// Logs a custom "expected failure" message and always returns a result (never panics).
///
/// # Example
/// 
#[allow(dead_code)]
pub async fn send_ix_expect_failure(
    client: &mut BanksClient,
    payer: &Keypair,
    signers: &[&Keypair],
    ix: Instruction,
    blockhash: Hash,
) -> Result<(), BanksClientError> {
    // Build and sign the transaction
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(signers, blockhash);

    // Submit and handle result
    match client.process_transaction(tx).await {
        Ok(_) => {
            // Transaction succeeded but was expected to fail
            eprintln!("❌ Expected failure but transaction succeeded");
            Err(BanksClientError::TransactionError(TransactionError::InstructionError(
                0,
                solana_sdk::instruction::InstructionError::Custom(9999), // Dummy custom error
            )))
        },
        Err(e) => {
            // Failure was expected and occurred
            println!("😎 Transaction failed as expected: {:?}", e);
            Ok(())
        }
    }
}


/// Registers and pre-funds test accounts in the simulated local blockchain.
///
/// This utility function is used during the setup of integration tests with 
/// Solana's ProgramTest framework. It allows test developers to inject a list 
/// of Keypair accounts into the test ledger with a specified balance of lamports. 
///
/// These accounts can then be used in transactions, instruction signing, or as PDAs
/// (Program Derived Addresses) during smart contract execution.
///
/// # Parameters
///
/// - program_test: A mutable reference to the ProgramTest instance. This object
///   holds the simulated blockchain state and is responsible for initializing the test environment.
///
/// - accounts: A slice of references to Keypair instances. Each keypair represents
///   a test account to be added to the simulated environment.
///
/// - lamports: The number of lamports to pre-fund each account with. This should be 
///   a sufficiently large value to cover rent-exemption and transaction fees during testing.
///
/// - owner: The owner program ID to associate with each account. In most cases, this 
///   is system_program::ID for externally controlled wallets, or the program’s ID
///   for program-owned PDAs.
///
/// # Example
///
/// 
/// let payer = Keypair::new();
/// let user = Keypair::new();
/// fund_test_accounts(&mut program_test, &[&payer, &user], 10_000_000_000, system_program::ID);
///
///
/// # Notes
/// - This function **must** be called **before** .start().await on the ProgramTest,
///   otherwise the injected accounts won't be registered in the simulated environment.
/// - If you plan to derive PDAs or perform CPI (cross-program invocations), ensure that 
///   the accounts added are properly initialized and owned by the expected program ID.
///
/// # Use Cases
/// - Funding a payer and caller wallet for instruction signing and execution.
/// - Pre-loading dummy accounts with lamports to simulate transfers, staking, or vesting flows.
/// - Creating throwaway accounts for stress testing and invalid state reproduction.
#[allow(dead_code)]
pub fn fund_test_accounts(
    program_test: &mut ProgramTest,
    accounts: &[&Keypair],
    lamports: u64,
    owner: Pubkey,
) {
    for key in accounts {
        program_test.add_account(
            key.pubkey(),
            Account {
                lamports,
                data: vec![],
                owner,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    program_test.set_compute_max_units(600_000); 
}


/// Fund accounts manually after ProgramTest has started
#[allow(dead_code)]
pub async fn fund_test_accounts_manual(
    banks_client: &mut BanksClient,
    accounts: &[&Keypair],
    lamports: u64,
    payer: &Keypair,
    recent_blockhash: &solana_sdk::hash::Hash,
) {
    for account in accounts {
        let transfer_ix = system_instruction::transfer(
            &payer.pubkey(),
            &account.pubkey(),
            lamports,
        );

        let tx = Transaction::new_signed_with_payer(
            &[transfer_ix],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        );

        banks_client.process_transaction(tx).await.unwrap();
    }
}

/// Constructs a Solana Instruction from Anchor-compatible accounts and instruction data.
///
/// This utility simplifies the creation of Solana instructions when using the Anchor framework,
/// allowing test writers or program clients to easily build valid instructions by providing:
///
/// - An accounts struct implementing ToAccountMetas, typically the result of a #[derive(Accounts)] context.
/// - A variant implementing InstructionData, usually one of the program's enum instructions.
/// - The program ID to which the instruction will be sent.
///
/// This is particularly useful when writing integration tests or off-chain clients that need to
/// interact with an Anchor-based program without duplicating low-level boilerplate.
///
/// # Parameters
///
/// - accounts: The Anchor context struct (e.g., MyInstructionContext) implementing ToAccountMetas.
/// - instruction_data: The instruction variant (e.g., MyInstruction { amount: 100 }) implementing InstructionData.
/// - program_id: The Pubkey of the on-chain program to which this instruction should be sent.
///
/// # Returns
/// A solana_sdk::instruction::Instruction ready to be used in a transaction.
///
/// # Example
/// 

/// let ix = build_instruction(
///     MyContext { ... },
///     my_program::instruction::MyInstruction { value: 42 },
///     my_program::ID,
/// );
///
#[allow(dead_code)]
pub fn build_instruction<A: anchor_lang::ToAccountMetas, D: anchor_lang::InstructionData>(
    accounts: A,
    instruction_data: D,
    program_id: Pubkey,
) -> Instruction {
    Instruction {
        program_id,
        accounts: accounts.to_account_metas(None),
        data: instruction_data.data(),
    }
}


/// A utility struct representing the full test environment for the Soccial Token smart contract.
///
/// This struct is returned by setup_test_env and encapsulates everything needed to interact
/// with the test blockchain instance during integration tests:
///
/// - banks_client: A client for submitting and reading transactions in the simulated blockchain.
/// - payer: The keypair responsible for paying transaction fees (typically the test admin).
/// - caller: The user executing instructions (can be the owner or a normal user).
/// - recent_blockhash: The latest blockhash, required to sign valid transactions.
/// - program_id: The deployed program's ID for the current test context.
///
/// This struct enables convenient reuse of test components and centralized access to key elements
/// required for building and executing instructions.
///
/// # Typical Usage
/// 

/// let env = setup_test_env(false).await.unwrap();
/// let client = &mut env.banks_client;
/// let payer = &env.payer;
/// let caller = &env.caller;
///


///
/// # Notes
/// - You should always update the recent_blockhash using get_latest_blockhash()
///   before submitting a transaction.
/// - This struct is primarily used in tests but can also be adapted for simulation/debugging tools.
#[allow(dead_code)]
pub struct TestEnv {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub caller: Keypair,
    pub recent_blockhash: Hash,
    pub program_id: Pubkey,
}

/// Initializes the full Solana test environment for the Soccial Token program,
/// including test accounts, program deployment, and optional state initialization.
///
/// This helper function is used to set up integration tests using solana-program-test
/// with minimal boilerplate. It performs the following steps:
///
/// 1. Registers the soccial_token program using ProgramTest.
/// 2. Creates a payer and a caller Keypair (representing the admin and user).
/// 3. Funds the test accounts with enough lamports for rent and transaction fees.
/// 4. Starts the test validator (banks_client) and fetches the latest blockhash.
/// 5. Optionally runs the initialize_token_env function to deploy all core PDAs.
///
/// # Arguments
/// - skip_init: If true, skips initialization (initialize_token_env). Useful for custom/manual flows.
///
/// # Returns
/// A TestEnv struct containing:
/// - banks_client: The program test client.
/// - payer: The fee payer keypair.
/// - caller: The main actor (usually the owner).
/// - recent_blockhash: The latest blockhash to sign transactions.
/// - program_id: The program ID of soccial_token.
///
/// # Example
/// 
/// let env = setup_test_env(false).await.unwrap();
/// // Use env.banks_client, env.payer, env.caller...
///


///
/// # Notes
/// - Always update recent_blockhash before signing new transactions if they take long to build.
/// - Call initialize_token_env manually later if skip_init is true.
#[allow(dead_code)]
pub async fn setup_test_env(
    skip_init: bool, // If true, skips initialize_token_env()
) -> Result<TestEnv, TransportError> {
    let program_id = soccial_token::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);



    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(
        &mut program_test,
        &[&payer, &caller],
        10_000_000_000,
        system_program::ID,
    );
    
    //let seeds = derive_seeds(&program_id, &caller.pubkey());
    // Mint Authority
    //program_test.add_account(seeds.mint_authority, Account::default());
    //program_test.add_account(seeds.offchain_reserve_vault, Account::default());
    //program_test.add_account(seeds.revenue_vault, Account::default());
    //program_test.add_account(seeds.rewards_vault, Account::default());

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    if !skip_init {
        initialize_token_env(program_id, &mut banks_client, &payer, &caller, recent_blockhash).await?;
    }

    Ok(TestEnv {
        banks_client,
        payer,
        caller,
        recent_blockhash,
        program_id,
    })
}


/// Constructs a Solana instruction using an Anchor-style context and instruction data.
///
/// This utility provides a cleaner and type-safe way of constructing instructions
/// by leveraging Anchor's ToAccountMetas and InstructionData traits.
///
/// - ToAccountMetas handles automatic conversion of account contexts into AccountMeta lists.
/// - InstructionData serializes the instruction variant into its binary form.
///
/// This is extremely useful for tests and clients that want to avoid manual instruction
/// construction logic, and instead use their Anchor-generated context and instruction types.
///
/// # Arguments
/// - program_id: The ID of the program to invoke.
/// - ctx: A struct that implements ToAccountMetas (usually generated by #[derive(Accounts)]).
/// - ix_data: A variant implementing InstructionData (usually from your instruction mod).
///
/// # Returns
/// A valid Solana Instruction ready to be sent in a transaction.
///
/// # Example
/// 
/// let instruction = anchor_ix(
///     my_program::ID,
///     accounts_struct,
///     my_program::instruction::MyInstruction { amount: 10 },
/// );
///


#[allow(dead_code)]
pub fn anchor_ix<C: ToAccountMetas, I: InstructionData>(
    program_id: Pubkey,
    ctx: C,
    ix_data: I,
) -> Instruction {
    Instruction {
        program_id,
        accounts: ctx.to_account_metas(None),
        data: ix_data.data(),
    }
}

/// Retrieves the current on-chain Unix timestamp from the Clock sysvar.
///
/// This function should be used instead of Clock::get() when working inside
/// Solana program tests (solana-program-test), as Clock::get() requires
/// an active invoke_context, which is not available outside instruction handlers.
///
/// # Arguments
/// * banks_client - The active BanksClient used to fetch the sysvar.
///
/// # Returns
/// * i64 - The current Unix timestamp according to the simulated Solana runtime.
///
/// # Example
/// 

/// let now = get_current_timestamp(&mut banks_client).await;
/// let future_time = now + 3600; // one hour in the future
///


#[allow(dead_code)]
pub async fn get_current_timestamp(banks_client: &mut BanksClient) -> i64 {
    let clock = banks_client.get_sysvar::<Clock>().await.unwrap();
    clock.unix_timestamp
}

//// Asserts that the result failed with the given custom Anchor error code.
/// Usage: assert_custom_error(result, UserErrorCode::NotOwner, "Expected NotOwner error");
#[allow(dead_code)]
pub fn assert_custom_error<E: Into<u32> + core::fmt::Debug>(
    result: Result<(), TransportError>,
    expected_error: E,
    fail_msg: &str,
) {
    let expected_code = expected_error.into();

    match result {
        Err(TransportError::TransactionError(TransactionError::InstructionError(
            _,
            InstructionError::Custom(code),
        ))) => {
            if code != expected_code {
                panic!(
                    "🚨 {} – Failed as expected, but with wrong custom error code (expected {}, got {})",
                    fail_msg, expected_code, code
                );
            }
            // Success case: failed with correct error code
            println!("😎 Transaction failed as expected");
        }

        Err(TransportError::TransactionError(e)) => {
            panic!(
                "🚨 {} – Failed with unexpected transaction error: {:?}",
                fail_msg, e
            );
        }

        Err(e) => {
            panic!(
                "🚨 {} – Unexpected error (not a transaction error): {:?}",
                fail_msg, e
            );
        }

        Ok(()) => {
            panic!(
                "🚨 {} – Passed but was expected to fail with custom error {}",
                fail_msg, expected_code
            );
        }
    }
}
