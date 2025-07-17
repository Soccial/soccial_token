// accounts/vesting.ts

import { PublicKey, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the accounts object for releaseVestedTokens instruction.
 */
export function buildReleaseVestedTokensAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    tokenState: seeds.tokenState,
    userAccess: seeds.userAccess,
    vestingSchedule: seeds.vestingSchedule,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    //recipientOfLamports: seeds.recipientOfLamports,
    vestingVaultTokenAccount: seeds.vestingVaultTokenAccount,
    vestingVault: seeds.vestingVault,
    destinationTokenAccount: seeds.userTokenATA,
    tokenProgram: TOKEN_PROGRAM_ID,
    //systemProgram: seeds.systemProgram,
    clock: SYSVAR_CLOCK_PUBKEY
  };
}

/**
 * Builds the accounts object for manageVesting instruction.
 */
export function buildManageVestingAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    caller,
    participant,
    vestingSchedule: seeds.vestingSchedule,
    vestingState: seeds.vestingState,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    vestingVault: seeds.vestingVault,
    vestingVaultTokenAccount: seeds.vestingVaultTokenAccount,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    destinationTokenAccount: seeds.userTokenATA,
    //associatedTokenProgram: seeds.associatedTokenProgram,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    //systemProgram: seeds.systemProgram,
  };
}

/**
 * Builds the accounts object for editVestingSchedule instruction.
 */
export function buildEditVestingScheduleAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    caller,
    participant,
    vestingSchedule: seeds.vestingSchedule,
    vestingState: seeds.vestingState,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    vestingVault: seeds.vestingVault,
    vestingVaultTokenAccount: seeds.vestingVaultTokenAccount,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    destinationTokenAccount: seeds.userTokenATA,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    //associatedTokenProgram: seeds.associatedTokenProgram,
    //systemProgram: seeds.systemProgram,
  };
}

/**
 * Builds the accounts object for immutableVestingSchedule instruction.
 */
export function buildImmutableVestingScheduleAccounts(seeds: AccountSeeds, caller: PublicKey, participant: PublicKey) {
  return {
    caller,
    participant,
    vestingSchedule: seeds.vestingSchedule,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    //systemProgram: seeds.systemProgram,
  };
}
