// ===========================================================================
// FeeDistribution – SCTK Economy Fee Management Module
// ---------------------------------------------------------------------------
//
// This module handles **fee logic** for the Soccial Token (SCTK), including:
// - Validating fee configuration (in basis points)
// - Dynamically updating fee values (rewards / airdrop)
// - Splitting total fee amounts into vault portions
// - Calculating the residual "revenue" fee
//
// ---------------------------------------------------------------------------
// Fee Structure:
// - All fees are represented in **BPS** (basis points): 1% = 100 BPS
// - `rewards_fee_bps`: % of fees allocated to the Rewards Vault
// - `airdrop_fee_bps`: % of fees allocated to the Airdrop Vault
// - Remainder goes to the Revenue Vault (`revenue_fee_bps`)
//
// ---------------------------------------------------------------------------
// Key Features:
// - Safety-first: includes bounds checking and overflow protection
// - Extensible: can support dynamic fee models and reconfiguration
// - Efficient: all math is u64-based and computed inline
//
// ---------------------------------------------------------------------------
// Storage:
// - Struct size: `LEN = 4 bytes`
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use crate::{
    economics::error::EconomicsErrorCode,
    economy::fee::{FEE_BPS_BASE, MAX_AIRDROP_FEE_BPS, MAX_REWARDS_FEE_BPS, MIN_FEE_BPS},
};

#[event]
pub struct RewardsFeeUpdated {
    pub new_bps: u16,
    pub percent: f64,
}

#[event]
pub struct AirdropFeeUpdated {
    pub new_bps: u16,
    pub percent: f64,
}


/// 📊 Handles the economy operations of the Soccial Token (SCTK).
///
/// Includes:
/// - Fee validation
/// - Fee distribution to vaults
///
/// All percentages are handled in **basis points (BPS)**: 1% = 100 BPS.
#[derive(Clone, Copy, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct FeeDistribution {
    pub rewards_fee_bps: u16,
    pub airdrop_fee_bps: u16,
}


impl FeeDistribution {
   

    /// Splits a total fee into its reward, airdrop, and revenue components.
    ///
    /// ## Parameters:
    /// - `total_fee`: The full fee amount (in base units, e.g., lamports or SCTK units)
    ///
    /// ## Returns:
    /// - `(rewards, airdrop, revenue)` – individual vault allocations
    ///
    /// ## Safety:
    /// - Calls `finalize()` internally to validate fee bounds
    /// - Checks for overflow when subtracting rewards + airdrop from total
    ///
    /// ## Errors:
    /// - `EconomicsErrorCode::Overflow` if math fails
    /// - `EconomicsErrorCode::InvalidFeeValue` if any configured BPS is invalid
    pub(crate) fn split_fee(&self, total_fee: u64) -> Result<(u64, u64, u64)> {
        self.finalize()?; // Validate that both reward and airdrop BPS are within bounds

        let rewards = total_fee * self.rewards_fee_bps as u64 / FEE_BPS_BASE as u64;
        let airdrop = total_fee * self.airdrop_fee_bps as u64 / FEE_BPS_BASE as u64;
        let revenue = total_fee.checked_sub(rewards + airdrop).ok_or(EconomicsErrorCode::Overflow)?;
        Ok((rewards, airdrop, revenue)) 
    }
    
    /// Updates the rewards fee (in BPS) after validating it falls within bounds.
    ///
    /// ## Parameters:
    /// - `new_bps`: The new fee percentage in basis points (1% = 100 BPS)
    ///
    /// ## Errors:
    /// - `InvalidFeeValue` if the fee is outside allowed range
    ///
    /// ## Dev Output:
    /// Logs confirmation message with updated BPS and %.
    pub(crate) fn update_rewards_fee(&mut self, new_bps: u16) -> Result<()> {
        require!(
            (MIN_FEE_BPS..=MAX_REWARDS_FEE_BPS).contains(&new_bps),
            EconomicsErrorCode::InvalidFeeValue
        );
        self.rewards_fee_bps = new_bps;
        msg!("✅ Rewards fee updated to {} BPS ({}%)", new_bps, new_bps as f64 / 100.0);

        emit!(RewardsFeeUpdated {
            new_bps,
            percent: new_bps as f64 / 100.0,
        });

        Ok(())
    }

    /// Updates the airdrop fee (in BPS) after validating it falls within bounds.
    ///
    /// ## Parameters:
    /// - `new_bps`: The new fee percentage in basis points (1% = 100 BPS)
    ///
    /// ## Errors:
    /// - `InvalidFeeValue` if the fee is outside allowed range
    ///
    /// ## Dev Output:
    /// Logs confirmation message with updated BPS and %.
    pub(crate) fn update_airdrop_fee(&mut self, new_bps: u16) -> Result<()> {
        require!(
            (MIN_FEE_BPS..=MAX_AIRDROP_FEE_BPS).contains(&new_bps),
            EconomicsErrorCode::InvalidFeeValue
        );
        self.airdrop_fee_bps = new_bps;
        msg!("✅ Airdrop fee updated to {} BPS ({}%)", new_bps, new_bps as f64 / 100.0);

        emit!(AirdropFeeUpdated {
            new_bps,
            percent: new_bps as f64 / 100.0,
        });

        Ok(())
    }


    /// Validates that both `rewards_fee_bps` and `airdrop_fee_bps`
    /// are within allowed bounds.
    ///
    /// ## Returns:
    /// - `Ok(self)` if both are valid
    /// - `Err(InvalidFeeValue)` if either is invalid
    fn finalize(self) -> Result<Self> {
        require!(Self::is_valid(self.rewards_fee_bps), EconomicsErrorCode::InvalidFeeValue);
        require!(Self::is_valid(self.airdrop_fee_bps), EconomicsErrorCode::InvalidFeeValue);
        Ok(self)
    }

     /// Validates if a single fee (in BPS) is within the allowed global range.
    ///
    /// ## Internal Use:
    /// Called during `finalize()` to enforce BPS limits.
    ///
    /// ## Logic:
    /// - `MIN_FEE_BPS <= fee <= FEE_BPS_BASE`
    fn is_valid(fee_bps: u16) -> bool {
        (MIN_FEE_BPS..=FEE_BPS_BASE).contains(&fee_bps)
    }

    /// Constant space used by this struct in bytes.
    pub const LEN: usize = 2 + 2; // u16 + u16
}
