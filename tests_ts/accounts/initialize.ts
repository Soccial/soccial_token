// utils/helpers.ts

import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SYSVAR_RENT_PUBKEY, SystemProgram } from "@solana/web3.js";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the initializeToken instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller (authority).
 * @returns An object containing all accounts required by initializeToken.
 */
export function buildInitializeTokenAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller: caller,
    tokenState: seeds.tokenState,
    mint: seeds.tokenMint,
    mintAuthority: seeds.mintAuthority,
    authorityTokenAccount: seeds.authorityTokenAccount,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the initializeEconomy instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller (authority).
 * @returns An object containing all accounts required by initializeEconomy.
 */
export function buildInitializeEconomyAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller: caller,
    stakingState: seeds.stakingState,
    liquidityVault: seeds.liquidityVault,
    stakingVault: seeds.stakingVault,
    reservedSupplyVault: seeds.reservedSupplyVault,
    airdropVault: seeds.airdropVault,
    vestingVault: seeds.vestingVault,
    offchainReserve: seeds.offchainReserveVault,
    revenueVault: seeds.revenueVault,
    rewardsVault: seeds.rewardsVault,
    insuranceVault: seeds.insuranceVault,
    treasuryVault: seeds.treasuryVault,
    tokenMint: seeds.tokenMint,
    tokenState: seeds.tokenState,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the initializeSplToken instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller (authority).
 * @returns An object containing all accounts required by initializeSplToken.
 */
export function buildInitializeSplTokenAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller: caller,
    mint: seeds.tokenMint,
    mintAuthority: seeds.mintAuthority,
    tokenState: seeds.tokenState,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    authorityTokenAccount: seeds.authorityTokenAccount,

    contractTokenAccount: seeds.contractTokenAccount,
    contractTokenOwner: seeds.contractTokenOwner,

    offchainReserve: seeds.offchainReserveVault,
    liquidityVault: seeds.liquidityVault,
    stakingVault: seeds.stakingVault,
    revenueVault: seeds.revenueVault,
    rewardsVault: seeds.rewardsVault,
    airdropVault: seeds.airdropVault,
    reservedSupplyVault: seeds.reservedSupplyVault,
    vestingVault: seeds.vestingVault,
    insuranceVault: seeds.insuranceVault,
    treasuryVault: seeds.treasuryVault,

    offchainReserveVaultTokenAccount: seeds.offchainReserveVaultTokenAccount,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    stakingVaultTokenAccount: seeds.stakingVaultTokenAccount,
    revenueVaultTokenAccount: seeds.revenueVaultTokenAccount,
    rewardsVaultTokenAccount: seeds.rewardsVaultTokenAccount,
    airdropVaultTokenAccount: seeds.airdropVaultTokenAccount,
    reservedSupplyVaultTokenAccount: seeds.reservedSupplyVaultTokenAccount,
    vestingVaultTokenAccount: seeds.vestingVaultTokenAccount,
    insuranceVaultTokenAccount: seeds.insuranceVaultTokenAccount,
    treasuryVaultTokenAccount: seeds.treasuryVaultTokenAccount,
  };
}

/**
 * Builds the account object for the initializeFoundersVesting instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller (contract owner).
 * @returns An object containing all accounts required by initializeFoundersVesting.
 */

export function buildInitializeFoundersVestingAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    vestingState: seeds.vestingState,
    team1VestingSchedule: seeds.team1VestingSchedule,
    team2VestingSchedule: seeds.team2VestingSchedule,
    tokenState: seeds.tokenState,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}
