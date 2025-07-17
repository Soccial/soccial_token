// ======================================================================
// Soccial Token – Initialization Contexts
//
// Defines core account contexts for initializing the Soccial Token contract,
// including the main token setup, economy components, SPL token mint,
// and founders’ vesting schedules.
//
// Each context ensures proper PDA derivation, ownership validation,
// and the setup of associated token accounts as needed.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::{
    token::TokenState, utils::error::ErrorCode, vesting::VestingState
};
// =============================
/// Initializes the core configuration for the Soccial Token, including the contract settings,
/// system control, admin config, permissions, and the user query limiter for the caller.
// ==============================
#[derive(Accounts)]
pub struct InitializeToken<'info> {
    // =========================================================================
    // Signer
    // =========================================================================

    /// The caller that pays and signs for initialization.
    #[account(mut)]
    pub caller: Signer<'info>,

    // =========================================================================
    // Core PDAs to be manually initialized via invoke_signed
    // =========================================================================

    /// CHECK: Token State PDA to be created via invoke_signed.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump,
    )]
    pub token_state: AccountInfo<'info>,

    /// CHECK: Mint Authority PDA for minting control.
    #[account(
        mut,
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    /// CHECK: Token Mint PDA (SCTK Mint).
    #[account(
        mut,
        seeds = [b"token_mint"],
        bump,
    )]
    pub mint: AccountInfo<'info>,

    /// CHECK: Global Governance configuration PDA.
    #[account(
        mut,
        seeds = [b"governance_state"],
        bump,
    )]
    pub governance_state: AccountInfo<'info>,

    /// CHECK: Vesting state PDA.
    #[account(
        mut,
        seeds = [b"vesting_state"],
        bump,
    )]
    pub vesting_state: AccountInfo<'info>,

    /// CHECK: Staking state PDA.
    #[account(
        mut,
        seeds = [b"staking_state"],
        bump,
    )]
    pub staking_state: AccountInfo<'info>,

    // =========================================================================
    // Initial Token Allocation
    // =========================================================================

    /// CHECK: Associated Token Account of the mint authority.
    /// Must be owned by `mint_authority`. Validated manually.
    #[account(mut)]
    pub authority_token_account: AccountInfo<'info>,

    // =========================================================================
    // Programs & Sysvars
    // =========================================================================

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


// ==============================
// Token Economy Context
// ==============================

/// Initializes the core economic components (vaults + token state)
#[derive(Accounts)]
pub struct InitializeEconomy<'info> {
    // =========================================================================
    // Signer
    // =========================================================================

    /// The account initializing the economy setup.
    #[account(mut)]
    pub caller: Signer<'info>,

    // =========================================================================
    // Core PDA Accounts (manually created with invoke_signed)
    // =========================================================================

    /// CHECK: Staking configuration and counters.
    #[account(
        mut,
        seeds = [b"staking_state"],
        bump,
    )]
    pub staking_state: AccountInfo<'info>,

    /// CHECK: Vault for general liquidity (main reserve).
    #[account(
        mut,
        seeds = [b"liquidity_vault"],
        bump,
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// CHECK: Vault holding tokens staked by users.
    #[account(
        mut,
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// CHECK: Vault holding tokens reserved for scheduled release.
    #[account(
        mut,
        seeds = [b"reserved_supply_vault"],
        bump,
    )]
    pub reserved_supply_vault: AccountInfo<'info>,

    /// CHECK: Vault for airdrop distribution pool.
    #[account(
        mut,
        seeds = [b"airdrop_vault"],
        bump,
    )]
    pub airdrop_vault: AccountInfo<'info>,

    /// CHECK: Vault holding tokens for vesting purposes.
    #[account(
        mut,
        seeds = [b"vesting_vault"],
        bump,
    )]
    pub vesting_vault: AccountInfo<'info>,

    /// CHECK: Vault reserved for off-chain conversions or fiat liquidity.
    #[account(
        mut,
        seeds = [b"offchain_reserve_vault"],
        bump,
    )]
    pub offchain_reserve_vault: AccountInfo<'info>,

    /// CHECK: Vault that receives platform revenue (e.g. fees).
    #[account(
        mut,
        seeds = [b"revenue_vault"],
        bump,
    )]
    pub revenue_vault: AccountInfo<'info>,

    /// CHECK: Vault that holds tokens to be distributed as rewards.
    #[account(
        mut,
        seeds = [b"rewards_vault"],
        bump,
    )]
    pub rewards_vault: AccountInfo<'info>,

    /// CHECK: Insurance vault for emergencies and unexpected risks.
    #[account(
        mut,
        seeds = [b"insurance_vault"],
        bump,
    )]
    pub insurance_vault: AccountInfo<'info>,

    /// CHECK: Treasury vault for operational and financial management.
    #[account(
        mut,
        seeds = [b"treasury_vault"],
        bump,
    )]
    pub treasury_vault: AccountInfo<'info>,

    /// CHECK: Mint for the main token (SCTK). Created separately.
    #[account(
        mut,
        seeds = [b"token_mint"],
        bump,
    )]
    pub token_mint: AccountInfo<'info>,

    /// Global token state tracking supply and ownership.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump,
    )]
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitializeEconomy<'info> {
    pub fn validate(&self) -> Result<()> {
        if !self.token_state.core.initialized {
            return Err(ErrorCode::TokenNotInitialized.into());
        }

        Ok(())
    }
}

