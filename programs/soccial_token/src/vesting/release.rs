// ===========================================================================
// Vesting Release Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles the secure release of vested tokens to participants,
// enforcing all vesting rules, time-based checks, and associated account validation.
//
// ---------------------------------------------------------------------------
// Core Responsibilities:
// - Validates PDA and vesting ID for safety
// - Computes claimable tokens based on cliff and schedule
// - Verifies destination account is owned by the participant
// - Transfers only the unreleased portion using CPI with PDA signer
//
// ---------------------------------------------------------------------------
// Security:
// - Release can only go to the participant’s ATA
// - All inputs are verified against derived PDAs
// - Immutable schedules and lock status respected
//
// ---------------------------------------------------------------------------
// Function:
// - `release_vested_tokens()` – Allows claiming vested tokens once unlocked
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    vesting::context::*,
    vesting::VestingErrorCode,
    utils::error::ErrorCode,
};

#[event]
pub struct VestedTokensReleased {
    pub participant: Pubkey,
    pub vesting_id: u64,
    pub amount: u64,
    pub timestamp: i64,
    pub fully_claimed: bool,
}


/// ===========================================================================
/// release_vested_tokens
/// ---------------------------------------------------------------------------
/// Releases vested tokens to the participant’s associated token account (ATA).
///
/// ## Behavior:
/// - Verifies the vesting schedule matches the expected PDA
/// - Calculates how many tokens have vested but are still unclaimed
/// - Ensures release is only made to the participant’s ATA
/// - Transfers only the unreleased amount using CPI from the vesting vault
///
/// ## Permissions:
/// - Can be called by the participant or by an admin with `"manage_vesting"`
///
/// ## Flow:
/// 1. Validate PDA address (vesting_schedule derived from participant + ID)
/// 2. Confirm vesting is active and releaseable
/// 3. Compute amount still eligible for release
/// 4. Validate that destination ATA is correct
/// 5. Update schedule state and transfer tokens
///
/// ## Constraints:
/// - Tokens can only be released to the participant’s ATA
/// - Only unreleased vested amounts are claimable
///
/// ## Errors:
/// - `Unauthorized`: If PDA or ATA do not match
/// - `VestingNotActive`: If the schedule is not active
/// - `NoTokensToRelease`: If there's nothing new to claim
/// ===========================================================================
pub(crate) fn release_vested_tokens(
    ctx: &mut Context<ReleaseVestedTokens>,
) -> Result<()> {

    let schedule = &mut ctx.accounts.vesting_schedule;

    let clock = &ctx.accounts.clock;

    let participant = schedule.participant;

    // ------------------------------------------------------------------
    // Step 1: Validate vesting_id and PDA address
    // ------------------------------------------------------------------

    let (expected_pda, _) = Pubkey::find_program_address(
        &[
            b"vesting_schedule",
            participant.as_ref(),
            &schedule.vesting_id.to_le_bytes(),
        ],
        ctx.program_id,
    );

    require_keys_eq!(schedule.key(), expected_pda, ErrorCode::Unauthorized);

    // ------------------------------------------------------------------
    // Step 2: Check vesting status and calculate vested tokens
    // ------------------------------------------------------------------

    require_eq!(schedule.status, 1, VestingErrorCode::VestingNotActive);

    let vested = schedule.calculate_vested_amount(clock.unix_timestamp);
    let already_released = schedule.released_tokens;

    require!(vested > already_released, VestingErrorCode::NoTokensToRelease);

    let to_release = vested.saturating_sub(already_released);
    require!(to_release > 0, VestingErrorCode::NoTokensToRelease);

    // ------------------------------------------------------------------
    // Step 3: Validate ATA destination
    // ------------------------------------------------------------------
    // We must ensure the tokens will be withdraw only to the participant

    let expected_ata = get_associated_token_address(&participant, &ctx.accounts.mint.key());
    require_keys_eq!(
        ctx.accounts.destination_token_account.key(),
        expected_ata,
        ErrorCode::Unauthorized
    );

    // ------------------------------------------------------------------
    // Step 4: Update state
    // ------------------------------------------------------------------

    schedule.released_tokens += to_release;
    schedule.last_claim_time = clock.unix_timestamp;

    // ------------------------------------------------------------------
    // Step 5: Transfer tokens
    // ------------------------------------------------------------------
    
    let seeds: &[&[u8]] = &[b"vesting_vault", &[ctx.bumps.vesting_vault]];
    let signer: &[&[&[u8]]] = &[seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vesting_vault_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: ctx.accounts.vesting_vault.to_account_info(),
        },
        signer,
    );
    transfer(cpi_ctx, to_release)?;

    msg!(
        "💸 Released {} vested tokens to participant {} (vesting_id: {}).",
        to_release,
        schedule.participant,
        schedule.vesting_id
    );
    
    emit!(VestedTokensReleased {
        participant: schedule.participant,
        vesting_id: schedule.vesting_id,
        amount: to_release,
        timestamp: clock.unix_timestamp,
        fully_claimed: schedule.released_tokens >= schedule.total_tokens,
    });


    // We don't need the account anymore. Let's delete if and recover the remaining lamports
    if schedule.released_tokens >= schedule.total_tokens
    {
        ctx.accounts.vesting_schedule.close(ctx.accounts.recipient_of_lamports.to_account_info())?;
    }

    Ok(())
}
