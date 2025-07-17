
// ===========================================================================
// Vesting Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module manages the full lifecycle of vesting schedules in the Soccial Token
// economy. It allows secure, rule-based token releases over time with support for:
//
// - Custom cliff periods
// - Linear or cyclical vesting
// - Initial unlocked allocations
// - Mutability toggling (e.g., making schedules immutable)
// - Safe cancellation with automatic refund of unreleased tokens
//
// All schedules are tied to PDAs based on participant pubkey and vesting_id,
// ensuring uniqueness and cryptographic security.
//
// ---------------------------------------------------------------------------
// Core Functions:
// - `create_vesting_schedule()` – Allocates and initializes a new vesting schedule
// - `set_immutable()` – Marks a vesting schedule as permanent (non-editable)
// - `update_vesting_schedule()` – Modifies schedule parameters or adjusts token amounts
// - `cancel_vesting_schedule()` – Cancels a schedule and refunds remaining tokens
//
// ---------------------------------------------------------------------------
// Security:
// - Enforces PDA address validation for all vesting accounts
// - Validates vault balances before transfer
// - Prevents mutation or cancellation once marked immutable
// - All transfers use CPI with signer seeds from PDAs
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;

use crate::vesting::context::{ManageVesting, EditVestingSchedule, ImmutableVestingSchedule};
use anchor_spl::token::{transfer, Transfer};
use crate::economy::TOTAL_SUPPLY;
use crate::utils::error::ErrorCode;
use crate::vesting::VestingErrorCode;

#[event]
pub struct VestingScheduleCreated {
    pub participant: Pubkey,
    pub vesting_id: u64,
    pub total_tokens: u64,
    pub start_time: i64,
    pub cliff_duration: i64,
    pub vesting_duration: i64,
    pub immutable: bool,
}

#[event]
pub struct VestingMadeImmutable {
    pub vesting_id: u64,
    pub participant: Pubkey,
}

#[event]
pub struct VestingScheduleUpdated {
    pub participant: Pubkey,
    pub vesting_id: u64,
    pub total_tokens: u64,
    pub immutable: bool,
}

#[event]
pub struct VestingScheduleCancelled {
    pub participant: Pubkey,
    pub vesting_id: u64,
    pub refunded_tokens: u64,
}



/// ===========================================================================
/// create_vesting_schedule
/// ---------------------------------------------------------------------------
/// Creates a new vesting schedule and allocates tokens for controlled release
///
/// ## Behavior:
/// - Assigns a unique `vesting_id` (incremented from `vesting_state`)
/// - Transfers tokens from the `liquidity_vault` into the `vesting_vault`
/// - Stores release logic: cliff, cycles, vesting duration, initial unlock
///
/// ## Requirements:
/// - The vesting account must be uninitialized (`status == 0`)
/// - Total tokens must be > 0 and ≤ TOTAL_SUPPLY
/// - PDA must match derived key (participant + vesting_id)
///
/// ## Notes:
/// - Time parameters are expressed in seconds
/// - `immutable = true` disables future edits or cancellation
///
/// ## Errors:
/// - AlreadyInitialized: Vesting account was previously used
/// - Unauthorized: If PDA mismatch
/// - InvalidArgument: If token values or durations are inconsistent
/// ===========================================================================
pub(crate) fn create_vesting_schedule(
    ctx: &mut Context<ManageVesting>,
    participant: Pubkey,
    start_time: i64,
    cliff_duration: i64,
    cycles: i64,
    vesting_duration: i64,
    initial_tokens: u64,
    total_tokens: u64,
    immutable: bool,
) -> Result<()> {
    let schedule = &mut ctx.accounts.vesting_schedule;
    let vesting_state = &mut ctx.accounts.vesting_state;

    require!(schedule.status == 0, VestingErrorCode::AlreadyInitialized);

    let vesting_id = vesting_state.last_id;
    let expected_vesting_key = Pubkey::find_program_address(
        &[b"vesting_schedule", participant.as_ref(), &vesting_id.to_le_bytes()],
        ctx.program_id,
    ).0;

    require!(schedule.key() == expected_vesting_key, ErrorCode::Unauthorized);
    require!(start_time >= 0, VestingErrorCode::InvalidStartTime);
    require!(cliff_duration >= 0, VestingErrorCode::InvalidCliff);
    require!(vesting_duration > 0, VestingErrorCode::InvalidVestingDuration);
    require!(total_tokens > 0, VestingErrorCode::InvalidTokenAmount);
    require!(total_tokens <= TOTAL_SUPPLY, ErrorCode::InvalidArgument);

    // Ensure liquidity_vault_token_account has enough tokens
    let liquidity_vault_seeds: &[&[u8]] = &[
        b"liquidity_vault",
        &[ctx.bumps.liquidity_vault],
    ];
    let signer = &[liquidity_vault_seeds];

    let cpi_accounts = Transfer {
        from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
        to: ctx.accounts.vesting_vault_token_account.to_account_info(),
        authority: ctx.accounts.liquidity_vault.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer,
    );

    transfer(cpi_ctx, total_tokens)?;

    // Initialize the vesting schedule
    schedule.participant = participant;
    schedule.start_time = start_time;
    schedule.cliff_duration = cliff_duration;
    schedule.cycles = cycles;
    schedule.vesting_duration = vesting_duration;
    schedule.initial_tokens = initial_tokens;
    schedule.total_tokens = total_tokens;
    schedule.released_tokens = 0;
    schedule.immutable = immutable;
    schedule.last_claim_time = start_time;
    schedule.vesting_id = vesting_id;
    schedule.status = 1;

    vesting_state.last_id += 1;

    msg!("🕒 Created vesting schedule for participant {} with vesting ID {}", participant, vesting_id);

    emit!(VestingScheduleCreated {
        participant,
        vesting_id,
        total_tokens,
        start_time,
        cliff_duration,
        vesting_duration,
        immutable,
    });

    Ok(())
}


