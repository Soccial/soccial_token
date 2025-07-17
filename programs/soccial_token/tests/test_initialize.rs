// ======================================================================
// Soccial Token – Integration Tests: Account Initialization & TestCall
//
// These integration tests cover the foundational setup of the Soccial Token
// contract. They ensure that all required accounts and PDAs are initialized
// correctly, and that the contract functions can run as expected on top
// of this base environment.
//
// Author: Paulo Rodrigues
// Project: Soccial Token
// Website: https://www.soccial.com/thetoken
// ======================================================================

use anchor_lang::prelude::borsh;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transport::TransportError,
};
use borsh::de::BorshDeserialize;

use soccial_token::{
    initialize::initialize::team_vesting, vesting::VestingSchedule
};
use testutils::basics::*;

mod testutils;

#[tokio::test]
async fn test_initialize_all_accounts_should_succeed() -> Result<(), TransportError> {
    let program_id = soccial_token::ID;
    let mut program_test = ProgramTest::default();
    program_test.add_program("soccial_token", program_id, None);

    let payer = Keypair::new();
    let caller = Keypair::new();

    fund_test_accounts(
        &mut program_test,
        &[&payer, &caller],
        50_000_000_000, // 50 SOL em lamports
        system_program::ID,
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Run full environment initialization
    let seeds = initialize_token_env(program_id, &mut banks_client, &payer, &caller, recent_blockhash)
        .await?;

    assert_all_pdas_exist_from_seeds(&mut banks_client, &seeds).await;
    assert_all_pdas_exist(&mut banks_client, &program_id, &caller.pubkey()).await;

    // Derive vesting schedule PDAs
    let team1 = team_vesting::team1_pubkey();
    let team2 = team_vesting::team2_pubkey();

    let vesting_id_team1: u64 = 1;
    let vesting_id_team2: u64 = 2;

    let (team1_pda, _) = Pubkey::find_program_address(
        &[b"vesting_schedule", team1.as_ref(), &vesting_id_team1.to_le_bytes()],
        &program_id,
    );

    let (team2_pda, _) = Pubkey::find_program_address(
        &[b"vesting_schedule", team2.as_ref(), &vesting_id_team2.to_le_bytes()],
        &program_id,
    );

    // Load and deserialize accounts
    let team1_account = banks_client
        .get_account(team1_pda)
        .await?
        .expect("Team1 vesting schedule should exist");
    let schedule1 = VestingSchedule::deserialize(&mut &team1_account.data[..]).unwrap();

    let team2_account = banks_client
        .get_account(team2_pda)
        .await?
        .expect("Team2 vesting schedule should exist");
    let schedule2 = VestingSchedule::deserialize(&mut &team2_account.data[..]).unwrap();

    // Expected values
    let cliff = team_vesting::TEAM_CLIFF;
    let duration = team_vesting::TEAM_VESTING_DURATION;
    let cycles = team_vesting::TEAM_CYCLES;
    let initial_tokens = team_vesting::INITIAL_TOKENS;
    let total_tokens = schedule1.total_tokens; 
    let current_time = schedule1.start_time;

    // ────────────────────────────────
    // Team 1 assertions
    // ────────────────────────────────
    assert_eq!(schedule1.participant, team1, "Team1 participant mismatch");
    assert_eq!(schedule1.vesting_id, vesting_id_team1, "Team1 vesting ID mismatch");
    assert_eq!(schedule1.cliff_duration, cliff, "Team1 cliff mismatch");
    assert_eq!(schedule1.vesting_duration, duration, "Team1 vesting duration mismatch");
    assert_eq!(schedule1.cycles, cycles, "Team1 cycles mismatch");
    assert_eq!(schedule1.initial_tokens, initial_tokens, "Team1 initial tokens mismatch");
    assert_eq!(schedule1.total_tokens, total_tokens, "Team1 total tokens mismatch");
    assert_eq!(schedule1.released_tokens, 0, "Team1 released tokens should be 0");
    assert_eq!(schedule1.status, 1, "Team1 status should be 1");
    assert!(schedule1.immutable, "Team1 should be immutable");
    assert_eq!(schedule1.last_claim_time, current_time, "Team1 last_claim_time mismatch");
    assert_eq!(schedule1.start_time, current_time, "Team1 start_time mismatch");

    // ────────────────────────────────
    // Team 2 assertions
    // ────────────────────────────────
    assert_eq!(schedule2.participant, team2, "Team2 participant mismatch");
    assert_eq!(schedule2.vesting_id, vesting_id_team2, "Team2 vesting ID mismatch");
    assert_eq!(schedule2.cliff_duration, cliff, "Team2 cliff mismatch");
    assert_eq!(schedule2.vesting_duration, duration, "Team2 vesting duration mismatch");
    assert_eq!(schedule2.cycles, cycles, "Team2 cycles mismatch");
    assert_eq!(schedule2.initial_tokens, initial_tokens, "Team2 initial tokens mismatch");
    assert_eq!(schedule2.total_tokens, total_tokens, "Team2 total tokens mismatch");
    assert_eq!(schedule2.released_tokens, 0, "Team2 released tokens should be 0");
    assert_eq!(schedule2.status, 1, "Team2 status should be 1");
    assert!(schedule2.immutable, "Team2 should be immutable");
    assert_eq!(schedule2.last_claim_time, current_time, "Team2 last_claim_time mismatch");
    assert_eq!(schedule2.start_time, current_time, "Team2 start_time mismatch");

    Ok(())
}
