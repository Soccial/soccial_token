// ===========================================================================
// Token Initialization Module for Soccial Token (SCTK)
// ---------------------------------------------------------------------------
//
// This module handles the **initial setup and configuration** of the Soccial Token contract.
// It is executed once, typically after deployment, to bootstrap the entire token system,
// including governance, staking, vesting, and all economic vaults.
//
// It includes the following key initialization flows:
//
// 1. initialize_token – Sets up core on-chain PDAs (TokenState, GovernanceState, StakingState, etc.)
// 2. initialize_economy – Creates all vault PDAs used for economic operations (staking, liquidity, treasury, etc.)
// 3. initialize_spl_token – Mints the initial token supply and distributes it to the vaults
// 4. initialize_founders_vesting – Creates vesting schedules for team members
//
// ---------------------------------------------------------------------------
// Design Philosophy
//
// - Manual Account Creation: All accounts are manually created with `invoke_signed` to provide full control.
// - Single-Run Logic: Each function enforces checks to ensure it can only run once.
// - PDA-Driven Security: Every PDA is derived consistently and verified to prevent spoofing.
// - Dev Feature Flag: If compiled with `--features devlogs`, emits structured logs for debugging and audit trails.
//
// ---------------------------------------------------------------------------
// Core Structures
//
// - TokenState – Holds the contract owner, paused state, fee config, and flags for subsystems.
// - GovernanceState – Stores proposal config and counters.
// - StakingState – Contains available staking plans and stake counters.
// - VestingState – Tracks global vesting ID and total schedules.
// - Vaults – Accounts for liquidity, staking, rewards, treasury, insurance, etc.
//
// ---------------------------------------------------------------------------
// Security
//
// - Only the contract owner can run these initialization functions.
// - All PDAs are validated before writing to prevent collisions or reinitialization.
// - SPL Mint authority is securely assigned to a program-derived PDA.
//
// ---------------------------------------------------------------------------
// Author: Paulo Rodrigues  
// Project: Soccial Token  
// Website: https://www.soccial.com/thetoken  
// ===========================================================================

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    program::invoke_signed, 
    rent::Rent, 
    system_instruction,
};
use anchor_spl::token::{self, Mint};

use crate::economics::state::FeeDistribution;
use crate::governance::GovernanceState;
use crate::initialize::InitializeErrorCode;
use crate::staking::StakingPlan;
use crate::token::*;
use crate::{  
    token::TokenState,
    staking::StakingState,
    utils::error::ErrorCode,
    initialize::context::*,
    economy::*
};
use anchor_spl::associated_token::create;
use crate::vesting::VestingState;
use anchor_lang::context::Context;

#[event]
pub struct GovernanceStateInitialized {
    pub min_vote_tokens: u64,
    pub quorum_percent: u64,        
    pub voting_duration: i64,
    pub validity_period: i64,
}

#[event]
pub struct VestingStateInitialized {
    pub total_schedules: u64,
    pub last_id: u64,
}

#[event]
pub struct StakingStateInitialized {
    pub plans: Vec<(u8, i64, u16, bool)>, 
}

#[event]
pub struct MintAuthorityPdaCreated;

#[event]
pub struct TokenMintInitialized {
    pub decimals: u8,
    pub mint: Pubkey,
}

#[event]
pub struct MintAuthorityAtaCreated;

#[event]
pub struct EconomyInitialized {
    pub caller: Pubkey,
    pub vaults: Vec<String>,
}

#[event]
pub struct SplTokenInitialized {
    pub caller: Pubkey,
    pub minted_vaults: Vec<(String, u64)>, // (vault_name, amount)
}

#[event]
pub struct TeamVestingInitialized {
    pub timestamp: i64,
    pub team1: Pubkey,
    pub team2: Pubkey,
    pub tokens_per_team: u64,
}


pub struct FoundersVestingInfo<'info> {
    pub participant: Pubkey,
    pub total_tokens: u64,
    pub vesting_account: &'info AccountInfo<'info>,
    pub vesting_token_account: &'info AccountInfo<'info>,
}

// ─────────────────────────────────────────────────────
// Initial Vault Allocations   
mod vaults {
    
    use crate::economy::TOTAL_SUPPLY;
    
    /// Initial supply allocated for DEX liquidity
    pub(crate) const INITIAL_OFFCHAIN_RESERVE_SUPPLY: u64 = TOTAL_SUPPLY * 0 / 100;

    /// Initial supply allocated for Rewards
    pub(crate) const INITIAL_REWARDS_SUPPLY: u64 = TOTAL_SUPPLY * 0 / 100;

    /// Initial supply allocated for Airdrop campaigns
    pub(crate) const INITIAL_AIRDROP_SUPPLY: u64 = TOTAL_SUPPLY * 5 / 100;

