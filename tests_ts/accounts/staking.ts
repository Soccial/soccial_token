// accounts/staking.ts

import { PublicKey, SystemProgram, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the buyAndStakeTokens instruction.
 */
export function buildBuyAndStakeTokensAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    stakingState: seeds.stakingState,
    stakingAccount: seeds.stakingAccount,
    participant,
    tokenMint: seeds.tokenMint,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    stakingVault: seeds.stakingVault,
    caller,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the stakeTokens instruction.
 */
export function buildStakeTokensAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    caller,
    participant,
    participantTokenAccount: seeds.userTokenATA,
    stakingState: seeds.stakingState,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    stakingVault: seeds.stakingVault,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    stakingAccount: seeds.stakingAccount,
    tokenMint: seeds.tokenMint,
    destinationTokenAccount: seeds.userTokenATA,
    mintAuthority: seeds.mintAuthority,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the reinforceStake instruction.
 */
export function buildReinforceStakeAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    caller,
    participant,
    participantTokenAccount: seeds.userTokenATA,
    stakingState: seeds.stakingState,
    stakingAccount: seeds.stakingAccount,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    stakingVault: seeds.stakingVault,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    tokenMint: seeds.tokenMint,
    destinationTokenAccount: seeds.userTokenATA,
    mintAuthority: seeds.mintAuthority,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the releaseStaked instruction.
 */
export function buildReleaseStakedAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    tokenState: seeds.tokenState,
    userAccess: seeds.userAccess,
    stakingAccount: seeds.stakingAccount,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    stakingVault: seeds.stakingVault,
    destinationTokenAccount: seeds.userTokenATA,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY
  };
}

/**
 * Builds the account object for the withdrawStaked instruction.
 */
export function buildWithdrawStakedAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    tokenState: seeds.tokenState,
    userAccess: seeds.userAccess,
    stakingAccount: seeds.stakingAccount,
    //recipientOfLamports: seeds.recipient,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    stakingVault: seeds.stakingVault,
    destinationTokenAccount: seeds.userTokenATA,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY

  };
}

/**
 * Builds the account object for the manageStaking instruction.
 */
export function buildManageStakingAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    stakingState: seeds.stakingState,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
  };
}