/// ===========================================================================
/// set_immutable
/// ---------------------------------------------------------------------------
/// Locks an active vesting schedule to make it immutable
///
/// ## Behavior:
/// - Sets `immutable = true` to prevent future updates or cancellation
/// - Validates that the vesting_id matches the expected value
///
/// ## Constraints:
/// - Cannot be called on an already-immutable schedule
/// - Schedule must match the provided `vesting_id`
///
/// ## Errors:
/// - VestingScheduleIsImmutable: If already locked
/// - Unauthorized: If vesting_id mismatch
/// ===========================================================================
pub(crate) fn set_immutable(
    ctx: &mut Context<ImmutableVestingSchedule>,
    vesting_id: u64,
) -> Result<()> {

     let schedule = &mut ctx.accounts.vesting_schedule;

    // If already immutable, no need to proceed
    require!(
        !schedule.immutable,
        VestingErrorCode::VestingScheduleIsImmutable
    );

    // Ensure that the vesting schedule matches the provided vesting_id
    require!(
        schedule.vesting_id == vesting_id,
        ErrorCode::Unauthorized
    );
    
    schedule.immutable = true;

    msg!(
    "🛑 Vesting schedule with ID {} is now locked and cannot be modified.", vesting_id);

    emit!(VestingMadeImmutable {
        vesting_id,
        participant: schedule.participant,
    });

    Ok(())
}


