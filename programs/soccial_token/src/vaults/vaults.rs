// ===========================================================================
// Vault Management Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module implements the logic for **secure token flow between users and vaults**,
// and between vaults themselves. It centralizes all deposit, withdraw, and transfer
// operations involving system PDAs like `liquidity_vault`, `rewards_vault`, etc.
//
// ---------------------------------------------------------------------------
// ## Supported Vault Operations:
// - Depositing tokens from users into system vaults
// - Withdrawing tokens from vaults into user wallets
// - Transferring tokens between system vaults
// - Moving tokens from the contract's fallback token account into a vault
//
//
// ---------------------------------------------------------------------------
// ## Vault Seed System:
// - Vaults are identified by static seeds (e.g. `"staking_vault"`, `"revenue_vault"`)  
// - Seeds are matched against known vault types via `resolve_vault_seeds()`  
// - Bump derivation is automatic and safe via PDA checks
//
//
// ---------------------------------------------------------------------------
// ## Functions:
// - `deposit`: Transfer tokens from user → vault
// - `withdraw`: Transfer tokens from vault → user
// - `transfer_between_vaults`: Vault-to-vault movement
// - `move_from_contract_to_vault`: Internal redistribution
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// License: MIT  
// ===========================================================================

use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use crate::{governance::{ProposalAccount, ProposalTypeBit}, vaults::{context::*, VaultError}};


#[event]
pub struct VaultDeposit {
    pub user: Pubkey,
    pub vault: String,
    pub amount: u64,
    pub reason: String,
}

#[event]
pub struct VaultWithdraw {
    pub user: Pubkey,
    pub vault: String,
    pub amount: u64,
    pub reason: String,
}

#[event]
pub struct VaultToVaultTransfer {
    pub amount: u64,
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[event]
pub struct ContractToVaultTransfer {
    pub amount: u64,
    pub to_vault: String,
}


pub enum VaultAction {
    Operation,
    Withdraw,
}

/// Enum defining all supported vault types in the Soccial Token system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultType {
    Airdrop,
    Insurance,
    Liquidity,
    OffchainReserve,
    ReservedSupply,
    Revenue,
    Rewards,
    Staking,
    Treasury,
    Vesting,
}

/// Static map associating VaultType ↔ seed used for PDA derivation.
static VAULT_MAP: &[(VaultType, &'static [u8])] = &[
    (VaultType::Airdrop,         b"airdrop_vault"),
    (VaultType::Insurance,       b"insurance_vault"),
    (VaultType::Liquidity,       b"liquidity_vault"),
    (VaultType::OffchainReserve, b"offchain_reserve_vault"),
    (VaultType::ReservedSupply,  b"reserved_supply_vault"),
    (VaultType::Revenue,         b"revenue_vault"),
    (VaultType::Rewards,         b"rewards_vault"),
    (VaultType::Staking,         b"staking_vault"),
    (VaultType::Treasury,        b"treasury_vault"),
    (VaultType::Vesting,         b"vesting_vault"),
];

impl VaultType {
    /// Returns the vault seed as a readable string.
    pub(crate) fn as_str(&self) -> &'static str {
        core::str::from_utf8(seed_from_vault_type(*self)).unwrap_or("unknown")
    }
}

/// Returns the vault PDA seed from a given VaultType.
fn seed_from_vault_type(vt: VaultType) -> &'static [u8] {
    VAULT_MAP.iter().find(|(k, _)| *k == vt).map(|(_, s)| *s).unwrap()
}

/// Resolves a VaultType from its PDA seed.
fn vault_type_from_seed(seed: &[u8]) -> Option<VaultType> {
    VAULT_MAP.iter().find(|(_, s)| *s == seed).map(|(vt, _)| *vt)
}

/// Detects the VaultType from the token account's authority field (PDA).
fn detect_vault_type(token_account: &TokenAccount) -> Option<VaultType> {
    let actual_authority = token_account.owner;

    for (vault_type, seed) in VAULT_MAP.iter() {
        let (expected_pda, _) = Pubkey::find_program_address(&[*seed], &crate::ID);

        if expected_pda == actual_authority {
            return Some(*vault_type);
        }
    }
    None
}

/// Describes types of vault transfers that require governance approval.
pub enum GovernanceType {
    RecoverFromAidrop,
    RecoverFromInsurance,
    RecoverFromStaking,
    RecoverFromVesting,
    TreasuryAllocation,
    AidropAllocation,
    StakingRecover,
    RewardsAllocation,
    VestingRecover,
}

