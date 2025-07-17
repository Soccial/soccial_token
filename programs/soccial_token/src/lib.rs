/// ======================================================================
/// Soccial Token (SCTK) Smart Contract
/// ======================================================================
/// 
/// This contract was created for the Soccial Token, a project aiming to revolutionize social interactions. 
/// You can find more information about the token at: https://www.soccial.com/thetoken
/// 
/// The Soccial Token will be the economic foundation of the Soccial social network, 
/// empowering users to engage, transact, and participate in a hybrid decentralized ecosystem.
/// 
/// Developed by Paulo Rodrigues - co-founder of Soccial, this contract is launched as open source to promote transparency 
/// and foster community trust. By sharing the code openly, we aim to encourage adoption and 
/// empower developers to use it in their own projects if they find it useful. 
/// 
/// Open source is about sharing, collaborating, and building something greater together. 
/// Feel free to fork it, improve it, and make it your own – Let's keep the innovation rolling!
/// 
/// However, please be aware that the contract is provided "as is", without any warranty or guarantee 
/// of functionality, reliability, or suitability for any purpose. Use it at your own risk. 
/// The author assumes no responsibility for any issues, losses, or damages resulting from its use. 
/// 
/// In the spirit of collaboration, feel free to contribute, adapt, or share. Enjoy!
/// 
/// License: MIT License
/// 
/// Author: Paulo Rodrigues
/// Project: Soccial Token
/// ======================================================================

use anchor_lang::{
    prelude::*,
    solana_program::pubkey::Pubkey,
};

pub type Result<T> = core::result::Result<T, anchor_lang::error::Error>;

// Importing system modules
pub mod utils;
pub mod auth;
pub mod initialize;
pub mod token;
pub mod market;
pub mod vesting;
pub mod airdrop;
pub mod governance;
pub mod vaults;
pub mod staking;
pub mod economics;
pub use utils::system;
use crate::airdrop::context::*;
use crate::auth::context::*;
use crate::governance::context::*;
use crate::initialize::context::*;
use crate::staking::context::*;
use crate::market::context::*;
use crate::token::context::*;
use crate::vaults::context::*;
use crate::vesting::context::*;
use crate::utils::error::ErrorCode;

declare_id!("4sbp1tZtsdUYnDjovQTyLvCCwxXZ78ifLNNLTAANV3Ci");

// System log event
#[event]
pub struct SystemLogEmitted {
    pub caller: Pubkey,
    pub message: String,
    pub timestamp: i64,
}

// 📊 Token Economy Configuration
pub mod economy {
     /// Token decimal places (standard 9 decimals)
    pub const TOKEN_DECIMAL: u8 = 9;

    /// Total supply of SCTK - 500.000.000 tokens * 10^9 = 500.000.000.000.000.000 units
    pub const TOTAL_SUPPLY: u64 = 500_000_000 * 10u64.pow(TOKEN_DECIMAL as u32);

    /// Maximum amount allowed per airdrop transaction
    pub const MAX_AIRDROP_AMOUNT: u64 = 10_000 * 10u64.pow(TOKEN_DECIMAL as u32);

    // ─────────────────────────────────────────────────────
    // Fee Distribution System
    // ─────────────────────────────────────────────────────
    //
    // This module defines the constants and logic for distributing fees collected
    // during transactions (e.g., deposits, buys, transfers) across system vaults.
    //
    // ## Fee Logic:
    // - A fee, expressed in basis points (BPS), is deducted from applicable operations.
    // - The fee is split among the following vaults:
    //   - `rewards_fee_bps` → sent to the Rewards Vault
    //   - `airdrop_fee_bps` → sent to the Airdrop Vault
    //   - The **remaining** BPS → sent to the Revenue Vault
    //
    // ## Use of Vaults:
    // - **Rewards Vault**: Used to fund staking rewards and community incentives.
    // - **Airdrop Vault**: Used to distribute tokens in promotional campaigns.
    // - **Revenue Vault**: Captures protocol revenue for operations and sustainability.
    //
    // ## BPS Reference:
    // - 1%   = 100 BPS
    // - 0.1% = 10 BPS
    // - 100% = 10_000 BPS (FEE_BPS_BASE)
    //
    // Each fee type has a defined maximum to protect the economy from misconfiguration.
    //
    // ─────────────────────────────────────────────────────
    pub mod fee {
        /// Base unit for all fee calculations in basis points (BPS).
        /// 1% = 100 BPS, so 100% = 10_000 BPS.
        pub const FEE_BPS_BASE: u16 = 10_000;

        /// Minimum allowed fee (in BPS). This also constrains the minimum values
        /// of DEFAULT_REWARDS_FEE_BPS and DEFAULT_AIRDROP_FEE_BPS.
        pub const MIN_FEE_BPS: u16 = 0;