// ==============================
// SPL Contexts
// ==============================
/// Accounts for initializing the SPL Token.
/// 
/// The Mint account must be uninitialized.
/// The Mint is created and initialized manually in the handler function.
/// 
/// We do not use `#[account(init, mint::decimals, mint::authority)]` here.
/// Instead, we manually create and initialize the Mint via system instruction + invoke_signed.

use anchor_spl::{associated_token::AssociatedToken, token::Token};

/// Accounts required for SPL token initialization and liquidity linkage.
/// Initializes all SPL Token vaults and accounts for the Soccial Token.
/// This includes creating ATAs for each vault authority and ensuring the mint authority holds its ATA.
#[derive(Accounts)]
pub struct InitializeSplToken<'info> {
    // =========================================================================
    // Signer
    // =========================================================================

    /// The initializer of the SPL Token setup.
    #[account(mut)]
    pub caller: Signer<'info>,

    // =========================================================================
    // Core Accounts
    // =========================================================================

    /// Token mint for the Soccial Token.
    #[account(
        mut,
        seeds = [b"token_mint"],
        bump
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Mint Authority PDA for minting control.
    #[account(
        seeds = [b"mint_authority"],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Global token state tracking economics and status flags.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    /// CHECK: Mint authority's ATA (used for minting tokens).
    #[account(
        mut,
        address = spl_associated_token_account::get_associated_token_address(
            &mint_authority.key(),
            &mint.key(),
        )
    )]
    pub authority_token_account: AccountInfo<'info>,

    /// CHECK: Token account owned by a PDA to collect tokens mistakenly sent to the contract.
    #[account(mut)]
    pub contract_token_account: AccountInfo<'info>,

    /// CHECK: PDA that owns the fallback contract token account.
    #[account(
        seeds = [b"contract_token_owner"],
        bump
    )]
    pub contract_token_owner: AccountInfo<'info>,

    // =========================================================================
    // Vault Authorities (PDAs only used for ATA creation)
    // =========================================================================

    /// CHECK: Vault
    #[account(seeds = [b"offchain_reserve_vault"], bump)]
    pub offchain_reserve_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"liquidity_vault"], bump)]
    pub liquidity_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"staking_vault"], bump)]
    pub staking_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"revenue_vault"], bump)]
    pub revenue_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"rewards_vault"], bump)]
    pub rewards_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"airdrop_vault"], bump)]
    pub airdrop_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"reserved_supply_vault"], bump)]
    pub reserved_supply_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"vesting_vault"], bump)]
    pub vesting_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"insurance_vault"], bump)]
    pub insurance_vault: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(seeds = [b"treasury_vault"], bump)]
    pub treasury_vault: AccountInfo<'info>,

    // =========================================================================
    // Vault Token Accounts (ATAs for each PDA authority)
    // =========================================================================
    /// CHECK: Vault
    #[account(mut)]
    pub offchain_reserve_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub liquidity_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub staking_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub revenue_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub rewards_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub airdrop_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub reserved_supply_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub vesting_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub insurance_vault_token_account: AccountInfo<'info>,

    /// CHECK: Vault
    #[account(mut)]
    pub treasury_vault_token_account: AccountInfo<'info>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitializeSplToken<'info> {
    pub fn validate(&self) -> Result<()> {
        if !self.token_state.core.initialized {
            return Err(ErrorCode::TokenNotInitialized.into());
        }

        if !self.token_state.core.economy_initialized {
            return Err(ErrorCode::TokenEconomyNotInitialized.into());
        }

        require_keys_eq!(
            self.caller.key(),
            self.token_state.core.owner,
            ErrorCode::Unauthorized
        );

        Ok(())
    }
}

// =========================================================================
/// Initializes the vesting schedules for the founding team members.
/// Creates two predefined vesting accounts manually via `invoke_signed`.
// =========================================================================
#[derive(Accounts)]
pub struct InitializeFoundersVesting<'info> {
    // =========================================================================
    // Signer
    // =========================================================================

    /// The initializer of the founders vesting schedules.
    #[account(mut)]
    pub caller: Signer<'info>,

    // =========================================================================
    // Core State
    // =========================================================================

    /// Vesting state tracking the global vesting IDs and configuration.
    #[account(
        mut,
        seeds = [b"vesting_state"],
        bump
    )]
    pub vesting_state: Account<'info, VestingState>,

    /// Token state tracking supply and configuration.
    #[account(
        mut,
        seeds = [b"token_state"],
        bump
    )]
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Vesting Schedule Accounts (Manually initialized PDAs)
    // =========================================================================

    /// PDA for the first founding team member’s vesting schedule.
    /// CHECK: Created and validated manually via `invoke_signed`.
    #[account(mut)]
    pub team1_vesting_schedule: AccountInfo<'info>,

    /// PDA for the second founding team member’s vesting schedule.
    /// CHECK: Created and validated manually via `invoke_signed`.
    #[account(mut)]
    pub team2_vesting_schedule: AccountInfo<'info>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}


impl<'info> InitializeFoundersVesting<'info> {
    pub fn validate(&self) -> Result<()> {
        if !self.token_state.core.initialized {
            return Err(ErrorCode::TokenNotInitialized.into());
        }

        if !self.token_state.core.economy_initialized {
            return Err(ErrorCode::TokenEconomyNotInitialized.into());
        }
       
        require_keys_eq!(
            self.caller.key(),
            self.token_state.core.owner,
            ErrorCode::Unauthorized
        );
      
        Ok(())
    }
}