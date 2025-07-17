// ===========================================================================
// Governance State Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module defines the persistent governance accounts used for proposal
// management and voting configuration within the Soccial Token ecosystem.
//
// ---------------------------------------------------------------------------
// ## Components:
// - `GovernanceState`: Stores global configuration (quorum %, durations, min vote power)
// - `ProposalAccount`: Represents an individual proposal and its voting data
//
// ---------------------------------------------------------------------------
// ## Features:
// - Dynamic governance config via key=value arguments
// - Type-safe space calculations (`LEN`)
// - Bitflag-based proposal typing
//
// ---------------------------------------------------------------------------
// ## Storage Constraints:
// - `GovernanceState`: 48 bytes
// - `ProposalAccount`: ~344 bytes (with capped description length)
//
// ---------------------------------------------------------------------------
// ## Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;

/// Global governance state account.
/// Stores settings related to proposals and voting.
#[account]
pub struct GovernanceState {
    pub last_id: u64,            // Number of proposals created so far.
    pub min_vote_tokens: u64,           // Minimum number of tokens required to vote
    pub quorum_percent: u64,            // Minimum percentage of votes required for quorum
    pub voting_duration: i64,           // Duration of the voting period in seconds
    pub validity_period: i64,           // Duration for which an approved proposal remains valid
    
}

impl GovernanceState {
    /// Space calculation for GovernanceState account:
    pub const LEN: usize = 
    8 +     // Anchor discriminator
    8 +     // last_id (u64)
    8 +     // min_vote_tokens (u64)
    8 +     // quorum_percent (u64)
    8 +     // voting_duration (i64)
    8;      // validity_period (i64)
    // Total: 48 bytes

    /// Dynamically updates governance parameters via string-based key=value pairs.
    ///
    /// ## Parameters:
    /// - `args`: vector of update strings (e.g., `"quorum_percent=6"`)
    ///
    /// ## Supported Keys:
    /// - `"min_tokens"` → `min_vote_tokens`
    /// - `"quorum_percent"` → `quorum_percent`
    /// - `"voting_duration"` → `voting_duration`
    /// - `"validity_period"` → `validity_period`
    ///
    /// ## Errors:
    /// - `NotEnoughArguments` if input is empty
    /// - `InvalidArgument` if any key is unknown or parsing fails
    ///
    /// ## Logs:
    /// Emits a log summary of all applied changes.
   pub(crate) fn apply_updates(&mut self, args: Vec<String>) -> Result<()> {
        require!(!args.is_empty(), crate::ErrorCode::NotEnoughArguments);

        let mut log: Vec<String> = vec![];

        for arg in args {
            let parts: Vec<&str> = arg.split('=').collect();
            require!(parts.len() == 2, crate::ErrorCode::InvalidArgument);
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "min_tokens" => {
                    let val = value.parse::<u64>().map_err(|_| crate::ErrorCode::InvalidArgument)?;
                    self.min_vote_tokens = val;
                    log.push(format!("min_tokens: {}", val));
                }
                "quorum_percent" => {
                    let val = value.parse::<u64>().map_err(|_| crate::ErrorCode::InvalidArgument)?;
                    self.quorum_percent = val;
                    log.push(format!("quorum_percent: {}%", val));
                }
                "voting_duration" => {
                    let val = value.parse::<i64>().map_err(|_| crate::ErrorCode::InvalidArgument)?;
                    self.voting_duration = val;
                    log.push(format!("voting_duration: {}s", val));
                }
                "validity_period" => {
                    let val = value.parse::<i64>().map_err(|_| crate::ErrorCode::InvalidArgument)?;
                    self.validity_period = val;
                    log.push(format!("validity_period: {}s", val));
                }
                _ => return Err(crate::ErrorCode::InvalidArgument.into()),
            }
        }

        msg!("⚙️ Updated governance settings → {}", log.join(" | "));
        Ok(())
    }
}


/// Individual governance proposal account.
/// Stores information about a specific proposal.
#[account]
pub struct ProposalAccount {
    pub id: u64,                       // Unique identifier for the proposal
    pub description: String,           // Description of the proposal (max 256 characters)
    pub proposal_bitflag: u32,         // Bitflags to define the type or category of the proposal
    pub votes_for: u64,                // Number of votes in favor
    pub votes_against: u64,            // Number of votes against
    pub start_time: i64,               // Unix timestamp when voting starts
    pub end_time: i64,                 // Unix timestamp when voting ends
    pub creator: Pubkey,               // Public key of the user who created the proposal
    pub is_used: bool,                 // Marks whether the proposal was ever voted on
    pub is_finalized: bool,            // Marks whether the proposal has been finalized
}

impl ProposalAccount {
    pub const MAX_DESCRIPTION_LEN: usize = 256; // Maximum description size (characters)

    /// Space calculation for ProposalAccount account:
    pub const LEN: usize = 
        8 +                                 // Anchor discriminator
        8 +                                 // id (u64)
        4 +                                 // String length prefix for description (u32)
        Self::MAX_DESCRIPTION_LEN +         // description content
        4 +                                 // proposal_bitflag (u32)
        8 +                                 // votes_for (u64)
        8 +                                 // votes_against (u64)
        8 +                                 // start_time (i64)
        8 +                                 // end_time (i64)
        32 +                                // creator (Pubkey)
        1 +                                 // is_used (bool)
        1;                                  // is_finalized (bool)
        // Total: 8 + 8 + 4 + 256 + 4 + 8 + 8 + 8 + 8 + 32 + 1 + 1 = 344 bytes
}

#[account]
pub struct VoteReceipt {
    pub proposal: Pubkey,
    pub voter: Pubkey,
    pub support: bool,
    pub vote_power: u64,
    pub timestamp: i64,
}