    /// Initial supply allocated for Buyback
    pub(crate) const INITIAL_BUYBACK_SUPPLY: u64 = TOTAL_SUPPLY * 0 / 100;

    /// Initial supply allocated for liquidity operations (Presale),
    pub(crate) const INITIAL_LIQUIDITY_SUPPLY: u64 = TOTAL_SUPPLY * 20 / 100;

    /// Initial supply allocated for Staking
    pub(crate) const INITIAL_STAKING_SUPPLY: u64 = TOTAL_SUPPLY * 0 / 100;

    /// Initial supply allocated for Vesting (Team)
    pub(crate) const INITIAL_VESTING_SUPPLY: u64 = TOTAL_SUPPLY * 10 / 100;

    /// Initial supply allocated for Insurance / Emergency Fund
    pub(crate) const INITIAL_INSURANCE_SUPPLY: u64 = TOTAL_SUPPLY * 10 / 100;

    /// Initial supply allocated for Treasury (marketing, partnerships, operations)
    pub(crate) const INITIAL_TREASURY_SUPPLY: u64 = TOTAL_SUPPLY * 5 / 100;

    /// Initial supply allocated for Revenue vault
    pub(crate) const INITIAL_REVENUE_SUPPLY: u64 = TOTAL_SUPPLY * 0 / 100;

    /// Reserved Supply = Total supply - (All other vaults sum)
    pub(crate) const INITIAL_RESERVED_SUPPLY: u64 =
        TOTAL_SUPPLY
        - INITIAL_OFFCHAIN_RESERVE_SUPPLY
        - INITIAL_REWARDS_SUPPLY
        - INITIAL_AIRDROP_SUPPLY
        - INITIAL_BUYBACK_SUPPLY
        - INITIAL_LIQUIDITY_SUPPLY
        - INITIAL_VESTING_SUPPLY
        - INITIAL_INSURANCE_SUPPLY
        - INITIAL_TREASURY_SUPPLY
        - INITIAL_REVENUE_SUPPLY;
}   

// ─────────────────────────────────────────────────────
// Team Vesting Parameters
pub mod team_vesting {

    use core::str::FromStr;
    use anchor_lang::prelude::Pubkey;

    /// Cliff period before tokens start unlocking (in seconds).
    /// Founders must wait 360 days before any tokens are released.
    pub const TEAM_CLIFF: i64 = 360 * 86400; // 360 days

    /// Total vesting duration after the cliff (in seconds).
    /// Tokens unlock over 540 days after the cliff period ends.
    pub const TEAM_VESTING_DURATION: i64 = 540 * 86400; // 540 days

    /// Number of release cycles during the vesting duration.
    /// In this case: 6 equal unlocks (one every 90 days).
    pub const TEAM_CYCLES: i64 = 6;

    /// Number of tokens unlocked immediately at the start.
    /// Founders receive 0 tokens upfront — full vesting applies.
    pub const INITIAL_TOKENS: u64 = 0;

    // Team account addresses listed as strings for full transparency
    pub const TEAM1_STR: &str = "E3Lx2pSj8ijBZSMz2FGaCqQ16sTSTCnSFE5WNFTbgyPH";
    pub const TEAM2_STR: &str = "En1vSHHFPxBgpcBrudov323ccx2dmLQejG7LTWzdanx7";

    pub fn team1_pubkey() -> Pubkey {
        Pubkey::from_str(TEAM1_STR).expect("Invalid TEAM1 pubkey")
    }

    pub fn team2_pubkey() -> Pubkey {
        Pubkey::from_str(TEAM2_STR).expect("Invalid TEAM2 pubkey")
    }

}


