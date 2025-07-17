// ======================================================================
// Soccial Token – Staking Contexts
//
// Defines account contexts for staking-related operations including
// buying and staking tokens, staking tokens directly, releasing staked tokens,
// and managing global staking configurations.
//
// Each context ensures correct PDA derivations, access control, and token flows.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::auth::user::UserAccessAccount;
use crate::staking::{StakingState, state::StakingAccount};
use crate::token::TokenState;

#[derive(Accounts)]
pub struct BuyAndStakeTokens<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The signer initiating the stake (usually the backend authority).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional access control for the caller (can be used to validate admin).
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Participant Info
    // =========================================================================

    /// CHECK: Used for address only; does not require validation. Does not need to sign.
    #[account(mut)]
    pub participant: AccountInfo<'info>,

    // =========================================================================
    // Token Mint & Configuration
    // =========================================================================

    /// The token mint of the Soccial Token (SCTK).
    pub token_mint: Account<'info, Mint>,

    /// Global token configuration and fee parameters.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Liquidity Vault (Source of tokens)
    // =========================================================================

    /// CHECK: PDA authority, validated by seeds. No data is read or written.
    #[account(
        seeds = [b"liquidity_vault"],
        bump,
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account owned by the liquidity vault PDA.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = liquidity_vault,
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Staking Vault (Destination of stake and rewards)
    // =========================================================================

    /// CHECK: PDA authority, validated by seeds. No data is read or written.
    #[account(
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// Token account owned by the staking vault PDA (receives both stake and rewards).
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = staking_vault,
    )]
    pub staking_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Staking State & Account
    // =========================================================================

    /// Global staking configuration account.
    #[account(mut)]
    pub staking_state: Account<'info, StakingState>,

    /// The account that will store the staking metadata.
    #[account(
        init_if_needed,
        seeds = [
            b"staking_account",
            participant.key().as_ref(),
            staking_state.last_id.to_le_bytes().as_ref()
        ],
        bump,
        payer = caller,
        space = StakingAccount::LEN,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    // =========================================================================
    // Programs
    // =========================================================================

    /// Required SPL token program.
    pub token_program: Program<'info, Token>,

    /// System program required for account creations.
    pub system_program: Program<'info, System>,

    /// Associated Token Program (used to validate token account derivations).
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[derive(Accounts)]
pub struct StakeTokens<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller initiating the staking (can be the user or an admin).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional access control for caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Participant Info
    // =========================================================================

    /// CHECK: The wallet or PDA of the participant staking tokens.
    #[account(mut, signer)]
    pub participant: AccountInfo<'info>,

    /// Participant's token account (SCTK) – source of staked tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = participant,
    )]
    pub participant_token_account: Account<'info, TokenAccount>,

    /// Destination token account (might be used for future mints or transfers).
    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = token_mint,
        associated_token::authority = participant,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Staking State & Account
    // =========================================================================

    /// The global staking state used to track all staking metadata.
    #[account(
        mut,
        seeds = [b"staking_state"],
        bump,
    )]
    pub staking_state: Account<'info, StakingState>,

    /// The account to store staking data for this specific stake.
    #[account(
        init_if_needed,
        seeds = [
            b"staking_account", 
            participant.key().as_ref(), 
            staking_state.last_id.to_le_bytes().as_ref()
        ],
        bump,
        payer = caller,
        space = StakingAccount::LEN,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    // =========================================================================
    // Vaults (Staking & Liquidity)
    // =========================================================================

    /// CHECK: The staking_vault PDA that owns the staking ATA.
    #[account(
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// Token account owned by the staking vault PDA – receives the staked tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = staking_vault,
    )]
    pub staking_vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for liquidity operations. No data is read or written.
    #[account(
        seeds = [b"liquidity_vault"],
        bump,
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account owned by the liquidity vault PDA.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = liquidity_vault,
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Token Mint & Authority
    // =========================================================================

    /// The token mint of the Soccial Token (SCTK).
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: PDA that acts as mint authority. Only validated via seeds.
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Global token configuration.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[derive(Accounts)]
pub struct ReinforceStake<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller initiating the staking (can be the user or an admin).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional access control for caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Participant Info
    // =========================================================================

    /// CHECK: The wallet or PDA of the participant staking tokens.
    #[account(mut, signer)]
    pub participant: AccountInfo<'info>,

    /// Participant's token account (SCTK) – source of staked tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = participant,
    )]
    pub participant_token_account: Account<'info, TokenAccount>,

    /// Destination token account (might be used for future mints or transfers).
    #[account(
        init_if_needed,
        payer = caller,
        associated_token::mint = token_mint,
        associated_token::authority = participant,
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Staking State & Account
    // =========================================================================

    /// The global staking state used to track all staking metadata.
    #[account(
        mut,
        seeds = [b"staking_state"],
        bump,
    )]
    pub staking_state: Account<'info, StakingState>,

    /// PDA storing stake metadata.
    #[account(mut)]
    pub staking_account: Account<'info, StakingAccount>,

    // =========================================================================
    // Vaults (Staking & Liquidity)
    // =========================================================================

    /// CHECK: The staking_vault PDA that owns the staking ATA.
    #[account(
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// Token account owned by the staking vault PDA – receives the staked tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = staking_vault,
    )]
    pub staking_vault_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for liquidity operations. No data is read or written.
    #[account(
        seeds = [b"liquidity_vault"],
        bump,
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account owned by the liquidity vault PDA.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = liquidity_vault,
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Token Mint & Authority
    // =========================================================================

    /// The token mint of the Soccial Token (SCTK).
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    /// CHECK: PDA that acts as mint authority. Only validated via seeds.
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Global token configuration.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Programs
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
}