/// Placeholder governance logic.
fn needs_governance(
    action: GovernanceType,
    proposal: Option<&mut Account<ProposalAccount>>,
    governance_state: &Account<crate::governance::GovernanceState>,
) -> Result<()> {

    let proposal = proposal.ok_or(VaultError::MissingProposalApproval)?;

    let expected_type = match action {
        GovernanceType::RecoverFromAidrop => ProposalTypeBit::RecoverFromAidrop,
        GovernanceType::RecoverFromInsurance => ProposalTypeBit::RecoverFromInsurance,
        GovernanceType::RecoverFromStaking => ProposalTypeBit::RecoverFromStaking,
        GovernanceType::RecoverFromVesting => ProposalTypeBit::RecoverFromVesting,
        GovernanceType::TreasuryAllocation => ProposalTypeBit::TreasuryAllocation,
        GovernanceType::AidropAllocation => ProposalTypeBit::AidropAllocation,
        GovernanceType::RewardsAllocation => ProposalTypeBit::RewardsAllocation,
        GovernanceType::StakingRecover => ProposalTypeBit::StakingRecover,
        GovernanceType::VestingRecover => ProposalTypeBit::VestingRecover,
    };

    crate::governance::require_approved_proposal(proposal, governance_state, expected_type)
}

/// Returns whether a transfer is allowed between two vaults (source → destination),
/// based on static rules or community-approved governance proposals.
///
/// ## Parameters:
/// - `source`: Source vault type
/// - `dest`: Destination vault type
/// - `proposal`: Proposal account to validate governance (if required)
/// - `governance_state`: Global governance settings
///
/// ## Returns:
/// - `Ok(true)` if transfer is allowed
/// - `Ok(false)` if not allowed
/// - `Err(...)` if governance check fails
///
/// ## Notes:
/// - Static transfers return `Ok(true)` immediately
/// - Governance-controlled transfers call `needs_governance(...)`
///
/// ## Errors:
/// - `GovernanceError::*` if proposal is invalid, not finalized, or mismatched
fn is_transfer_allowed(
    source: VaultType,
    dest: VaultType,
    proposal: Option<&mut Account<ProposalAccount>>,
    governance_state: &Account<crate::governance::GovernanceState>,
) -> Result<bool> {
    
    match (source, dest) {
        // Static allowed transfers.
        // Insurance
        // Enables compensation in case of system failures 
        (VaultType::Insurance, VaultType::Rewards) => Ok(true), 
        (VaultType::Insurance, VaultType::Staking) => Ok(true),
        (VaultType::Insurance, VaultType::Vesting) => Ok(true),
        (VaultType::Insurance, VaultType::OffchainReserve) => Ok(true),
     
        // Liquidity
        (VaultType::Liquidity, VaultType::Airdrop) => Ok(true),
        (VaultType::Liquidity, VaultType::Insurance) => Ok(true),
        (VaultType::Liquidity, VaultType::ReservedSupply) => Ok(true),

        // OffchainReserve
        (VaultType::OffchainReserve, VaultType::Liquidity) => Ok(true),
        (VaultType::OffchainReserve, VaultType::ReservedSupply) => Ok(true),

        // ReservedSupply
        (VaultType::ReservedSupply, VaultType::Liquidity) => Ok(true),  
        (VaultType::ReservedSupply, VaultType::Insurance) => Ok(true),  

        // Revenue
        // This is team free resource to use it for any logic purpose
        (VaultType::Revenue, VaultType::Airdrop) => Ok(true),
        (VaultType::Revenue, VaultType::Liquidity) => Ok(true),
        (VaultType::Revenue, VaultType::ReservedSupply) => Ok(true),
        (VaultType::Revenue, VaultType::Treasury) => Ok(true),
        (VaultType::Revenue, VaultType::Insurance) => Ok(true),
        (VaultType::Revenue, VaultType::Staking) => Ok(true),
        (VaultType::Revenue, VaultType::Rewards) => Ok(true),
        (VaultType::Revenue, VaultType::Vesting) => Ok(true),
        (VaultType::Revenue, VaultType::OffchainReserve) => Ok(true),

        // Treasury
        (VaultType::Treasury, VaultType::Airdrop) => Ok(true),
        (VaultType::Treasury, VaultType::Liquidity) => Ok(true),
        (VaultType::Treasury, VaultType::ReservedSupply) => Ok(true),
        (VaultType::Treasury, VaultType::Insurance) => Ok(true),
        (VaultType::Treasury, VaultType::Rewards) => Ok(true),

        // Vesting

        ////////////////////////
        // Governance-only paths
        ////////////////////////
        
        // Ask the community for vauls allocation
        (VaultType::ReservedSupply, VaultType::Airdrop) => {
            needs_governance(GovernanceType::AidropAllocation, proposal, governance_state)?;
            Ok(true)
        },
        (VaultType::ReservedSupply, VaultType::Rewards) => {
            needs_governance(GovernanceType::RewardsAllocation, proposal, governance_state)?;
            Ok(true)
        },
        
        // Recover excessive tokens on vaults in case of system failures
        (VaultType::Airdrop, VaultType::ReservedSupply) => {
            needs_governance(GovernanceType::RecoverFromAidrop, proposal, governance_state)?;
            Ok(true)
        },
        (VaultType::Insurance, VaultType::ReservedSupply) => {
            needs_governance(GovernanceType::RecoverFromInsurance, proposal, governance_state)?;
            Ok(true)
        },
        (VaultType::Staking, VaultType::ReservedSupply) => {
            needs_governance(GovernanceType::RecoverFromStaking, proposal, governance_state)?;
            Ok(true)
        },
        (VaultType::Vesting, VaultType::ReservedSupply) => {
            needs_governance(GovernanceType::RecoverFromVesting, proposal, governance_state)?;
            Ok(true)
        },

        // Recover vaults funds caused by a miss calculation or system failure
        (VaultType::ReservedSupply, VaultType::Treasury) => {
            needs_governance(GovernanceType::TreasuryAllocation, proposal, governance_state)?;
            Ok(true)
        },

          (VaultType::ReservedSupply, VaultType::Staking) => {
            needs_governance(GovernanceType::StakingRecover, proposal, governance_state)?;
            Ok(true)
        },
        (VaultType::ReservedSupply, VaultType::Vesting) => {
            needs_governance(GovernanceType::VestingRecover, proposal, governance_state)?;
            Ok(true)
        },
        
        // All others disallowed
        _ => Ok(false),
    }
}

