// ===========================================================================
// Soccial Token – Staking Configuration & State
// ---------------------------------------------------------------------------
//
// This module defines the **staking system** for the Soccial Token (SCTK), including:
// - Global staking state with available plans (`StakingState`)
// - Individual user staking records (`StakingAccount`)
// - Dynamic staking plan management (add, update, deactivate)
//
// Each plan supports lock-up periods, APR configurations, and user-specific entries.
//
// ---------------------------------------------------------------------------
// Features:
// - Supports up to **8 simultaneous plans** (e.g., 30, 90, 180 days, etc.)
// - Compact layout for efficient on-chain storage
// - Built-in validation and protection against overflows & duplicates
//
// ---------------------------------------------------------------------------
// Layout:
// - `StakingPlan`: Plan ID, duration, APR, status
// - `StakingState`: All available plans + counters
// - `StakingAccount`: Individual stake details
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::{staking::StakingErrorCode, utils::error::ErrorCode};

#[event]
pub struct StakingPlanCreated {
    pub plan_id: u8,
    pub lockup_duration: i64,
    pub apr_bps: u16,
}

#[event]
pub struct StakingPlanUpdated {
    pub plan_id: u8,
    pub new_lockup_duration: i64,
    pub new_apr_bps: u16,
}

#[event]
pub struct StakingPlanDeactivated {
    pub plan_id: u8,
}


/// Configuration for a staking plan (e.g., 30, 90, 180 dias)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub struct StakingPlan {
    /// Unique identifier of the plan (used to link in StakingAccount).
    pub plan_id: u8,

    /// Duration of the lock-up period in seconds.
    pub lockup_duration: i64,

    /// Annual Percentage Rate (APR) in basis points (e.g., 800 = 8.00%).
    pub apr_bps: u16,

    /// Whether this plan is currently active.
    pub active: bool,
}

/// Global staking state and configurable plans
#[account]
pub struct StakingState {
    /// The total number of staking entries created across all users.
    pub total_stakes: u64,

    /// The last used stake ID (used for PDA derivation).
    pub last_id: u64,

    /// List of available staking plans (fixed-size array for deterministic size).
    pub plans: [StakingPlan; 8], // suporta até 8 planos
}

/// ===========================================================================
/// Returns an active staking plan by its `plan_id`, or `None` if not found.
///
/// ## Parameters:
/// - `plan_id`: ID of the plan to retrieve.
///
/// ## Returns:
/// - `Some(StakingPlan)` if active and exists
/// - `None` otherwise
/// ===========================================================================
impl StakingState {
    pub const LEN: usize =
        8    // Anchor discriminator
        + 8  // total_stakes: u64
        + 8  // last_id: u64
        + (8 * 16); // 8 planos × 16 bytes (padded StakingPlan)
    
    /// Returns the staking plan by ID (if active)
    pub fn get_plan(&self, plan_id: u8) -> Option<StakingPlan> {
        self.plans.iter().find(|p| p.plan_id == plan_id && p.active).copied()
    }

    /// Adds a new staking plan to the system.
    ///
    /// ## Rules:
    /// - Rejects if a plan with the same ID already exists and is active
    /// - Replaces an inactive plan with same ID if it exists
    /// - Uses the first inactive slot if available
    /// - Fails if no space is available (max 8)
    ///
    /// ## Errors:
    /// - `InvalidArgument` if APR or lockup is invalid
    /// - `PlanAlreadyExists` if active duplicate exists
    /// - `TooManyPlans` if no slot is available
    pub fn add_plan(&mut self, new_plan: StakingPlan) -> Result<()> {
        require!(new_plan.lockup_duration > 0, ErrorCode::InvalidArgument);
        require!(new_plan.apr_bps > 0, ErrorCode::InvalidArgument);

        // Reject if an active plan with the same ID already exists
        require!(
            !self.plans.iter().any(|p| p.plan_id == new_plan.plan_id && p.active),
            StakingErrorCode::PlanAlreadyExists
        );

        // Replace if an inactive plan with the same ID is found
        if let Some(slot) = self.plans.iter_mut().find(|p| p.plan_id == new_plan.plan_id && !p.active) {
            *slot = new_plan;

            msg!(
                "🆕 Added staking plan → ID: {} | Lockup: {}s | APR: {} bps",
                new_plan.plan_id,
                new_plan.lockup_duration,
                new_plan.apr_bps
            );

            emit!(StakingPlanCreated {
                plan_id: new_plan.plan_id,
                lockup_duration: new_plan.lockup_duration,
                apr_bps: new_plan.apr_bps,
            });

            return Ok(());
        }

        // Use the first available inactive slot
        if let Some(slot) = self.plans.iter_mut().find(|p| !p.active) {
            *slot = new_plan;

            msg!(
                "🆕 Added staking plan → ID: {} | Lockup: {}s | APR: {} bps",
                new_plan.plan_id,
                new_plan.lockup_duration,
                new_plan.apr_bps
            );

            emit!(StakingPlanCreated {
                plan_id: new_plan.plan_id,
                lockup_duration: new_plan.lockup_duration,
                apr_bps: new_plan.apr_bps,
            });

            return Ok(());
        }

        // No available slot for new plan
        err!(StakingErrorCode::TooManyPlans)
    }