        /// Maximum allowed fee per category (e.g., rewards, airdrop, etc.).
        /// Capped at 20% = 2000 BPS.
        pub const MAX_FEE_BPS: u16 = 2_000;

        /// Default fee sent to the rewards vault: 5% (500 BPS).
        pub const DEFAULT_REWARDS_FEE_BPS: u16 = 500;

        /// Default fee sent to the airdrop vault: 0%.
        /// No fee is collected initially since the airdrop vault is pre-funded.
        pub const DEFAULT_AIRDROP_FEE_BPS: u16 = 0;

        /// Maximum allowed reward fee: 10% (1_000 BPS).
        pub const MAX_REWARDS_FEE_BPS: u16 = 1_000;

        /// Maximum allowed airdrop fee: 5% (500 BPS).
        pub const MAX_AIRDROP_FEE_BPS: u16 = 500;
    }

}

#[program]
pub mod soccial_token {

    use crate::auth::user::ExtraFlag;

    use super::*;  

    // Allowed methods
    // Including all permissions logic in the main file to make it easy for anyone to understand the contract permissions and avaiable methods

    // ========================================================
    // Market
    // ========================================================

    /// Buys tokens from the liquidity vault and transfers to the user’s SPL account.
    ///
    /// # Args
    /// * `args[0]` – Amount to buy (u64)
    /// * `args[1]` – Fee in BPS (u16)
    ///
    /// # Permissions
    /// * Requires `buy_tokens`

    pub fn buy_tokens(
        ctx: Context<BuyTokensContext>, 
        args: Vec<String>,
    ) -> Result<()> {
        
        require_args!(args, 2)?;
        let amount = parse_arg!(args, 0, u64)?;
        let fee_bps = parse_arg!(args, 1, u16)?;
        let caller = ctx.accounts.caller.key();
        
        secure!(ctx, &caller, "buy_tokens", true);

        market::buy_tokens(ctx, amount, fee_bps)
    }

    /// Mints tokens from the Soccial Wallet (off-chain) into the user’s SPL wallet.
    ///
    /// # Arguments
    /// * `args[0]` - The amount of tokens to deposit (in base units).
    ///
    /// # Permissions
    /// * Requires `mint_tokens` permission (internal operation by Soccial backend).
    ///
    /// # Security
    /// * This function can only be triggered by the backend PDA (`contract_token_owner`),
    ///   and should be rate-limited or permissioned off-chain.
    pub fn deposit_tokens(
        ctx: Context<DepositTokensContext>, 
        args: Vec<String>
    ) -> Result<()> {
        require_args!(args, 2)?;

        let caller = ctx.accounts.caller.key();
        
        secure!(ctx, &caller, "deposit_tokens", true);

        let amount = parse_arg!(args, 0, u64)?;
        let fee_bps = parse_arg!(args, 1, u16)?;

        market::deposit_tokens(ctx, amount, fee_bps)
    }

    /// Deposits tokens from the off-chain Soccial Wallet into the user’s SPL account.
    ///
    /// # Args
    /// * `args[0]` – Amount to deposit (u64)
    /// * `args[1]` – Fee in BPS (u16)
    ///
    /// # Permissions
    /// * Requires `mint_tokens`
   pub fn transfer_tokens(
        ctx: Context<TransferTokensContext>, 
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 2)?;
        
        let caller = ctx.accounts.caller.key();
        
        secure!(ctx, &caller, "transfer_tokens", true);

        let amount = parse_arg!(args, 0, u64)?;
        let fee_bps = parse_arg!(args, 1, u16)?;

