use anchor_lang::prelude::*;

// ======================================================================
// Soccial Token – Staking Error Definitions
//
// This module defines error codes related to staking operations,
// including invalid plans, reward calculations, lockups, and vault access.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum StakingErrorCode {
    /// The staking period has not yet ended.
    #[msg("Staking period has not yet ended.")]
    StakingPeriodNotOver,

    /// No rewards are available to claim.
    #[msg("No rewards available to claim.")]
    NoRewardsAvailable,

    /// Overflow occurred during reward calculation.
    #[msg("Reward calculation overflow.")]
    RewardOverflow,

    /// Insufficient staked balance to withdraw.
    #[msg("Insufficient staked balance to withdraw.")]
    InsufficientStakedBalance,

    /// The staking plan is not active or doesn't exist.
    #[msg("Invalid or inactive staking plan.")]
    InvalidStakingPlan,

    /// Staking amount must be greater than zero.
    #[msg("Staking amount must be greater than zero.")]
    InvalidStakeAmount,

    /// Failed to find a staking record for the given ID.
    #[msg("Staking record not found.")]
    StakingAccountNotFound,

    /// Cannot edit a plan that is already in use.
    #[msg("Cannot modify an active plan used in existing stakes.")]
    CannotEditActivePlan,

    /// A plan with the specified ID already exists.
    #[msg("Plan with this ID already exists.")]
    PlanAlreadyExists,

    /// Maximum number of staking plans reached.
    #[msg("Maximum number of plans reached.")]
    TooManyPlans,

    /// Staking plan was not found.
    #[msg("Staking plan not found.")]
    PlanNotFound,

    /// Tokens have already been withdrawn from this stake.
    #[msg("Tokens already withdrawn.")]
    AlreadyWithdrawn,

    /// User's wallet does not hold enough tokens to stake.
    #[msg("User does not have enough tokens to stake.")]
    InsufficientUserBalance,

    /// Vault does not have enough tokens to fulfill the stake.
    #[msg("Insufficient funds on vault. Contact us or try again later.")]
    InsufficientVaultBalance,

    /// Arithmetic overflow during reward calculation.
    #[msg("Arithmetic overflow during reward calculation.")]
    Overflow,

    /// Lockup period has not yet ended.
    #[msg("Staking is still locked.")]
    LockupPeriodNotEnded,

}