/// ===========================================================================
/// Resolves a vault account into its associated PDA seed and bump, depending
/// on the action type (Operation or Withdraw).
///
/// ## Behavior:
/// - Loops through all known seeds for the given action
/// - Computes expected PDA and matches it against provided account
///
/// ## Returns:
/// - Matching (seed, bump) tuple if successful
///
/// ## Errors:
/// - `VaultError::UnknownVaultType` if no seed matches the provided vault
/// ===========================================================================
pub(crate) fn resolve_vault_seeds(
    vault: &AccountInfo, 
    action: VaultAction
) -> Result<(&'static [u8], u8)> {
    let seeds = match action {
        VaultAction::Operation => vaults_seeds(),
        VaultAction::Withdraw => withdraw_seeds(),
    };

    for seed in seeds {
        let (expected_pda, bump) = Pubkey::find_program_address(&[*seed], &crate::ID);
        if vault.key() == expected_pda {
            return Ok((seed, bump));
        }
    }

    Err(VaultError::UnknownVaultType.into())
}

// ======================================================================
// Lists of known seeds per vault permission type
// ======================================================================
fn vaults_seeds() -> &'static [&'static [u8]] {
    &[
        b"airdrop_vault",
        b"insurance_vault",
        b"liquidity_vault",
        b"offchain_reserve_vault",
        b"reserved_supply_vault",
        b"revenue_vault",
        b"rewards_vault",
        b"staking_vault",
        b"treasury_vault",
        b"vesting_vault",
    ]
}

// Defines the list of vaults allowed to be accessed for direct withdrawals.
// Some vaults are intentionally excluded to enforce module-specific restrictions
// (e.g., staking/vesting modules must handle their own withdrawal logic).
//
// Vaults protected by design:
//
// - "airdrop_vault": Can only be distributed by airdrop.distribute() method.
// - "offchain_reserve_vault": Used exclusively for offchain marker operations.
//   Tokens from here are later routed to liquidity or reserved_supply vaults.
//
// - "staking_vault": Withdrawals must go through the staking module.
//
// - "vesting_vault": Withdrawals must be handled via the vesting module.
//
// - "reserved_supply_vault": This vault is intended to fund internal token
//   operations and is not meant for direct withdrawals.
fn withdraw_seeds() -> &'static [&'static [u8]] {
    &[
        b"insurance_vault",
        b"liquidity_vault",
        b"revenue_vault",
        b"rewards_vault",
        b"treasury_vault",
    ]
}