        market::transfer_tokens(ctx, amount, fee_bps)
    }

    // ========================================================
    // Vaults
    // ========================================================

    /// Deposits tokens into a system vault.
    ///
    /// # Args
    /// * `args[0]` – Amount to deposit (u64)
    ///
    /// # Permissions
    /// * Requires `deposit_vaults`
    pub fn vault_deposit<'a, 'b, 'c, 'info>(
        ctx: Context<VaultDepositContext>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        let amount = parse_arg!(args, 0, u64)?;
        let reason = args.get(1).cloned(); // Optional reason
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "deposit_vaults");

        crate::vaults::deposit(ctx, amount, reason)
    }

    /// Withdraws tokens from a system vault.
    ///
    /// # Args
    /// * `args[0]` – Amount to withdraw (u64)
    ///
    /// # Permissions
    /// * Requires `manage_vaults`
    pub fn vault_withdraw<'a, 'b, 'c, 'info>(
        ctx: Context<VaultWithdrawContext>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        let amount = parse_arg!(args, 0, u64)?;
        let reason = args.get(1).cloned(); // Optional reason

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_vaults");

        crate::vaults::withdraw(ctx, amount, reason)
    }

    /// Transfers tokens from one vault to another.
    ///
    /// # Args
    /// * `args[0]` – Amount (u64)
    /// * `args[1]` – Optional reason
    ///
    /// # Permissions
    /// * Requires `manage_vaults`
    pub fn transfer_between_vaults(
        ctx: Context<VaultTransferContext>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        let amount = parse_arg!(args, 0, u64)?;
        let reason = args.get(1).cloned(); // Optional reason

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_vaults");

        vaults::transfer_between_vaults(ctx, amount, reason)
    }

    /// Transfers tokens from contract fallback ATA to a vault.
    ///
    /// # Args
    /// * `args[0]` – Amount (u64)
    ///
    /// # Permissions
    /// * Requires `transfer_between_vaults`
    pub fn move_from_contract_to_vault(
        ctx: Context<ContractToVaultContext>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        let amount = parse_arg!(args, 0, u64)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "transfer_between_vaults");

        vaults::move_from_contract_to_vault(ctx, amount)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Vesting
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Creates a vesting schedule for a participant.
    ///
    /// # Args
    /// * `args[0]` – Pubkey of participant  
    /// * `args[1-6]` – Vesting parameters  
    /// * `args[7]` – Immutable flag
    ///
    /// # Permissions
    /// * Requires `create_vesting`
    pub fn create_vesting_schedule(
        mut ctx: Context<ManageVesting>,
        args: Vec<String>,
    ) -> Result<()> {
   
        require_args!(args, 6)?;

        let participant = parse_arg!(args, 0, Pubkey)?;
        let start_time = parse_arg!(args, 1, i64)?;
        let cliff_duration = parse_arg!(args, 2, i64)?;
        let cycles = parse_arg!(args, 3, i64)?;
        let vesting_duration = parse_arg!(args, 4, i64)?;
        let initial_tokens = parse_arg!(args, 5, u64)?;
        let total_tokens = parse_arg!(args, 6, u64)?;
        let immutable = parse_arg!(args, 7, bool)?;

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "create_vesting", true);

        vesting::create_vesting_schedule(
            &mut ctx,
            participant,
            start_time,
            cliff_duration,
            cycles,
            vesting_duration,
            initial_tokens,
            total_tokens,
            immutable
        )
    }

    /// Updates an existing vesting schedule.
    ///
    /// # Args
    /// * `args[0]` – Vesting ID  
    /// * `args[1-6]` – New vesting config  
    /// * `args[7]` – Immutable flag
    ///
    /// # Permissions
    /// * Requires `update_vesting`
    pub fn update_vesting_schedule(
        mut ctx: Context<EditVestingSchedule>,
        args: Vec<String>,
    ) -> Result<()> {
        // Expecting 3 arguments now (vesting_id + new durations)
        require_args!(args, 3)?;

        let vesting_id = parse_arg!(args, 0, u64)?;
        let start_time = parse_arg!(args, 1, i64)?;
        let cliff_duration = parse_arg!(args, 2, i64)?;
        let cycles = parse_arg!(args, 3, i64)?;
        let vesting_duration = parse_arg!(args, 4, i64)?;
        let initial_tokens = parse_arg!(args, 5, u64)?;
        let total_tokens = parse_arg!(args, 6, u64)?;
        let immutable = parse_arg!(args, 7, bool)?;

        let caller = ctx.accounts.caller.key();

        // Ensure the caller has permission to update vestings
        secure!(ctx, &caller, "update_vesting", true);

        // Call the inner vesting update logic
        vesting::update_vesting_schedule(
            &mut ctx,
            vesting_id,
            start_time,
            cliff_duration,
            cycles,
            vesting_duration,
            initial_tokens,
            total_tokens,
            immutable
        )
    }

    /// Cancels a vesting schedule.
    ///
    /// # Args
    /// * `args[0]` – Vesting ID
    ///
    /// # Permissions
    /// * Requires `manage_vesting`
    pub fn cancel_vesting_schedule(
        mut ctx: Context<EditVestingSchedule>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_vesting", true);

        let vesting_id = parse_arg!(args, 0, u64)?;

        vesting::cancel_vesting_schedule(&mut ctx, vesting_id)
    }

    /// Locks a vesting schedule as immutable.
    ///
    /// # Args
    /// * `args[0]` – Vesting ID
    ///
    /// # Permissions
    /// * Requires `manage_vesting`
    pub fn set_vesting_immutable(
        mut ctx: Context<ImmutableVestingSchedule>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_vesting", true);

        let vesting_id = parse_arg!(args, 0, u64)?;

        vesting::set_immutable(&mut ctx, vesting_id)
    }

    /// Claims vested tokens for a participant.
    ///
    /// # Permissions
    /// * Self or `manage_vesting`
    pub fn claim_vested_tokens(
        mut ctx: Context<ReleaseVestedTokens>,
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        let target = ctx.accounts.vesting_schedule.participant;
        secure_user_or_permission!(ctx, &caller, &target, "manage_vesting");
        vesting::release::release_vested_tokens(&mut ctx)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Staking
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Stakes tokens immediately after purchase using a selected plan.
    ///
    /// # Args
    /// * `args[0]` – Amount  
    /// * `args[1]` – Plan ID
    ///
    /// # Permissions
    /// * Requires `stake_tokens`
    pub fn buy_and_stake_tokens(
        ctx: Context<BuyAndStakeTokens>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 2)?;
        let amount = parse_arg!(args, 0, u64)?;
        let plan_id = parse_arg!(args, 1, u8)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "stake_tokens", true);

        staking::buy_and_stake_tokens(ctx, amount, plan_id)
    }


    /// Stakes tokens from the user's wallet using a specified plan.
    ///
    /// # Args
    /// * `args[0]` – Amount  
    /// * `args[1]` – Plan ID
    ///
    /// # Permissions
    /// * Requires `stake_tokens`
    pub fn stake_tokens(
        mut ctx: Context<StakeTokens>,
        args: Vec<String>,
    ) -> Result<()> {

        require_args!(args, 2)?;
        let amount = parse_arg!(args, 0, u64)?;
        let plan_id = parse_arg!(args, 1, u8)?;

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "stake_tokens", true);

        staking::stake_tokens(&mut ctx, amount, plan_id)
    }

    /// Adds more tokens to an existing stake (reinvestment or top-up).
    ///
    /// This allows the participant (or an authorized manager) to reinforce an active stake.
    /// The stake cycle restarts, and additional rewards are reserved from the liquidity vault.
    ///
    /// # Permissions
    /// * Self or `manage_staking`
    pub fn add_tokens_to_stake(
        mut ctx: Context<ReinforceStake>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let amount = parse_arg!(args, 0, u64)?;

        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "stake_tokens", true);

        staking::add_to_stake(&mut ctx, amount)
    }

    /// Withdraws staked tokens after the lockup ends.
    ///
    /// # Permissions
    /// * Self-only
    pub fn withdraw_staked_tokens(
        mut ctx: Context<WithdrawStaked>,
    ) -> Result<()> {

        let caller = ctx.accounts.caller.key();
        let target = ctx.accounts.staking_account.participant;

        // Only the participant can withdraw his tokens. Admin and owner don't have permissions for the action
        secure_user_only!(ctx, &caller, &target);

        staking::withdraw_staked_tokens(&mut ctx)
    }

    /// Claims staking rewards without withdrawing the stake.
    ///
    /// # Permissions
    /// * Self or `manage_staking`
    pub fn claim_staking_rewards(
        mut ctx: Context<ReleaseStaked>,
    ) -> Result<()> {

        let caller = ctx.accounts.caller.key();
        let target = ctx.accounts.staking_account.participant;

        secure_user_or_permission!(ctx, &caller, &target, "manage_staking");

        staking::claim_rewards(&mut ctx)
    }

    /// Adds a new staking plan.
    ///
    /// # Args
    /// * `args[0]` – Plan ID  
    /// * `args[1]` – Lockup duration  
    /// * `args[2]` – APR (in BPS)
    ///
    /// # Permissions
    /// * Requires `manage_contract`
    pub fn add_staking_plan(
        mut ctx: Context<ManageStaking>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 3)?;
        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_contract", true);

        let plan_id = parse_arg!(args, 0, u8)?;
        let lockup_duration = parse_arg!(args, 1, i64)?;
        let apr_bps = parse_arg!(args, 2, u16)?;

        staking::manage::add_staking_plan(&mut ctx, plan_id, lockup_duration, apr_bps)
    }

    /// Edits a staking plan.
    ///
    /// # Args
    /// * `args[0]` – Plan ID  
    /// * `args[1]` – Lockup duration  
    /// * `args[2]` – APR (in BPS)
    ///
    /// # Permissions
    /// * Requires `manage_contract`
    pub fn edit_staking_plan(
        mut ctx: Context<ManageStaking>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 3)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_contract", true);

        let plan_id = parse_arg!(args, 0, u8)?;
        let lockup_duration = parse_arg!(args, 1, i64)?;
        let apr_bps = parse_arg!(args, 2, u16)?;

        staking::manage::edit_staking_plan(&mut ctx, plan_id, lockup_duration, apr_bps)
    }

    /// Disables a staking plan.
    ///
    /// # Args
    /// * `args[0]` – Plan ID
    ///
    /// # Permissions
    /// * Requires `manage_contract`
    pub fn disable_staking_plan(
        mut ctx: Context<ManageStaking>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_contract", true);

        let plan_id = parse_arg!(args, 0, u8)?;

        staking::manage::disable_staking_plan(&mut ctx, plan_id)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Early Adopters & Whitelist
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Adds a user to early adopter phase 1 or 2.
    ///
    /// # Args
    /// * `args[0]` – Phase (1 or 2)
    ///
    /// # Permissions
    /// * Requires `manage_user`
    pub fn add_early_adopter(
        ctx: Context<ManageUser>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
      
        let caller = ctx.accounts.caller.key();
    
        secure!(ctx, &caller, "manage_user");

        let phase_raw: u8 = parse_arg!(args, 0, u8)?;
        let target_user = ctx.accounts.target_user.key();

        // Converte flag string para enum
        let flag = match phase_raw {
            1 => ExtraFlag::EarlyAdopter1,
            2 => ExtraFlag::EarlyAdopter2,
            _ => return Err(crate::auth::UserErrorCode::InvalidPhase.into()),
        };

        ctx.accounts
            .target_user_access
            .add_flag(flag, caller, target_user, &args[0])
        }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// User Flags
    //////////////////////////////////////////////////////////////////////////////////////////
    
    /// Adds a flag to the target user (e.g. EarlyAdopter1).
    ///
    /// # Args
    /// * `args[0]` – The flag name as string.
    ///
    /// # Permissions
    /// * Requires `manage_user`
    pub fn add_flag(
        ctx: Context<ManageUser>, 
        args: Vec<String>
    ) -> Result<()> {

        require_args!(args, 1)?;
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_user");

        let flag = ExtraFlag::from_str(&args[0])?;
       
        ctx.accounts.target_user_access
            .add_flag(flag, caller, ctx.accounts.target_user.key(), &args[0])
    }
    
    /// Removes a flag from the target user.
    ///
    /// # Args
    /// * `args[0]` – The flag name as string.
    ///
    /// # Permissions
    /// * Requires `manage_user`
    pub fn remove_flag(
        ctx: Context<ManageUserRemove>, 
        args: Vec<String>
    ) -> Result<()> {
        require_args!(args, 1)?;
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_user");

        let flag = ExtraFlag::from_str(&args[0])?;
        
        ctx.accounts.target_user_access
            .remove_flag(flag, caller, ctx.accounts.target_user.key(), &args[0])
    }
    
    /// Adds a user to early adopter phase 1 list.

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Governance
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Casts a vote on a proposal.
    ///
    /// # Args
    /// * `args[0]` – true (support) or false (reject)
    ///
    /// # Requirements
    /// * No permission required; based on token balance
    pub fn vote(
        ctx: Context<VoteOnProposal>,
        args: Vec<String>,
    ) -> Result<()> {

        require_args!(args, 3)?;
        let support = parse_arg!(args, 0, bool)?;

        // Total tokens held by the user in the Soccial Wallet (off-chain),
        // backed by reserved tokens in the offchain_vault.
        let total_offchain = parse_arg!(args, 1, u64)?;

        // Total tokens currently staked by the user,
        // backed by reserved tokens in the staking_vault.
        let total_staking = parse_arg!(args, 2, u64)?;
        
        let caller = ctx.accounts.caller.key();

        check!(ctx, &caller)?;

        crate::governance::voting::vote(ctx, support, total_offchain, total_staking)
    }

    /// Creates a new proposal.
    ///
    /// # Args
    /// * `args[0]` – Description (String)
    /// * `args[1..]` – Proposal types + optional [start_time, duration]
    ///
    /// # Permissions
    /// * Requires `create_proposal`
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 2)?; // description + at least one proposal type

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "create_proposal", true);

        let description: String = parse_arg!(args, 0, String)?;

        // Determine if the last 1 or 2 args are numeric (duration and/or start_time)
        let mut start_time: Option<i64> = None;
        let mut duration: Option<i64> = None;

        let mut proposal_type_strs: Vec<String> = vec![];
        let mut numeric_args: Vec<i64> = vec![];

        for arg in args.iter().skip(1) {
            if let Ok(val) = arg.parse::<i64>() {
                numeric_args.push(val);
            } else {
                proposal_type_strs.push(arg.clone());
            }
        }

        if numeric_args.len() > 0 {
            start_time = Some(numeric_args[0]);
        }
        if numeric_args.len() > 1 {
            duration = Some(numeric_args[1]);
        }   

        crate::governance::proposal::create_proposal(
            ctx,
            description,
            proposal_type_strs,
            start_time,
            duration,
        )
    }


    /// Finalizes a proposal after voting ends.
    ///
    /// # Permissions
    /// * Requires `finalize_proposal`
    pub fn finalize_proposal(
        ctx: Context<FinalizeProposal>,
    ) -> Result<()> {
        
        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "finalize_proposal", true);

        governance::proposal::finalize_proposal(ctx)
    }

    /// Updates governance settings dynamically.
    ///
    /// # Example
    /// * ["min_tokens=1000", "quorum_percent=60"]
    ///
    /// # Permissions
    /// * Requires `manage_contract`
    pub fn update_governance_settings(
        ctx: Context<ManageGovernance>,
        args: Vec<String>,
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_contract", true);

        ctx.accounts.governance_state.apply_updates(args)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Airdrop
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Distributes tokens via airdrop.
    ///
    /// # Args
    /// * `args[0]` – Amount to airdrop (u64)
    /// * `args[1]` – (Optional) Reason string
    ///
    /// # Permissions
    /// * Requires `airdrop_tokens`
    pub fn airdrop(
        mut ctx: Context<ManageAirdrop>, 
        args: Vec<String>,
    ) -> Result<()> {
      
        require_args!(args, 1)?;
   
        
        let caller = ctx.accounts.caller.key();
        
        secure!(ctx, &caller, "airdrop_tokens", true);

        let amount = parse_arg!(args, 0, u64)?; 
        let reason = args.get(1).cloned(); // Optional reason

        airdrop::distribute(&mut ctx, amount, reason)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Manage Economy
    //////////////////////////////////////////////////////////////////////////////////////////
    
    /// Updates the rewards fee (in BPS).
    ///
    /// # Args
    /// * `args[0]` – New rewards fee (u16)
    ///
    /// # Permissions
    /// * Requires `manage_economy`
    /// * Requires Governance Community Approval
    pub fn update_rewards_fee(
        ctx: Context<ManageContractGovernance>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        
        let caller = ctx.accounts.caller.key();
        let new_bps = parse_arg!(args, 0, u16)?;
        secure!(ctx, &caller, "manage_economy", true);
        
        governance::require_approved_proposal(
            &mut ctx.accounts.proposal,
            &ctx.accounts.governance_state,
            governance::ProposalTypeBit::UpdateRewardFee,
        )?;
                
        ctx.accounts.token_state.fee.update_rewards_fee(new_bps)?;

        governance::mark_proposal_as_used(&mut ctx.accounts.proposal)?;
        

        Ok(())
    }

    /// Updates the airdrop fee (in BPS).
    ///
    /// # Args
    /// * `args[0]` – New airdrop fee (u16)
    ///
    /// # Permissions
    /// * Requires `manage_economy`
    /// * Requires Governance Community Approval
    pub fn update_airdrop_fee(
        ctx: Context<ManageContractGovernance>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;
        let new_bps = parse_arg!(args, 0, u16)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_economy", true); 

        // === Check if proposal is approved and valid ===
        governance::require_approved_proposal(
            &mut ctx.accounts.proposal,
            &ctx.accounts.governance_state,
            governance::ProposalTypeBit::UpdateAirdropFee,
        )?;
        
        ctx.accounts.token_state.fee.update_airdrop_fee(new_bps)?;

        governance::mark_proposal_as_used(&mut ctx.accounts.proposal)?;

        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Admin
    //////////////////////////////////////////////////////////////////////////////////////////
    
    /// Adds a new admin to the contract.
    ///
    /// # Permissions
    /// * Only callable by the contract owner.
    pub fn add_admin(
        ctx: Context<ManageUser>
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        check_owner!(ctx, &caller)?;

        let target_user = ctx.accounts.target_user.key();
        ctx.accounts.target_user_access.grant_admin(caller, target_user)?;
        
        Ok(())
    }
    
    /// Removes an admin from the contract.
    ///
    /// # Permissions
    /// * Only callable by the contract owner.
    pub fn remove_admin(
        ctx: Context<ManageUserRemove>
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        
        check_owner!(ctx, &caller)?;
        let target_user = ctx.accounts.target_user.key();

        ctx.accounts.target_user_access.revoke_admin(caller, target_user)?;    

        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Permissions
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Assigns a permission to a user.
    ///
    /// # Arguments
    /// * `args[0]` - Permission name.
    ///
    /// # Permissions
    /// * Requires `manage_permissions`.
    pub fn assign_permission(
        ctx: Context<ManageUser>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_permissions", true);

        let permission_name = parse_arg!(args, 0, String)?;
        let target_user = ctx.accounts.target_user.key();

        ctx.accounts.target_user_access.assign_permission(&permission_name, caller, target_user, &permission_name)
        
    }

    /// Removes a permission from a user.
    ///
    /// # Arguments
    /// * `args[0]` - Permission name.
    ///
    /// # Permissions
    /// * Requires `manage_permissions`.
    pub fn unsign_permission(
        ctx: Context<ManageUserRemove>,
        args: Vec<String>,
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();
        secure!(ctx, &caller, "manage_permissions", true);  

        let permission_name = parse_arg!(args, 0, String)?;    
        let target_user = ctx.accounts.target_user.key();

        ctx.accounts.target_user_access.unsign_permission(&permission_name, caller, target_user, &permission_name)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Contract Settings
    //////////////////////////////////////////////////////////////////////////////////////////

    /// Pauses the entire contract, disabling most interactions until resumed.
    ///
    /// # Permissions
    /// * Requires `manage_contract` permission.
    pub fn pause_contract(
        ctx: Context<ManageContract>,
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_contract", true);

        ctx.accounts.token_state.core.pause(caller);

        Ok(())
    }

    /// Resumes the contract after it has been paused.
    ///
    /// # Permissions
    /// * Requires `manage_contract` permission.
    pub fn resume_contract(
        ctx: Context<ManageContract>,
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();

        secure!(ctx, &caller, "manage_contract", true);

        ctx.accounts.token_state.core.resume(caller);

        Ok(()) 
    }

    /// Sets a new API authority for the contract.
    ///
    /// # Arguments
    /// * `args[0]` - New authority Pubkey (string format).
    ///
    /// # Permissions
    /// * Requires `manage_contract`.
    pub fn set_api_authority(
        ctx: Context<ManageContract>,
        args: Vec<String>,
    ) -> Result<()> {
        // Extract the caller's public key
        let caller = ctx.accounts.caller.key();

        // Ensure we have at least 1 argument (the new authority pubkey)
        require_args!(args, 1)?;

        // Parse the first argument as a Pubkey (will fail if invalid)
        let new_authority = parse_arg!(args, 0, Pubkey)?;

        // Verify the caller has the required permission to manage the contract
        secure!(ctx, &caller, "manage_contract");

        // Update the API authority in the contract settings
        ctx.accounts.token_state.core.set_api_authority(new_authority, caller);

        Ok(())
    }

    /// Updates the contract version.
    ///
    /// # Arguments
    /// * `args[0..2]` - Version: major, minor, patch (u8).
    ///
    /// # Permissions
    /// * Requires `manage_contract`.
    pub fn update_version(
        ctx: Context<ManageContract>,
        args: Vec<String>,
    ) -> Result<()> {
        
        // Ensure we have exactly 3 arguments (major, minor, patch)
        require_args!(args, 3)?;

        let caller = ctx.accounts.caller.key();
        // Parse each part of the version separately
        let major = parse_arg!(args, 0, u8)?;
        let minor = parse_arg!(args, 1, u8)?;
        let patch = parse_arg!(args, 2, u8)?;

        // Verify the caller has permission to manage the contract
        secure!(ctx, &caller, "manage_contract", true);

        // Update the version using the CoreSettings method
        ctx.accounts.token_state.core.update_version(major, minor, patch, caller);

        Ok(())
    }

    /// Creates or updates the token metadata URI on-chain.
    ///
    /// # Arguments
    /// * `args[0]` - New metadata URI (String).
    ///
    /// # Permissions
    /// * Requires contract ownership (`check_owner!`)
    ///
    /// # Behavior
    /// * If the metadata account does not exist, it will be created with default fields.
    /// * If it exists, it will update the `uri` field while keeping other data unchanged.
    pub fn update_metadata_uri(
        ctx: Context<ChangeUpdateAuthority>, 
        args: Vec<String>
    ) -> Result<()> {
        require_args!(args, 1)?;

        let caller = ctx.accounts.caller.key();
        // Parse the first argument as a Pubkey (will fail if invalid)
        let new_uri = parse_arg!(args, 0, String)?;

        // Verify the caller has permission to manage the contract
        check_owner!(ctx, &caller)?;

        crate::token::metadata::update_metadata_uri(ctx, new_uri)?;

        Ok(())
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    /// Initialize methods ordering:
    /// 1. initialize_token
    /// 2. initialize_economy
    /// 3. initialize_spl_token
    /// 4. initialize_founders_vesting
    //////////////////////////////////////////////////////////////////////////////////////////
  
    /// Initializes the Soccial Token core configuration and essential state accounts.
    ///
    /// This function must be called only once. If the accounts already exist, it will abort early.
    ///
    /// # Arguments
    /// * `args` - A vector with the total supply as the first argument.
    //#[access_control(ctx.accounts.validate())]
    pub fn initialize_token(
        mut ctx: Context<InitializeToken>, 
    ) -> Result<()> {
        
        crate::initialize::initialize::initialize_token(&mut ctx)
    }

    /// Initializes the economic state components (liquidity, governance).
    ///
    /// This function should only be called once by the initial owner.
    pub fn initialize_economy(
        mut ctx: Context<InitializeEconomy>
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();
        
        check_owner!(ctx, &caller)?;
        
        crate::initialize::initialize::initialize_economy(&mut ctx)
    }

    /// Creates and initializes a SPL Token Mint with the given number of decimals.
    ///
    /// The Mint account must be passed in already derived and uninitialized.
    /// The authority is set to the caller, and no freeze authority is set.
    ///
    /// # Arguments
    /// * `decimals` – Number of decimals for the token (commonly 6 or 9).
    ///
    /// # Errors
    /// - Fails if the Mint account already exists or cannot be created.
    /// - Fails if initialization with SPL fails.
    pub fn initialize_spl_token(
        ctx: Context<InitializeSplToken>
    ) -> Result<()> {
    
        let caller = ctx.accounts.caller.key();

        check_owner!(ctx, &caller)?;

        crate::initialize::initialize::initialize_spl_token(ctx)
    }

    /// Initializes the vesting schedule for the founding team.
    ///
    /// This function sets up the vesting accounts for the two designated founder addresses.
    /// It creates the vesting schedule PDAs, initializes them with the proper vesting config,
    /// and transfers tokens from the liquidity vault to the vesting vault.
    ///
    /// # Behavior
    /// - Creates and initializes two vesting schedule accounts (one per founder).
    /// - Each founder receives 50% of the total initial founders vesting allocation.
    /// - Tokens are locked with a cliff and linear release schedule defined in the vesting module.
    ///
    /// # Errors
    /// - Returns an error if the token contract is not yet initialized.
    /// - Returns `Unauthorized` if caller is not the contract owner.
    /// - Returns if the vesting schedule accounts are not correctly derived or already exist.
    pub fn initialize_founders_vesting(
        ctx: Context<InitializeFoundersVesting>
    ) -> Result<()> {
        let caller = ctx.accounts.caller.key();

        check_owner!(ctx, &caller)?;

        crate::initialize::initialize::initialize_founders_vesting(ctx)
    }

    //////////////////////////////////////////////////////////////////////////////////////////
    // End of Initialize methods
    ////////////////////////////////////////////////////////////////////////////////////////// 
    
    /// =======================================================
    /// EmitSystemLog – Blockchain-Level Log for System Events
    /// -------------------------------------------------------
    /// This internal utility allows trusted actors (e.g., admin,
    /// owner, or system processes) to emit structured logs on-chain.
    ///
    /// Purpose:
    /// - Leave verifiable system-level messages tied to a transaction
    /// - Announce internal operations, maintenance, or upgrades
    /// - Serve as public-facing trace points for debugging or audit
    ///
    /// Format includes a fixed prefix to prevent spoofing:
    ///   `[SOCCIAL TOKEN LOG]` ensures reliable log detection
    ///
    /// Notes:
    /// - These logs are immutable and stored permanently
    /// - Can be queried by external observers or tools
    /// - Useful for governance, transparency and protocol tracking
    /// =======================================================
    pub fn emit_system_log(
        ctx: Context<EmitSystemLog>, 
        args: Vec<String>
    ) -> Result<()> {
        // Validate input
        require_args!(args, 1)?;
        let message = parse_arg!(args, 0, String)?;

        // Check that caller has system-level query permissions
        secure!(ctx, &ctx.accounts.caller.key(), "emit_log", true);

        // Emit the log on-chain
        msg!("📢 [SOCCIAL TOKEN LOG] {}", message);

        emit!(SystemLogEmitted {
            caller: ctx.accounts.caller.key(),
            message: message.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });


        Ok(())
    }

    #[cfg(feature = "dev")]
    use anchor_spl::token::{self, MintTo};

    /// DEVELOPMENT ONLY!
    /// Mints new tokens to the user's account (for testing only).
    ///
    /// # Arguments
    /// * `args` - Vector where the first element must be the amount (u64).
    ///
    /// # Permissions
    /// * Requires `mint_tokens` permission.
    /// * Requires `feature = "dev"` enabled.
    ///
    /// # Effects
    /// * Increases the circulating supply by minting from the SPL Token.
    /// * Only available in test environments.
    #[cfg(feature = "dev")]
    pub fn mint(
        ctx: Context<MintTokens>, 
        args: Vec<String>,
    ) -> Result<()> {
        // Validate and parse arguments
        require_args!(args, 1)?;
        let amount = parse_arg!(args, 0, u64)?;

        // Prepare CPI context
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
        );

        // Signer seeds for PDA
        let seeds: &[&[u8]] = &[b"mint_authority", &[ctx.bumps.mint_authority]];
        token::mint_to(cpi_ctx.with_signer(&[seeds]), amount)?;

        msg!("🧪 Minted {} tokens to {}", amount, ctx.accounts.recipient_token_account.key());
        Ok(())
    }

}
