use anchor_lang::prelude::*;

// ======================================================================
// Soccial Token – Initialization Error Definitions
//
// This module defines errors related to the initialization of core
// contract components, such as SPL Token setup, economy state, and 
// team vesting schedules.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

#[error_code]
pub enum InitializeErrorCode {

    #[msg("Contract already initialized.")]
    ContractAlreadyInitialized,

    #[msg("Economy components already initialized.")]
    EconomyAlreadyExists,
    
    #[msg("Spl Token components already initialized.")]
    SplTokenAlreadyExists,

    #[msg("SPL Token already initialized.")]
    SplTokenAlreadyInitialized,

    #[msg("Contract economy already initialized.")]
    ContractEconomyAlreadyInitialized,

    #[msg("Proposal not found.")]
    ProposalNotFound,

    #[msg("Team vesting already created.")]
    TeamVestinglreadyCreate,
 
}