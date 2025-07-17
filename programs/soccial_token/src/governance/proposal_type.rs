// ===========================================================================
// Governance Proposal Type System – Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module defines the set of valid proposal types that can be submitted
// through the Soccial Token on-chain governance system. Each proposal type
// is mapped to a unique bit (0–31) to enable efficient filtering, validation,
// and bitmask-based access control.
//
// ---------------------------------------------------------------------------
// ## Features:
// - Enum `ProposalTypeBit` with explicit discriminant values
// - Utility methods for name lookup, bit resolution, and conversion
// - Constant `ALL` slice for reflection or iteration
//
// ---------------------------------------------------------------------------
// ## Use Cases:
// - Validating proposal categories on submission
// - Matching access permissions or role-based rights per proposal type
// - Serializing types to human-readable strings
//
// ---------------------------------------------------------------------------
// ## Example:
//   let proposal = ProposalTypeBit::UpdateGovernance;
//   let label = proposal.name(); // → "UpdateGovernance"
//   let mask = proposal.as_bit(); // → 0b0000000001
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;


/// ===========================================================================
/// Enum: ProposalTypeBit
///
/// Defines all supported governance proposal types in the Soccial system.
/// Each type has a fixed `u8` discriminant and is used to categorize
/// proposals and apply fine-grained rules or voting logic.
///
///
/// ## Utilities:
/// - `ProposalTypeBit::ALL`: Static slice of all (label, value) pairs
/// - `ProposalTypeBit::from_str("UpdateRewardRate")` → `Some(UpdateRewardRate)`
/// - `ProposalTypeBit::from_bit(3)` → `Some(UpdateAirdropRate)`
/// - `ProposalTypeBit::name()` → Returns string label
/// - `ProposalTypeBit::as_bit()` → Returns `1 << variant` (bitmask)
///
/// ## Use Case:
/// Enables compact representation of proposal types and consistent
/// validation throughout the governance engine.
///
/// ===========================================================================

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum ProposalTypeBit {
    // --- Governance & Strategic Decisions ---
    UpdateGovernance = 0,
    StrategicDecision,
    SystemUpgrade,
    EmergencyShutdown,

    // --- Tax & Fee Adjustments ---
    AdjustTaxRate,
    UpdateRewardFee,
    UpdateAirdropFee,

    // --- Allocations ---
    TreasuryAllocation,
    RewardsAllocation,
    AidropAllocation,

    // --- Recover Operations ---
    RecoverFromAidrop,
    RecoverFromInsurance,
    RecoverFromStaking,
    RecoverFromVesting,
    
    StakingRecover,
    VestingRecover,

    // --- Insurance & Protection ---
    AllocateInsurance,

    // --- Misc ---
    Custom,
}

impl ProposalTypeBit {
    pub const ALL: &'static [(&'static str, ProposalTypeBit)] = &[
      // Governance & Strategic Decisions
        ("UpdateGovernance", Self::UpdateGovernance),
        ("StrategicDecision", Self::StrategicDecision),
        ("SystemUpgrade", Self::SystemUpgrade),
        ("EmergencyShutdown", Self::EmergencyShutdown),

        // Tax & Fee Adjustments
        ("AdjustTaxRate", Self::AdjustTaxRate),
        ("UpdateRewardFee", Self::UpdateRewardFee),
        ("UpdateAirdropFee", Self::UpdateAirdropFee),

        // Allocations
        ("TreasuryAllocation", Self::TreasuryAllocation),
        ("RewardsAllocation", Self::RewardsAllocation),
        ("AidropAllocation", Self::AidropAllocation),

        // Recover Operations
        ("RecoverFromAidrop", Self::RecoverFromAidrop),
        ("RecoverFromInsurance", Self::RecoverFromInsurance),
        ("RecoverFromStaking", Self::RecoverFromStaking),
        ("RecoverFromVesting", Self::RecoverFromVesting),

        ("StakingRecover", Self::StakingRecover),
        ("VestingRecover", Self::VestingRecover),

        // Insurance & Protection
        ("AllocateInsurance", Self::AllocateInsurance),

        // Misc
        ("Custom", Self::Custom),
    ];

    pub(crate) fn from_str(name: &str) -> Option<Self> {
        Self::ALL.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
    }

    pub(crate) fn as_bit(self) -> u32 {
        1u32 << (self as u8)
    }
}