/// ===========================================================================
/// Deposits tokens from a user's SPL token account into the specified system vault.
///
/// ## Use Case:
/// - Adds liquidity, funds rewards, or performs treasury operations
///
/// ## Behavior:
/// - Validates amount
/// - Verifies vault via `resolve_vault_seeds`
/// - Transfers tokens via CPI
///
/// ## Logs:
/// - Emits a message with vault name and optional reason string
///
/// ## Errors:
/// - `InvalidVaultAmount` if amount is 0 or negative
/// - `UnknownVaultType` if vault doesn't match expected seeds
/// ===========================================================================

pub(crate) fn deposit<'info>(
    ctx: Context<VaultDepositContext>,
    amount: u64,
    reason: Option<String>,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidVaultAmount);

    let (seed, _) = resolve_vault_seeds(&ctx.accounts.vault, VaultAction::Operation)?;
    let vault_name = core::str::from_utf8(seed).unwrap_or("unknown");

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.participant_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.participant.to_account_info(),
        },
    );

    token::transfer(cpi_ctx, amount)?;

    let log_reason = reason.as_deref().unwrap_or("N/A");

    msg!("💰 Deposit of {} tokens into vault '{}' | Reason: {}", amount, vault_name, log_reason);

    emit!(VaultDeposit {
        user: ctx.accounts.participant.key(),
        vault: vault_name.to_string(),
        amount,
        reason: log_reason.to_string(),
    });

    Ok(())
}

/// ===========================================================================
/// Withdraws tokens from a vault (PDA) back to a user’s SPL token account.
///
/// ## Use Case:
/// - Paying staking rewards, airdrops, or redeeming reserves
///
/// ## Behavior:
/// - Validates amount and vault balance
/// - Validates vault PDA via `resolve_vault_seeds`
/// - Performs CPI transfer with signer seeds
///
/// ## Logs:
/// - Emits message with vault name and optional reason
///
/// ## Errors:
/// - `InvalidVaultAmount`, `InsufficientVaultBalance`
/// - `UnknownVaultType` if vault is not valid
/// ===========================================================================
pub(crate) fn withdraw<'info>(
    ctx: Context<VaultWithdrawContext>,
    amount: u64,
    reason: Option<String>,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidVaultAmount);
    require!(
        ctx.accounts.vault_token_account.amount >= amount,
        VaultError::InsufficientVaultBalance
    );

    let (seed, bump) = resolve_vault_seeds(&ctx.accounts.vault, VaultAction::Withdraw)?;
    let vault_name = core::str::from_utf8(seed).unwrap_or("unknown");
    

    let signer_seeds: &[&[u8]] = &[seed, &[bump]];
    let signer_seeds_nested: &[&[&[u8]]] = &[signer_seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.clone(),
        },
        signer_seeds_nested,
    );

    token::transfer(cpi_ctx, amount)?;

    let log_reason = reason.as_deref().unwrap_or("N/A");

    msg!("🏦 Withdraw of {} tokens from vault '{}' | Reason: {}", amount, vault_name, log_reason);

    emit!(VaultWithdraw {
        user: ctx.accounts.user_token_account.owner,
        vault: vault_name.to_string(),
        amount,
        reason: log_reason.to_string(),
    });

    Ok(())
}


