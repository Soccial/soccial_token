// ===========================================================================
// Vesting State Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module defines the account structures used to manage and track vesting
// operations within the Soccial Token protocol. It enables time-based token
// distribution through programmable release schedules.
//
// ---------------------------------------------------------------------------
// Components:
// - `VestingState`: Tracks total number of vesting schedules and assigns unique IDs
// - `VestingSchedule`: Stores metadata and configuration for a participant’s vesting
//
// ---------------------------------------------------------------------------
// Key Features:
// - Linear or cyclical vesting (e.g. monthly releases or continuous)
// - Immutable flag to lock schedule post-creation
// - Cancelable schedules (unless immutable)
// - Built-in calculation logic to determine vested tokens at any time
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;

/// Stores global state for vesting schedules.
///
/// - `total_schedules`: Total number of schedules ever created
/// - `last_id`: The next vesting ID to assign (incremented sequentially)
#[account]
pub struct VestingState {
    pub total_schedules: u64,     // 8 bytes
    pub last_id: u64,     // 8 bytes
}

impl VestingState {
    pub const LEN: usize = 8 + 8 + 8; // 8 discriminator + 2x u64
}


/// Represents a single vesting schedule for a participant.
///
/// Each participant can have multiple vesting schedules identified by different `vesting_id`s.
/// This structure tracks the allocation of tokens that are released over time.
#[account]
pub struct VestingSchedule {
    /// The participant (user) who will receive the vested tokens.
    pub participant: Pubkey,

    /// The timestamp at which the vesting schedule starts.
    pub start_time: i64,

    /// The duration (in seconds) of the cliff period before any tokens can be claimed.
    pub cliff_duration: i64,

    /// The total duration (in seconds) over which all tokens will vest.
    pub vesting_duration: i64,

    /// Number of vesting cycles. If set to 0, uses linear distribution.
    /// Otherwise, tokens are released in equal chunks across the cycles.
    pub cycles: i64,
    
    /// The number of tokens to release immediately at start (TGE-like behavior).
    pub initial_tokens: u64,

    /// The total number of tokens allocated for this vesting schedule.
    pub total_tokens: u64,

    /// The number of tokens that have already been released to the participant.
    pub released_tokens: u64,

    /// Whether this vesting schedule can be modified after creation.
    pub immutable: bool,

    /// The last time (timestamp) when the participant claimed vested tokens.
    pub last_claim_time: i64,

    /// A unique identifier for the vesting schedule (used for PDA derivation).
    pub vesting_id: u64,

    /// Current status of the vesting schedule:
    /// 0 = uninitialized, 1 = active, 3 = cancelled.
    pub status: u8,
}

impl VestingSchedule {
    /// Computes the byte size required to allocate the VestingSchedule account.
    ///
    /// Layout breakdown:
    pub const LEN: usize =
        8   // Anchor discriminator
        + 32  // participant: Pubkey
        + 8   // start_time: i64
        + 8   // cliff_duration: i64
        + 8   // vesting_duration: i64
        + 8   // cycles: i64
        + 8   // initial_tokens: u64
        + 8   // total_tokens: u64
        + 8   // released_tokens: u64
        + 1   // immutable: bool
        + 8   // last_claim_time: i64
        + 8   // vesting_id: u64
        + 1   // status: u8
        + 7;  // padding for alignment (next multiple of 8)

    /// Calculates the total number of tokens vested (but not necessarily claimed) so far.
    ///
    /// ## Logic:
    /// - If `cycles == 0`: linear release between cliff and end
    /// - If `cycles > 0`: tokens released in equal chunks per cycle
    /// - Initial tokens are always available at `start_time`
    ///
    /// ## Parameters:
    /// - `current_time`: The current UNIX timestamp
    ///
    /// ## Returns:
    /// - `u64`: Total tokens vested as of `current_time`
    ///
    /// ## Safety:
    /// - Returns early if still in cliff period
    /// - Caps ratio/cycles within logical bounds to avoid overflows
    pub(crate) fn calculate_vested_amount(&self, current_time: i64) -> u64 {
        // Always available: initial_tokens are released at the start_time
        if current_time < self.start_time {
            return 0;
        }

        let total_vested = self.initial_tokens;

        // Before cliff ends: only initial_tokens are available
        let cliff_end = self.start_time + self.cliff_duration;
        if current_time < cliff_end {
            return total_vested;
        }

        let elapsed = current_time - cliff_end;

        // No vesting logic applied if config is invalid
        if self.vesting_duration <= 0 || self.total_tokens == 0 {
            return total_vested;
        }

        // LINEAR vesting
        if self.cycles <= 0 {
            let ratio = (elapsed as f64 / self.vesting_duration as f64).clamp(0.0, 1.0);
            let linear_vested = (self.total_tokens as f64 * ratio).floor() as u64;
            return total_vested + linear_vested;
        }

        // CYCLICAL vesting
        let cycle_duration = self.vesting_duration / self.cycles;
        if cycle_duration <= 0 {
            return total_vested + self.total_tokens;
        }

        let passed_cycles = (elapsed / cycle_duration).clamp(0, self.cycles);
        let tokens_per_cycle = self.total_tokens / self.cycles as u64;

        total_vested + (passed_cycles as u64 * tokens_per_cycle)
    }


}
