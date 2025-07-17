// ======================================================================
// Soccial Token – Market Contexts
//
// Defines account contexts for token market operations including buying tokens,
// depositing tokens, transferring tokens, and fee distribution.
//
// Contexts include PDA validations, token account checks, optional user access,
// and global token state references.
//
// License: MIT License
// Author: Paulo Rodrigues
// Project: Soccial Token
// ======================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{auth::user::UserAccessAccount, token::state::TokenState};

#[derive(Accounts)]
pub struct BuyTokensContext<'info> {
    // ─────────────────────────────────────────────────────────────
    // Signer
    // ─────────────────────────────────────────────────────────────

    /// The buyer executing the token purchase.
    pub caller: Signer<'info>,

    // ─────────────────────────────────────────────────────────────
    // Buyer Token Account
    // ─────────────────────────────────────────────────────────────
    /// Token account owned by the buyer, receives purchased tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = buyer_token_account.owner
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

   // ------------------------------------------------------------------------
    // Liquidity Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for liquidity vault operations. Verified in handler.
    #[account(
        seeds = [b"liquidity_vault"],
        bump
    )]
    pub liquidity_vault: AccountInfo<'info>,

    /// Token account holding tokens available for purchase.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = liquidity_vault
    )]
    pub liquidity_vault_token_account: Account<'info, TokenAccount>,
    
    // ------------------------------------------------------------------------
    // Rewards Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for rewards distribution logic. Verified in handler.
    #[account(
        seeds = [b"rewards_vault"],
        bump
    )]
    pub rewards_vault: AccountInfo<'info>,

    /// Token account holding tokens used for rewards distribution.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = rewards_vault
    )]
    pub rewards_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Revenue Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for revenue collection logic. Verified in handler.
    #[account(
        seeds = [b"revenue_vault"],
        bump
    )]
    pub revenue_vault: AccountInfo<'info>,

    /// Token account accumulating revenue from token sales or fees.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = revenue_vault
    )]
    pub revenue_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Airdrop Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for airdrop logic. Verified in handler.
    #[account(
        seeds = [b"airdrop_vault"],
        bump
    )]
    pub airdrop_vault: AccountInfo<'info>,

    /// Token account holding tokens reserved for airdrops.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = airdrop_vault
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,


    // ─────────────────────────────────────────────────────────────
    // Mint
    // ─────────────────────────────────────────────────────────────

    /// Token mint (SCTK).
    pub token_mint: Account<'info, Mint>,

    // ─────────────────────────────────────────────────────────────
    // Access Control
    // ─────────────────────────────────────────────────────────────

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    // ─────────────────────────────────────────────────────────────
    // State & Programs
    // ─────────────────────────────────────────────────────────────

    /// TokenState account for system-wide configuration.
    pub token_state: Account<'info, TokenState>,

    /// Token program required for token transfers.
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DepositTokensContext<'info> {
    // ------------------------------------------------------------------------
    // Caller and Access Control
    // ------------------------------------------------------------------------

    /// The caller initiating the deposit (must be authorized).
    #[account(mut)]
    pub caller: Signer<'info>,

    /// Optional user access control (validated if present).
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    /// Global token configuration and state.
    pub token_state: Account<'info, TokenState>,

    // ------------------------------------------------------------------------
    // Token Mint
    // ------------------------------------------------------------------------

    /// Mint of the SCTK token.
    pub token_mint: Account<'info, Mint>,

    // ------------------------------------------------------------------------
    // Offchain Reserve Vault (source of tokens)
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for offchain reserve operations. Verified in handler.
    #[account(seeds = [b"offchain_reserve_vault"], bump)]
    pub offchain_reserve_vault: AccountInfo<'info>,

    /// Offchain reserve vault's token account.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = offchain_reserve_vault
    )]
    pub offchain_reserve_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Destination Account
    // ------------------------------------------------------------------------

    /// CHECK: Authority expected to match recipient's token account. Used for validation only.
    pub destination_authority: AccountInfo<'info>,

    /// Token account receiving the tokens.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = destination_authority
    )]
    pub destination_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Rewards Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for rewards logic. Verified in handler.
    #[account(seeds = [b"rewards_vault"], bump)]
    pub rewards_vault: AccountInfo<'info>,

    /// Rewards vault's token account.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = rewards_vault
    )]
    pub rewards_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Revenue Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for revenue logic. Verified in handler.
    #[account(seeds = [b"revenue_vault"], bump)]
    pub revenue_vault: AccountInfo<'info>,

    /// Revenue vault's token account.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = revenue_vault
    )]
    pub revenue_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Airdrop Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for airdrop logic. Verified in handler.
    #[account(seeds = [b"airdrop_vault"], bump)]
    pub airdrop_vault: AccountInfo<'info>,

    /// Airdrop vault's token account.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = airdrop_vault
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // SPL Token Program
    // ------------------------------------------------------------------------

    /// SPL Token Program.
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokensContext<'info> {
    /// The caller initiating the transfer (must be authorized).
    pub caller: Signer<'info>,

    // ------------------------------------------------------------------------
    // Rewards Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for rewards logic. Verified in handler.
    #[account(seeds = [b"rewards_vault"], bump)]
    pub rewards_vault: AccountInfo<'info>,

    /// Token account holding tokens designated for rewards.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = rewards_vault
    )]
    pub rewards_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Revenue Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for revenue logic. Verified in handler.
    #[account(seeds = [b"revenue_vault"], bump)]
    pub revenue_vault: AccountInfo<'info>,

    /// Token account holding tokens allocated for revenue collection.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = revenue_vault
    )]
    pub revenue_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Airdrop Vault
    // ------------------------------------------------------------------------

    /// CHECK: PDA authority for airdrop logic. Verified in handler.
    #[account(seeds = [b"airdrop_vault"], bump)]
    pub airdrop_vault: AccountInfo<'info>,

    /// Token account holding tokens reserved for airdrops.
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = airdrop_vault
    )]
    pub airdrop_vault_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Sender & Recipient
    // ------------------------------------------------------------------------

    /// Token account of the sender (must match mint and authority).
    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = sender
    )]
    pub sender_token_account: Account<'info, TokenAccount>,

    /// CHECK: The actual sender (must sign the transaction).
    #[account(mut, signer)]
    pub sender: AccountInfo<'info>,

    /// Token account receiving the transferred tokens.
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,

    // ------------------------------------------------------------------------
    // Misc
    // ------------------------------------------------------------------------

    /// Optional access control account for the caller.
    #[account(
        seeds = [b"user_access", caller.key().as_ref()],
        bump
    )]
    pub user_access: Option<Account<'info, UserAccessAccount>>,

    /// SPL Token mint (SCTK).
    pub token_mint: Account<'info, Mint>,

    /// Global token configuration.
    pub token_state: Account<'info, TokenState>,

    /// SPL Token program.
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct FeeDistributionContext<'info> {
    pub token_state: Account<'info, TokenState>,

    /// CHECK: Token program passed as generic account, validated at runtime.
    pub token_program: AccountInfo<'info>,

    /// CHECK: Passed as generic token account reference. Validation occurs during CPI.
    pub source_token_account: AccountInfo<'info>,

    /// CHECK: Rewards vault ATA used in token transfer logic; validated in instruction.
    pub rewards_vault_token_account: AccountInfo<'info>,

    /// CHECK: Airdrop vault ATA used in token transfer logic; validated in instruction.
    pub airdrop_vault_token_account: AccountInfo<'info>,

    /// CHECK: Revenue vault ATA used in token transfer logic; validated in instruction.
    pub revenue_vault_token_account: AccountInfo<'info>,

    /// CHECK: Authority passed as signer and checked at runtime.
    pub authority: AccountInfo<'info>,
}