#[derive(Accounts)]
pub struct ReleaseStaked<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller releasing tokens (can be the user or admin).
    pub caller: Signer<'info>,

    /// Access control data for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Staking Account & Metadata
    // =========================================================================

    /// CHECK: The staking account for the participant and specific stake ID.
    #[account(
        mut,
        seeds = [
            b"staking_account",
            staking_account.participant.as_ref(),
            staking_account.stake_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub staking_account: Account<'info, StakingAccount>,

    // =========================================================================
    // Token Mint & Authority
    // =========================================================================

    /// The token mint of SCTK.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority that owns the vaults.
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Token configuration and core settings.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Vault & Token Accounts
    // =========================================================================

    /// CHECK: PDA representing the staking vault authority.
    #[account(
        mut,
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// The ATA of the staking vault holding staked tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = staking_vault,
    )]
    pub staking_vault_token_account: Account<'info, TokenAccount>,

    /// The destination ATA for the user receiving released tokens.
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Programs & Sysvars
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Clock sysvar (used for time validation on staking releases).
    pub clock: Sysvar<'info, Clock>,
}

/// Context for releasing previously staked tokens.
#[derive(Accounts)]
pub struct WithdrawStaked<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The caller releasing tokens (can be the user or admin).
    pub caller: Signer<'info>,

    /// Access control data for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Staking Metadata & Closure
    // =========================================================================

    /// CHECK: The staking account for the participant and specific stake ID.
    #[account(
        mut,
        seeds = [
            b"staking_account",
            staking_account.participant.as_ref(),
            staking_account.stake_id.to_le_bytes().as_ref()
        ],
        bump,
        close = mint_authority
    )]
    pub staking_account: Account<'info, StakingAccount>,

    /// The wallet that will receive remaining lamports when `staking_account` is closed.
    #[account(mut, address = token_state.core.owner)]
    pub recipient_of_lamports: SystemAccount<'info>,

    // =========================================================================
    // Token Mint & Authority
    // =========================================================================

    /// The token mint of SCTK.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// CHECK: PDA authority that owns the vaults.
    #[account(
        mut,
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Global token configuration and settings.
    pub token_state: Account<'info, TokenState>,

    // =========================================================================
    // Vaults & Token Accounts
    // =========================================================================

    /// CHECK: PDA representing the staking vault authority.
    #[account(
        mut,
        seeds = [b"staking_vault"],
        bump,
    )]
    pub staking_vault: AccountInfo<'info>,

    /// The ATA of the staking vault holding staked tokens.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = staking_vault,
    )]
    pub staking_vault_token_account: Account<'info, TokenAccount>,

    /// The destination ATA for the user receiving released tokens.
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    // =========================================================================
    // Programs & Sysvars
    // =========================================================================

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar
    pub clock: Sysvar<'info, Clock>,
}

/// Context for managing global staking configuration.
#[derive(Accounts)]
pub struct ManageStaking<'info> {
    // =========================================================================
    // Caller & Access Control
    // =========================================================================

    /// The admin or user managing the staking state.
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Access control for permission validation.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump,
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // =========================================================================
    // Staking & Token State
    // =========================================================================

    /// The global staking state (config + counter).
    #[account(mut)]
    pub staking_state: Account<'info, StakingState>,

    /// Global token configuration.
    pub token_state: Account<'info, TokenState>,
}