/// ===========================================================================
/// Transfers tokens between two system vaults, enforcing vault policy rules.
///
/// ## Use Case:
/// - System-level rebalancing of token allocations between vaults
/// - Moving tokens across functional modules (e.g., from Revenue to Treasury)
///
/// ## Behavior:
/// - Validates that source and destination vaults are distinct
/// - Verifies source vault using `resolve_vault_seeds`
/// - Detects destination vault type via token account ownership
/// - Checks if the transfer is allowed via `is_transfer_allowed`
/// - Executes CPI transfer with signer authority derived from source vault
///
/// ## Logs:
/// - Emits a log with amount, source, destination and optional reason
///
/// ## Errors:
/// - `InvalidVaultAmount` if amount is zero
/// - `InvalidItselfVaultTransfer` if transferring to the same vault
/// - `UnknownVaultType` if either vault type is unrecognized
/// - `UnauthorizedVaultTransfer` if transfer is not allowed by policy
/// ===========================================================================
pub(crate) fn transfer_between_vaults(
    ctx: Context<VaultTransferContext>,
    amount: u64,
    reason: Option<String>,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidVaultAmount);

    require!(
        ctx.accounts.source_vault_token_account.key() != ctx.accounts.destination_vault_token_account.key(),
        VaultError::InvalidItselfVaultTransfer
    );

    let (source_seed, bump) = resolve_vault_seeds(&ctx.accounts.source_vault, VaultAction::Operation)?;
    let source_type = vault_type_from_seed(source_seed).ok_or(VaultError::UnknownVaultType)?;
    let source_name = core::str::from_utf8(source_seed).unwrap_or("unknown");

    let destination_type = detect_vault_type(&ctx.accounts.destination_vault_token_account)
        .ok_or(VaultError::UnknownVaultType)?;

    
    let dest_name = destination_type.as_str();

    let is_allowed = is_transfer_allowed(
        source_type,
        destination_type,
        ctx.accounts.proposal.as_mut(), 
        &ctx.accounts.governance_state,
    )?;

    require!(is_allowed, VaultError::UnauthorizedVaultTransfer);

    let signer_seeds: &[&[u8]] = &[source_seed, &[bump]];
    let signer_seeds_nested: &[&[&[u8]]] = &[signer_seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_vault_token_account.to_account_info(),
            to: ctx.accounts.destination_vault_token_account.to_account_info(),
            authority: ctx.accounts.source_vault_authority.to_account_info(),
        },
        signer_seeds_nested,
    );

    token::transfer(cpi_ctx, amount)?;

    let log_reason = reason.as_deref().unwrap_or("N/A");

    msg!(
        "🔁 Transferring {} tokens from vault '{}' to vault '{}' | Reason: {}",
        amount,
        source_name,
        dest_name,
        log_reason
    );

    emit!(VaultToVaultTransfer {
        amount,
        from: source_name.to_string(),
        to: dest_name.to_string(),
        reason: log_reason.to_string(),
    });

    // mark governance proposal as used
    if let Some(proposal) = ctx.accounts.proposal.as_deref_mut() {
        crate::governance::mark_proposal_as_used(proposal)?;
    }


    Ok(())
}


/// ===========================================================================
/// Transfers tokens from the contract’s fallback SPL token account (owned by PDA)
/// into a known vault, using `contract_token_owner` as signer.
///
/// ## Use Case:
/// - Reclaims mistakenly sent tokens
/// - Redistributes system-held balance into economic vaults
///
/// ## Behavior:
/// - Resolves destination vault via seed
/// - Builds CPI with PDA signer: `contract_token_owner`
/// - Transfers full amount to vault’s ATA
///
/// ## Security:
/// - Only known vaults are allowed (via `vaults_seeds`)
///
/// ## Errors:
/// - Invalid amount
/// - Invalid vault seed
/// ===========================================================================
pub(crate) fn move_from_contract_to_vault(
    ctx: Context<ContractToVaultContext>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, VaultError::InvalidVaultAmount);
    
    // Resolve vault seed and bump based on the destination PDA
    let (seed, _bump) = resolve_vault_seeds(&ctx.accounts.destination_vault, VaultAction::Operation)?;
    let vault_name = core::str::from_utf8(seed).unwrap_or("unknown");

    // PDA signer: contract_token_owner
    let signer_seeds: &[&[u8]] = &[b"contract_token_owner", &[ctx.bumps.source_authority]];
    let signer_seeds_nested: &[&[&[u8]]] = &[signer_seeds];

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: ctx.accounts.destination_vault_token_account.to_account_info(),
            authority: ctx.accounts.source_authority.to_account_info(),
        },
        signer_seeds_nested,
    );

    token::transfer(cpi_ctx, amount)?;

     msg!(
        "🚚 Moved {} tokens from contract fallback account into vault '{}'",
        amount,
        vault_name
    );

    emit!(ContractToVaultTransfer {
        amount,
        to_vault: vault_name.to_string(),
    });

    Ok(())
}