/// ===========================================================================
/// Initializes the TokenState and Core PDAs for Soccial Token
///
/// This function is the **first step** in deploying the Soccial Token contract.
/// It creates and initializes all the critical base accounts required to operate
/// the contract, including configuration state, governance, staking, vesting,
/// and mint authority PDAs.
///
/// ## Accounts Initialized:
/// - `TokenState` – Core configuration (owner, fee structure, status flags)
/// - `GovernanceState` – Proposal config, quorum rules, voting duration
/// - `StakingState` – Contains the default staking plans
/// - `VestingState` – Tracks global vesting counter and IDs
/// - `Mint Authority` – PDA used to sign mint operations
/// - `Token Mint` – SPL token definition for the Soccial Token
/// - `Mint Authority ATA` – Associated token account owned by the mint authority
///
/// ## Logic Flow:
/// - Each PDA is created manually via `invoke_signed` for full control
/// - Default values are serialized directly using `try_serialize`
/// - All bump seeds are verified against Anchor’s context
///
/// ## Constraints:
/// - Can only be executed **once**
/// - Must be executed before `initialize_economy` or `initialize_spl_token`
///
/// ## Dev Mode:
/// - If compiled with `--features devlogs`, logs are printed with config details
///
/// ## Errors:
/// - `ContractAlreadyInitialized` if `TokenState` is not empty
/// - Any invoke or serialization failure results in program error
///
/// ===========================================================================
pub(crate) fn initialize_token(ctx: &mut Context<InitializeToken>) -> Result<()> {
   
    let caller = ctx.accounts.caller.key();
   
    // -----------------------------------------
    // Initial security & logic check
    // -----------------------------------------

    let settings_info = &ctx.accounts.token_state;
    
    if !settings_info.to_account_info().data_is_empty() {
        return Err(InitializeErrorCode::ContractAlreadyInitialized.into());
    }

    let rent = Rent::get()?;

    // -----------------------------------------
    // Step 1: Create TokenState Account
    // -----------------------------------------

    let settings_rent = rent.minimum_balance(TokenState::LEN);
    let settings_bump = ctx.bumps.token_state;
    let settings_seeds: &[&[u8]] = &[b"token_state", &[settings_bump]];

    invoke_signed(
        &system_instruction::create_account(
            &caller,
            &ctx.accounts.token_state.key(),
            settings_rent,
            TokenState::LEN as u64,
            ctx.program_id,
        ),
        &[
            ctx.accounts.caller.to_account_info(),
            ctx.accounts.token_state.to_account_info(),
            ctx.accounts.system_program.to_account_info(),  
        ],
        &[settings_seeds],
    )?;

    let decimals = TOKEN_DECIMAL;
    
    // -----------------------------------------
    // Governance State Initialization
    // -----------------------------------------
    
    if ctx.accounts.governance_state.to_account_info().data_is_empty() {
        let rent = Rent::get()?.minimum_balance(GovernanceState::LEN);
    
        let governance_bump = ctx.bumps.governance_state;
        let governance_seeds: &[&[u8]] = &[b"governance_state", &[governance_bump]];

    
        invoke_signed(
            &system_instruction::create_account(
                &caller,
                &ctx.accounts.governance_state.key(),
                rent,
                GovernanceState::LEN as u64,
                ctx.program_id,
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                ctx.accounts.governance_state.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[governance_seeds],
        )?;
    
        // Initialize with default values
        let state = GovernanceState {
            last_id: 1,
            min_vote_tokens: 1_000,
            quorum_percent: 0, // 0% in the beggining
            voting_duration: 604800,   // 7 days
            validity_period: 604800,   // 7 days
        };

    
        let mut data = ctx.accounts.governance_state.try_borrow_mut_data()?;
        state.try_serialize(&mut *data)?;

        emit!(GovernanceStateInitialized {
            min_vote_tokens: state.min_vote_tokens,
            quorum_percent: state.quorum_percent,
            voting_duration: state.voting_duration,
            validity_period: state.validity_period,
        });
    
        #[cfg(feature = "devlogs")]
        {
            msg!("🗳️ GovernanceState initialized with default values");
        }
    }

    // -----------------------------------------
    // Vesting State Initialization
    // -----------------------------------------
    
    if ctx.accounts.vesting_state.to_account_info().data_is_empty() {
        let rent = Rent::get()?.minimum_balance(VestingState::LEN);
    
        let vesting_bump = ctx.bumps.vesting_state;
        let vesting_seeds: &[&[u8]] = &[b"vesting_state", &[vesting_bump]];

    
        invoke_signed(
            &system_instruction::create_account(
                &caller,
                &ctx.accounts.vesting_state.key(),
                rent,
                VestingState::LEN as u64,
                ctx.program_id,
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                ctx.accounts.vesting_state.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[vesting_seeds],
        )?;
    
        // Initialize with default values
        let state = VestingState {
            total_schedules: 0,
            last_id: 1,
        };
    
        let mut data = ctx.accounts.vesting_state.try_borrow_mut_data()?;
        state.try_serialize(&mut *data)?;

        emit!(VestingStateInitialized {
            total_schedules: state.total_schedules,
            last_id: state.last_id,
        });
    
        #[cfg(feature = "devlogs")]
        {
            msg!("🗳️ VestingState initialized with default values");
        }
    }

    // -----------------------------------------
    // Staking State Initialization
    // -----------------------------------------
    
    if ctx.accounts.staking_state.to_account_info().data_is_empty() {
        let rent = Rent::get()?.minimum_balance(StakingState::LEN);
    
        let stacking_bump = ctx.bumps.staking_state;
        let stacking_seeds: &[&[u8]] = &[b"staking_state", &[stacking_bump]];

    
        invoke_signed(
            &system_instruction::create_account(
                &caller,
                &ctx.accounts.staking_state.key(),
                rent,
                StakingState::LEN as u64,
                ctx.program_id,
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                ctx.accounts.staking_state.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[stacking_seeds],
        )?;
    
        let plans = [
            StakingPlan {
                plan_id: 1,
                lockup_duration: 30 * 86400,
                apr_bps: 66,
                active: true,
            },
            StakingPlan {
                plan_id: 2,
                lockup_duration: 90 * 86400,
                apr_bps: 200,
                active: true,
            },
            StakingPlan {
                plan_id: 3,
                lockup_duration: 180 * 86400,
                apr_bps: 420,
                active: true,
            },
            StakingPlan {
                plan_id: 4,
                lockup_duration: 365 * 86400,
                apr_bps: 850,
                active: true,
            },
            // Empty slots (up to 8)
            StakingPlan {
                plan_id: 5,
                lockup_duration: 1095 * 86400,
                apr_bps: 3000,
                active: false,
            },
            StakingPlan {
                plan_id: 6,
                lockup_duration: 0,
                apr_bps: 0,
                active: false,
            },
            StakingPlan {
                plan_id: 7,
                lockup_duration: 0,
                apr_bps: 0,
                active: false,
            },
            StakingPlan {
                plan_id: 8,
                lockup_duration: 0,
                apr_bps: 0,
                active: false,
            },
        ];

        let state = StakingState {
            total_stakes: 0,
            last_id: 1,
            plans,
        };

    
        let mut data = ctx.accounts.staking_state.try_borrow_mut_data()?;
        state.try_serialize(&mut *data)?;

        emit!(StakingStateInitialized {
            plans: state.plans.iter().map(|p| (p.plan_id, p.lockup_duration, p.apr_bps, p.active)).collect(),
        });
    
        #[cfg(feature = "devlogs")]
        {
            msg!("🗳️ StakingState initialized with default values");
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Step 2: Create Mint Authority PDA
    // ─────────────────────────────────────────────────────────────
    if ctx.accounts.mint_authority.lamports() == 0 {
        let mint_authority_bump = ctx.bumps.mint_authority;
        let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &[mint_authority_bump]];

        invoke_signed(
            &system_instruction::create_account(
                &caller,
                &ctx.accounts.mint_authority.key(),
                rent.minimum_balance(0),
                0,
                ctx.program_id,
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                ctx.accounts.mint_authority.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[mint_authority_seeds],
        )?;

        #[cfg(feature = "devlogs")]
        {
           msg!("✅ Mint Authority PDA created."); 
        }
    }

    // ─────────────────────────────────────────────────────────────
    // Step 3: Create Token Mint PDA
    // ─────────────────────────────────────────────────────────────
    if ctx.accounts.mint.lamports() == 0 {
        let mint_bump = ctx.bumps.mint;
        let mint_seeds: &[&[u8]] = &[b"token_mint", &[mint_bump]];

        invoke_signed(
            &system_instruction::create_account(
                &caller,
                &ctx.accounts.mint.key(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[mint_seeds],
        )?;

        // ─────────────────────────────────────────────────────────────
        // Step 4: Initialize the SPL Mint (This is the Soccial Token!)
        // ─────────────────────────────────────────────────────────────
        invoke_signed(
            &spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &ctx.accounts.mint.key(),
                &ctx.accounts.mint_authority.key(),
                None,
                decimals,
            )?,
            &[
                ctx.accounts.mint.to_account_info(),
                ctx.accounts.rent.to_account_info(),
            ],
            &[mint_seeds],
        )?;

        
        #[cfg(feature = "devlogs")]
        {
            msg!("✅ Token Mint PDA created and SPL Mint initialized.");
        }

        emit!(TokenMintInitialized {
            decimals, 
            mint: ctx.accounts.mint.key()
         });
        
    }

    // ─────────────────────────────────────────────────────────────
    // Step 5: Create ATA for Mint Authority
    // ─────────────────────────────────────────────────────────────
    let mint_authority_bump = ctx.bumps.mint_authority;
    let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &[mint_authority_bump]];

    create(
        CpiContext::new_with_signer(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.caller.to_account_info(),
                associated_token: ctx.accounts.authority_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[mint_authority_seeds],
        ),
    )?;


       // pub circulating_supply: u64,
    let settings = TokenState {
        core: CoreSettings {
            owner: caller,
            mint: ctx.accounts.mint.key(),
            api_authority: Pubkey::default(),
            initialized: true,
            economy_initialized: false,
            spl_initialized: false,
            team_vesting_initialized: false,
            paused: false,
            version: VersionInfo {
                major: 1,
                minor: 0,
                patch: 0,
            },
        },
        fee: FeeDistribution {
            rewards_fee_bps: fee::DEFAULT_REWARDS_FEE_BPS,
            airdrop_fee_bps: fee::DEFAULT_AIRDROP_FEE_BPS
        }
    };
    
    let account_info = ctx.accounts.token_state.to_account_info();
    let mut data = account_info.try_borrow_mut_data()?;
    settings.try_serialize(&mut *data)?;
    
    #[cfg(feature = "devlogs")]
    {
        // Debug log
        msg!("⚙️ TokenState:");
        msg!("• owner: {}", caller);
        msg!("• initialized: true");
        msg!("• economy_initialized: false");
        msg!("• total_supply: {}", total_supply);
        msg!("• airdrop_supply: {}", total_supply / 5);
        msg!("• reserved_supply: {}", total_supply / 2);
        msg!("✅ ATA for Mint Authority created and initialized.");
        msg!("✅ Soccial Token initialized with total supply: {}", total_supply);
    }


    msg!("🌐 Soccial Token (SCTK) successfully created → Mint Address: {}", ctx.accounts.mint.key());

    emit!(MintAuthorityAtaCreated {});
    
    Ok(())
}


/// ===========================================================================
/// Initializes the Core Economy Vaults for the Soccial Token
///
/// This function sets up the foundational token vaults used throughout
/// the Soccial Token economy. These vaults manage allocations for liquidity,
/// staking, airdrops, revenue, treasury, and other system components.
///
/// It must be called **after** the contract has been initialized via `initialize_token`.
///
/// ## Vaults Created:
/// - `liquidity_vault`
/// - `staking_vault`
/// - `revenue_vault`
/// - `rewards_vault`
/// - `airdrop_vault`
/// - `reserved_supply_vault`
/// - `vesting_vault`
/// - `offchain_reserve_vault`
/// - `insurance_vault`
/// - `treasury_vault`
///
/// ## Logic Flow:
/// - Checks if the token was initialized (`TokenState.core.initialized`)
/// - Checks if the economy was already initialized
/// - Iterates over all expected vaults and creates them if missing
/// - Uses `invoke_signed` for secure PDA creation
/// - Marks the economy as initialized in `TokenState`
///
/// ## Constraints:
/// - Idempotent: re-running won’t fail if vaults already exist
/// - Must be run by the contract owner
///
/// ## Dev Mode:
/// - If `devlogs` feature is enabled, prints each vault as it is initialized
///
/// ## Errors:
/// - Fails if `TokenState` is not initialized
/// - Fails if `economy_initialized` is already `true`
/// - Fails if vault creation or rent calculation fails
///
/// ===========================================================================
#[access_control(ctx.accounts.validate())]
pub(crate) fn initialize_economy(ctx: &mut Context<InitializeEconomy>) -> Result<()> {
   
  
    let settings_info = &ctx.accounts.token_state;
    
    #[cfg(feature = "devlogs")]
    {
        let caller = ctx.accounts.caller.key();
        // Debug log
        msg!("⚙️ TokenState:");
        msg!("• owner: {}", caller);
    }
    
    // -----------------------------------------
    // Contract must be initialized before use
    // -----------------------------------------
    if !settings_info.core.initialized {
        return Err(ErrorCode::TokenNotInitialized.into());
    }

    // -----------------------------------------
    // Contract must be initialized before use
    // -----------------------------------------
    if settings_info.core.economy_initialized {
        return Err(InitializeErrorCode::ContractEconomyAlreadyInitialized.into());
    }

    //let rent = Rent::get()?;
    
    let caller = ctx.accounts.caller.key();

    // -----------------------------------------
    // Vaults Initialization
    // -----------------------------------------
    let vaults = vec![
        (b"offchain_reserve_vault".as_ref(), &ctx.accounts.offchain_reserve_vault, ctx.bumps.offchain_reserve_vault),
        (b"liquidity_vault".as_ref(), &ctx.accounts.liquidity_vault, ctx.bumps.liquidity_vault),
        (b"staking_vault".as_ref(), &ctx.accounts.staking_vault, ctx.bumps.staking_vault),
        (b"revenue_vault".as_ref(), &ctx.accounts.revenue_vault, ctx.bumps.revenue_vault),
        (b"rewards_vault".as_ref(), &ctx.accounts.rewards_vault, ctx.bumps.rewards_vault),
        (b"airdrop_vault".as_ref(), &ctx.accounts.airdrop_vault, ctx.bumps.airdrop_vault),
        (b"reserved_supply_vault".as_ref(), &ctx.accounts.reserved_supply_vault, ctx.bumps.reserved_supply_vault),
        (b"vesting_vault".as_ref(), &ctx.accounts.vesting_vault, ctx.bumps.vesting_vault),
        (b"treasury_vault".as_ref(), &ctx.accounts.treasury_vault, ctx.bumps.treasury_vault),
        (b"insurance_vault".as_ref(), &ctx.accounts.insurance_vault, ctx.bumps.insurance_vault),
    ];

    for (seed, vault_account, bump) in vaults {
        if vault_account.lamports() == 0 {
            let rent_lamports = Rent::get()?.minimum_balance(0);
            let signer: &[&[&[u8]]] = &[&[seed, &[bump]]]; 

            invoke_signed(
                &system_instruction::create_account(
                    &caller,
                    &vault_account.key(),
                    rent_lamports,
                    0,
                    ctx.program_id,
                ),
                &[
                    ctx.accounts.caller.to_account_info(),
                    vault_account.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                signer,
            )?;
        }
        #[cfg(feature = "devlogs")]
        {
            let seed_str = core::str::from_utf8(seed).unwrap_or("UnknownVault");
            let formatted = seed_str
                .replace("_", " ")     
                .split_whitespace()       
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<String>>()
                .join("");
        
            msg!("✅ {} PDA created and initialized.", formatted);
        }
        
    }

    ctx.accounts.token_state.core.economy_initialized = true;


    msg!("✅ Economy components initialized.");

    let created_vaults = vec![
        "OffchainReserveVault",
        "LiquidityVault",
        "StakingVault",
        "RevenueVault",
        "RewardsVault",
        "AirdropVault",
        "ReservedSupplyVault",
        "VestingVault",
        "TreasuryVault",
        "InsuranceVault",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<String>>();

    emit!(EconomyInitialized {
        caller,
        vaults: created_vaults,
    });


    Ok(())
}

/// Initializes the SPL Token mint and internal token state.
/// This must be called after the contract and economy are initialized,
/// and it can only be called once.
#[access_control(ctx.accounts.validate())]
pub(crate) fn initialize_spl_token(
    ctx: Context<InitializeSplToken>
) -> Result<()> {
    let settings_info = &ctx.accounts.token_state;

    #[cfg(feature = "devlogs")]
    {
        let caller = ctx.accounts.caller.key();
        msg!("⚙️ TokenState:");
        msg!("• owner: {}", caller);
    }

    // ─────────────────────────────────────────────────────────────
    // Contract initialization checks
    // ─────────────────────────────────────────────────────────────
    if !settings_info.core.initialized {
        return Err(ErrorCode::TokenNotInitialized.into());
    }

    if !settings_info.core.economy_initialized {
        return Err(ErrorCode::TokenEconomyNotInitialized.into());
    }

    if settings_info.core.spl_initialized {
        return Err(InitializeErrorCode::SplTokenAlreadyExists.into());
    }

    // ─────────────────────────────────────────────────────────────
    // Step 1: Create fallback contract_token_account (ATA of contract_token_owner PDA)
    // ─────────────────────────────────────────────────────────────
    // This fallback account ensures that if any user mistakenly sends SCTK tokens
    // to the program (contract address), the tokens end up in a recoverable vault.
    //
    // The ATA is created for a PDA called "contract_token_owner", which acts as
    // the owner of the account — since programs can't own token accounts directly.
    //
    // This ATA will later be used to recover any tokens erroneously sent to the
    // program itself (e.g., via UI/user error or external transfer).

    create(
        CpiContext::new_with_signer(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.caller.to_account_info(),
                associated_token: ctx.accounts.contract_token_account.to_account_info(),
                authority: ctx.accounts.contract_token_owner.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &[&[b"contract_token_owner", &[ctx.bumps.contract_token_owner]]],
        ),
    )?;

    #[cfg(feature = "devlogs")]
    {
        msg!("✅ contract_token_account (ATA) created for PDA 'contract_token_owner'");
    }

    // ─────────────────────────────────────────────────────────────
    // Step 2: Create Vault Token Accounts (ATA for each vault)
    // ─────────────────────────────────────────────────────────────

    let mut minted_vaults = Vec::new();
    let vaults = vec![
        (&ctx.accounts.offchain_reserve_vault, &ctx.accounts.offchain_reserve_vault_token_account, vaults::INITIAL_OFFCHAIN_RESERVE_SUPPLY, b"offchain_reserve_vault".as_ref()),
        (&ctx.accounts.liquidity_vault, &ctx.accounts.liquidity_vault_token_account, vaults::INITIAL_LIQUIDITY_SUPPLY, b"liquidity_vault".as_ref()),
        (&ctx.accounts.staking_vault, &ctx.accounts.staking_vault_token_account, vaults::INITIAL_STAKING_SUPPLY, b"liquidity_vault".as_ref()),
        (&ctx.accounts.revenue_vault, &ctx.accounts.revenue_vault_token_account, vaults::INITIAL_REVENUE_SUPPLY, b"revenue_vault".as_ref()),
        (&ctx.accounts.rewards_vault, &ctx.accounts.rewards_vault_token_account, vaults::INITIAL_REWARDS_SUPPLY, b"rewards_vault".as_ref()),
        (&ctx.accounts.airdrop_vault, &ctx.accounts.airdrop_vault_token_account, vaults::INITIAL_AIRDROP_SUPPLY, b"airdrop_vault".as_ref()),
        (&ctx.accounts.reserved_supply_vault, &ctx.accounts.reserved_supply_vault_token_account, vaults::INITIAL_RESERVED_SUPPLY, b"reserved_supply_vault".as_ref()),
        (&ctx.accounts.vesting_vault, &ctx.accounts.vesting_vault_token_account, vaults::INITIAL_VESTING_SUPPLY, b"vesting_vault".as_ref()),
        (&ctx.accounts.insurance_vault, &ctx.accounts.insurance_vault_token_account, vaults::INITIAL_INSURANCE_SUPPLY, b"insurance_vault".as_ref()),
        (&ctx.accounts.treasury_vault, &ctx.accounts.treasury_vault_token_account, vaults::INITIAL_TREASURY_SUPPLY, b"treasury_vault".as_ref()),
    ];


    for (vault, vault_ata, initial_supply, _vault_seed) in vaults {
        create(
            CpiContext::new(
                ctx.accounts.associated_token_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: ctx.accounts.caller.to_account_info(),
                    associated_token: vault_ata.to_account_info(),
                    authority: vault.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                },
            ),
        )?;

      
            let seed_str = core::str::from_utf8(_vault_seed).unwrap_or("UnknownVault");
            let formatted = seed_str
                .replace("_", " ")
                .split_whitespace()
                .map(|word| {
                    let mut c = word.chars();
                    match c.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<String>>()
                .join("");
            
            #[cfg(feature = "devlogs")]
            { 
                msg!("✅ {} ATA created and initialized.", formatted);
            }

        // ─────────────────────────────────────────────────────────────
        // Mint initial tokens
        // ─────────────────────────────────────────────────────────────
        if initial_supply > 0 {
            let mint_authority_bump = ctx.bumps.mint_authority;
            let mint_authority_seeds: &[&[u8]] = &[b"mint_authority", &[mint_authority_bump]];
            let signer_seeds = &[mint_authority_seeds];

            let mint_to_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: vault_ata.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            );

            token::mint_to(mint_to_ctx, initial_supply)?;

            //msg!("✅ Minted {} tokens to {}", initial_supply, formatted);

            minted_vaults.push((formatted, initial_supply));

        } else {
            #[cfg(feature = "devlogs")]
            {
                msg!("⚠️ Skipped minting for {} (0 supply)", formatted);
            }
        }
        
    }

    let summary = minted_vaults
        .iter()
        .filter(|(_, amount)| *amount > 0)
        .map(|(name, amount)| format!("{}: {} units", name, amount))
        .collect::<Vec<String>>()
        .join(", ");


    msg!("✅ Vault token distribution complete → {}", summary);

    let caller = ctx.accounts.caller.key();

    emit!(SplTokenInitialized {
        caller,
        minted_vaults,
    });

    #[cfg(feature = "devlogs")]
    { 
        msg!("✅ All Vault Token Accounts created.");
    }

    // ─────────────────────────────────────────────────────────────
    // Step 3 : Update TokenState
    // ─────────────────────────────────────────────────────────────
    let token_state = &mut ctx.accounts.token_state;
    token_state.core.spl_initialized = true;


    #[cfg(feature = "devlogs")]
    {
        msg!("✅ SPL Token & Mint Initialized.");
    }


    Ok(())
}

/// ===========================================================================
/// Initializes the Team Vesting Schedules for Soccial Token
///
/// This function creates and registers vesting schedules for the core team
/// members defined in the contract. These schedules release tokens over time,
/// using a cliff and vesting duration defined in the `economy::vesting` module.
///
/// ## Logic Flow:
/// - Verifies that the base contract and SPL mint are initialized
/// - Checks that team vesting has not already been executed
/// - For each team member:
///   - Derives the correct PDA for the `VestingSchedule`
///   - Creates the vesting account manually with `invoke_signed`
///   - Writes the schedule (cliff, cycles, duration, totals) into the account
/// - Updates the `vesting_state.last_id` counter
/// - Marks `TokenState.core.team_vesting_initialized = true`
///
/// ## Team Setup:
/// - 2 vesting schedules are created (e.g., `team1` and `team2`)
/// - Each team member receives half of the `INITIAL_VESTING_SUPPLY`
///
/// ## Constraints:
/// - Must only be called once
/// - Can only be called after:
///   - `initialize_token`
///   - `initialize_economy`
///   - `initialize_spl_token`
///
/// ## Dev Mode:
/// - Emits detailed logs of which team members were assigned how many tokens
///
/// ## Errors:
/// - Fails if vesting was already initialized
/// - Fails if PDA derivation or serialization is incorrect
/// - Fails if any `invoke_signed` call fails
///
/// ===========================================================================
#[access_control(ctx.accounts.validate())]
pub(crate) fn initialize_founders_vesting(ctx: Context<InitializeFoundersVesting>) -> Result<()> {

    // -----------------------------------------
    // Step 1: Initial security & logic check
    // -----------------------------------------

    let settings_info = &ctx.accounts.token_state;
    
    if !settings_info.core.initialized {
        return Err(ErrorCode::TokenNotInitialized.into());
    }

    if !settings_info.core.economy_initialized {
        return Err(ErrorCode::TokenEconomyNotInitialized.into());
    }

    if !settings_info.core.spl_initialized {
        return Err(ErrorCode::SplNotInitialized.into());
    }

     if settings_info.core.team_vesting_initialized {
        return Err(InitializeErrorCode::TeamVestinglreadyCreate.into());
    }

    // ─────────────────────────────────────────────────────────────
    // Step 2 : Create team vesting
    // We already have the tokens on the vesting_vault.
    // We just need to create to create the vesting schedule
    // ─────────────────────────────────────────────────────────────    
    let current_time = Clock::get()?.unix_timestamp;
    let total = vaults::INITIAL_VESTING_SUPPLY / 2;

    let team1 = team_vesting::team1_pubkey();
    let team2 = team_vesting::team2_pubkey();

    let vesting_state = &mut ctx.accounts.vesting_state;

    let teammembers = vec![
        (team1, &ctx.accounts.team1_vesting_schedule, "team1_vesting_schedule"),
        (team2, &ctx.accounts.team2_vesting_schedule, "team2_vesting_schedule"),
    ];

    let mut team_vested = Vec::new();

    for (participant, vesting_account_info, _account_label) in teammembers {
        let vesting_id = vesting_state.last_id;
        
        // Derive PDA
        let (expected_key, bump) = Pubkey::find_program_address(
            &[b"vesting_schedule", participant.as_ref(), &vesting_id.to_le_bytes()],
            ctx.program_id,
        );

        msg!("{} - {} - {}", vesting_account_info.key(), expected_key, vesting_id);
        
        require!(vesting_account_info.key() == expected_key, ErrorCode::Unauthorized);

        // Create the vesting schedule account manually
        let rent = Rent::get()?;
        let space = crate::vesting::VestingSchedule::LEN;
        let lamports = rent.minimum_balance(space);

        let signer_seeds: &[&[u8]] = &[
            b"vesting_schedule",
            participant.as_ref(),
            &vesting_id.to_le_bytes(),
            &[bump],
        ];

        invoke_signed(
            &system_instruction::create_account(
                &ctx.accounts.caller.key(),
                vesting_account_info.key,
                lamports,
                space as u64,
                ctx.program_id,
            ),
            &[
                ctx.accounts.caller.to_account_info(),
                vesting_account_info.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        // Write vesting data into the account
        let mut data = vesting_account_info.try_borrow_mut_data()?;

        let vesting_schedule = crate::vesting::VestingSchedule {
            participant,
            start_time: current_time,
            cliff_duration: team_vesting::TEAM_CLIFF,
            cycles: team_vesting::TEAM_CYCLES,
            vesting_duration: team_vesting::TEAM_VESTING_DURATION,
            initial_tokens: team_vesting::INITIAL_TOKENS,
            total_tokens: total,
            released_tokens: 0,
            immutable: true,
            last_claim_time: current_time,
            vesting_id,
            status: 1,
        };

        // Serialize to account data
        vesting_schedule.serialize(&mut *data)?;

        vesting_state.last_id += 1;

        team_vested.push((participant, total));

    }

    let summary = team_vested
        .into_iter()
        .map(|(participant, amount)| format!("{}: {} tokens", participant, amount))
        .collect::<Vec<String>>()
        .join(", ");

    
    // ─────────────────────────────────────────────────────────────
    // Step 3: Update state
    // ─────────────────────────────────────────────────────────────  
 
    let token_state = &mut ctx.accounts.token_state;
    token_state.core.team_vesting_initialized = true;
    

    msg!("✅ Team vesting initialized → {}", summary);

    emit!(TeamVestingInitialized {
        timestamp: current_time,
        team1,
        team2,
        tokens_per_team: total,
    });


    // ─────────────────────────────────────────────────────────────
    // Step 4 : The Token in ready to rock!
    // ─────────────────────────────────────────────────────────────    

    msg!("🎉 Soccial Token 1.0.0 deployed successfully! A new digital economy begins — connecting people and talents, powering ideas, gamifying collaboration, and rewarding the community.");

    Ok(())
}