/// ===========================================================================
/// update_vesting_schedule
/// ---------------------------------------------------------------------------
/// Modifies a vesting schedule’s timing and allocation
///
/// ## Behavior:
/// - Updates cliff, cycles, vesting duration, initial/unlocked tokens
/// - Adjusts token balances between liquidity and vesting vaults
///
/// ## Transfer Logic:
/// - If new total_tokens > old: transfer the delta into the vesting vault
/// - If new total_tokens < old: return the surplus to liquidity vault
///
/// ## Constraints:
/// - Schedule must be active and mutable
/// - Duration values must be valid and consistent
///
/// ## Errors:
/// - Unauthorized: If vesting_id mismatch
/// - InvalidArgument: If invalid durations or amounts
/// - InsufficientFunds: If liquidity vault lacks needed tokens
/// - VestingScheduleIsImmutable: If locked
/// ===========================================================================
pub(crate) fn update_vesting_schedule(
    ctx: &mut Context<EditVestingSchedule>,
    vesting_id: u64,
    start_time: i64,
    cliff_duration: i64,
    cycles: i64,
    vesting_duration: i64,
    initial_tokens: u64,
    total_tokens: u64,
    immutable: bool
) -> Result<()> {
    let schedule = &mut ctx.accounts.vesting_schedule;

    require!(!schedule.immutable, VestingErrorCode::VestingScheduleIsImmutable);
    require!(schedule.vesting_id == vesting_id, ErrorCode::Unauthorized);
    require!(vesting_duration > 0, ErrorCode::InvalidArgument);
    require!(cliff_duration >= 0, ErrorCode::InvalidArgument);
    require!(cliff_duration <= vesting_duration, ErrorCode::InvalidArgument);
    require!(schedule.status == 1, VestingErrorCode::VestingNotActive);

    let old_total = schedule.total_tokens;
    let new_total = total_tokens;

    
    if new_total > old_total {
        let diff = new_total - old_total;
        let balance = ctx.accounts.liquidity_vault_token_account.amount;
        require!(balance >= diff, VestingErrorCode::InsufficientFunds);

        let seeds: &[&[u8]] = &[b"liquidity_vault", &[ctx.bumps.liquidity_vault]];
        let signer = &[seeds];

        let cpi_accounts = Transfer {
            from: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            to: ctx.accounts.vesting_vault_token_account.to_account_info(),
            authority: ctx.accounts.liquidity_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        transfer(cpi_ctx, diff)?;
        msg!("📤 Transferred {} tokens from Liquidity Vault to Vesting Vault (increase)", diff);
        
    } else if new_total < old_total {
        let diff = old_total - new_total;

        let seeds: &[&[u8]] = &[b"vesting_vault", &[ctx.bumps.vesting_vault]];
        let signer = &[seeds];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vesting_vault_token_account.to_account_info(),
            to: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            authority: ctx.accounts.vesting_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        transfer(cpi_ctx, diff)?;
        msg!("📥 Returned {} tokens from Vesting Vault to Liquidity Vault (decrease)", diff);
    }

    // Update data
    schedule.start_time = start_time;
    schedule.cliff_duration = cliff_duration;
    schedule.cycles = cycles;
    schedule.vesting_duration = vesting_duration;
    schedule.initial_tokens = initial_tokens;
    schedule.total_tokens = total_tokens;
    schedule.immutable = immutable;

    msg!("✏️ Vesting schedule {} updated for participant {}", vesting_id, schedule.participant);

    emit!(VestingScheduleUpdated {
        participant: schedule.participant,
        vesting_id,
        total_tokens,
        immutable,
    });


    Ok(())
}


/// ===========================================================================
/// cancel_vesting_schedule
/// ---------------------------------------------------------------------------
/// Cancels a vesting schedule and returns unreleased tokens
///
/// ## Behavior:
/// - Transfers unreleased tokens from the `vesting_vault` back to the `liquidity_vault`
/// - Marks schedule status as `3` (cancelled)
///
/// ## Constraints:
/// - Schedule must be active (`status == 1`)
/// - Schedule must be mutable (`immutable = false`)
///
/// ## Errors:
/// - VestingScheduleIsImmutable: If already locked
/// - Unauthorized: If vesting_id mismatch
/// - AlreadyCancelled: If already cancelled
/// - VestingNotActive: If status is not 1
/// ===========================================================================
pub(crate) fn cancel_vesting_schedule(
    ctx: &mut Context<EditVestingSchedule>,
    vesting_id: u64,
) -> Result<()> {
    let schedule = &mut ctx.accounts.vesting_schedule;


     // Prevent editing if the schedule is marked as immutable
    require!(
        !schedule.immutable,
        VestingErrorCode::VestingScheduleIsImmutable
    );
    
    // Ensure that the vesting schedule matches the provided vesting_id
    require!(
        schedule.vesting_id == vesting_id,
        ErrorCode::Unauthorized
    );

    // Check if it's already cancelled
    require!(
        schedule.status != 3,
        VestingErrorCode::AlreadyCancelled
    );

    // Ensure it is currently active before cancelling
    require!(
        schedule.status == 1,
        VestingErrorCode::VestingNotActive
    );

    let remaining = schedule.total_tokens - schedule.released_tokens;

    if remaining > 0 {
        let seeds: &[&[u8]] = &[b"vesting_vault", &[ctx.bumps.vesting_vault]];
        let signer = &[seeds];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vesting_vault_token_account.to_account_info(),
            to: ctx.accounts.liquidity_vault_token_account.to_account_info(),
            authority: ctx.accounts.vesting_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer,
        );

        transfer(cpi_ctx, remaining)?;
        msg!("🔁 Refunded {} tokens from Vesting Vault to Liquidity Vault", remaining);
    }

    // Set the status to cancelled
    schedule.status = 3; // Cancelled

    msg!(
        "🛑 Vesting schedule with ID {} for participant {} has been cancelled.",
        vesting_id,
        schedule.participant
    );

    emit!(VestingScheduleCancelled {
        participant: schedule.participant,
        vesting_id,
        refunded_tokens: remaining,
    });

    Ok(())
}