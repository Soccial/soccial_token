
// ===========================================================================
// Staking Plan Management for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles the administrative operations related to staking plans,
// including creation, editing, and disabling of available APR lockup options.
//
// ---------------------------------------------------------------------------
// ## Purpose:
// - Enable the addition of new staking plans dynamically
// - Allow updates to APR or lockup durations of existing plans
// - Disable outdated plans without affecting historical staking data
//
// ---------------------------------------------------------------------------
// ## Core Concepts:
// - **StakingPlan**: Defines a unique staking configuration (APR + duration)
// - Each plan is identified by a `plan_id` (u8)
// - Plans are stored in the `StakingState` and indexed by `plan_id`
// - Disabling a plan keeps its record but prevents future use
//
// ---------------------------------------------------------------------------
// ## Available Instructions:
// - `add_staking_plan`: Register a new staking plan
// - `edit_staking_plan`: Update the APR or lockup of a plan
// - `disable_staking_plan`: Deactivate a plan (read-only history remains)
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================


use anchor_lang::prelude::*;
use crate::staking::{context::*, StakingPlan};


/// ===========================================================================
/// Adds a New Staking Plan to the System
///
/// Registers a new staking plan with a lockup duration and APR.
/// New plan becomes immediately active and selectable by users.
///
/// ## Parameters:
/// - `plan_id`: Unique identifier for the new plan
/// - `lockup`: Lockup duration in seconds
/// - `apr_bps`: Annual APR in basis points (1% = 100bps)
///
/// ## Errors:
/// - Delegated to `StakingState::add_plan`
/// ===========================================================================
pub(crate) fn add_staking_plan(ctx: &mut Context<ManageStaking>, plan_id: u8, lockup: i64, apr_bps: u16) -> Result<()> {
    let plan = StakingPlan {
        plan_id,
        lockup_duration: lockup,
        apr_bps,
        active: true,
    };
    ctx.accounts.staking_state.add_plan(plan)
}

/// ===========================================================================
/// Updates Lockup or APR of an Existing Plan
///
/// Modifies the configuration of a staking plan while keeping its ID.
///
/// ## Parameters:
/// - `plan_id`: ID of the staking plan to edit
/// - `lockup`: New lockup duration in seconds
/// - `apr_bps`: New APR in basis points
///
/// ## Errors:
/// - Delegated to `StakingState::update_plan`
/// ===========================================================================
pub(crate) fn edit_staking_plan(ctx: &mut Context<ManageStaking>, plan_id: u8, lockup: i64, apr_bps: u16) -> Result<()> {
    ctx.accounts.staking_state.update_plan(plan_id, lockup, apr_bps)
}

/// ===========================================================================
/// Disables an Existing Staking Plan
///
/// Marks a staking plan as inactive, preventing future use,
/// while preserving its historical record.
///
/// ## Parameters:
/// - `plan_id`: Identifier of the plan to disable
///
/// ## Errors:
/// - Delegated to `StakingState::deactivate_plan`
/// ===========================================================================
pub(crate) fn disable_staking_plan(ctx: &mut Context<ManageStaking>, plan_id: u8) -> Result<()> {
    ctx.accounts.staking_state.deactivate_plan(plan_id)
}