    /// Updates an existing plan’s lockup duration and APR.
    ///
    /// ## Parameters:
    /// - `plan_id`: Plan to modify
    /// - `new_lockup`: New lockup time (seconds)
    /// - `new_apr_bps`: New APR in basis points
    ///
    /// ## Errors:
    /// - `InvalidArgument` if values are invalid
    /// - `PlanNotFound` if plan does not exist or is inactive
    pub fn update_plan(&mut self, plan_id: u8, new_lockup: i64, new_apr_bps: u16) -> Result<()> {
        require!(new_lockup > 0, ErrorCode::InvalidArgument);
        require!(new_apr_bps > 0, ErrorCode::InvalidArgument);

        let plan = self.plans.iter_mut().find(|p| p.plan_id == plan_id && p.active)
            .ok_or(StakingErrorCode::PlanNotFound)?;

        plan.lockup_duration = new_lockup;
        plan.apr_bps = new_apr_bps;


        msg!(
            "✏️ Updated staking plan → ID: {} | New Lockup: {}s | New APR: {} bps",
            plan_id,
            new_lockup,
            new_apr_bps
        );

        emit!(StakingPlanUpdated {
            plan_id,
            new_lockup_duration: new_lockup,
            new_apr_bps: new_apr_bps,
        });

        Ok(())
    }

    /// Deactivates a staking plan (soft delete).
    ///
    /// ## Parameters:
    /// - `plan_id`: Plan to deactivate
    ///
    /// ## Logic:
    /// - Marks `active = false` without removing it
    ///
    /// ## Errors:
    /// - `PlanNotFound` if plan does not exist
    pub fn deactivate_plan(&mut self, plan_id: u8) -> Result<()> {
        let plan = self.plans.iter_mut().find(|p| p.plan_id == plan_id)
            .ok_or(StakingErrorCode::PlanNotFound)?;

        require!(plan.active, StakingErrorCode::PlanNotFound);

        plan.active = false;

        msg!("🚫 Deactivated staking plan → ID: {}", plan_id);

        emit!(StakingPlanDeactivated {
            plan_id
        });

        Ok(())
    }


}


/// Represents a single staking entry from a user.
///
/// Each participant can have multiple stakes across different plans.
/// Each stake is linked to a specific staking plan by `plan_id`.
#[account]
pub struct StakingAccount {
     /// The participant (user) who staked tokens.
    pub participant: Pubkey,

    /// Unique identifier for the stake (used for PDA).
    pub stake_id: u64,

    /// Timestamp of the **current active cycle** (updated after each renewal or claim).
    pub start_time: i64,

    /// Duration of each lock-up period (in seconds), copied from plan.
    pub lockup_duration: i64,

    /// Annual Percentage Rate (APR) in basis points (bps), copied from plan.
    pub apr_bps: u16,

    /// Total amount of tokens staked (principal only).
    pub staked_tokens: u64,

    /// Whether the stake has been withdrawn (irreversible).
    pub withdrawn: bool,

    /// Identifier of the staking plan.
    pub plan_id: u8,

    /// Total rewards claimed so far across all cycles.
    pub total_rewards_claimed: u64,

    /// Total number of completed reward cycles (for analytics or UI).
    pub cycles_completed: u16,

}

impl StakingAccount {
    /// Byte size required to allocate the StakingAccount account.
    ///
    /// Layout breakdown:
    pub const LEN: usize =
        8  // Anchor discriminator
        + 32 // participant
        + 8  // stake_id
        + 8  // start_time
        + 8  // lockup_duration
        + 2  // apr_bps
        + 8  // staked_tokens
        + 1  // withdrawn
        + 1  // plan_id
        + 8  // total_rewards_claimed
        + 2  // cycles_completed
        + 6; // padding to align to 8 bytes (total = 92)
}
