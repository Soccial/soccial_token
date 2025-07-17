// accounts/vault.ts

import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the accounts object for the vaultDeposit instruction.
 */
export function buildVaultDepositAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  participant: PublicKey,
  vault: PublicKey,
  vaultTokenAccount: PublicKey,
  vaultAuthority: PublicKey
) {
  return {
    caller,
    participant,
    participantTokenAccount: seeds.userTokenATA,
    tokenMint: seeds.tokenMint,
    vaultTokenAccount,
    vault,
    vaultAuthority,
    userAccess: null,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the accounts object for the vaultWithdraw instruction.
 * This version allows specifying which vault to withdraw from.
 */
export function buildVaultWithdrawAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  vault: PublicKey,
  vaultTokenAccount: PublicKey,
  vaultAuthority: PublicKey
) {
  return {
    caller,
    vaultTokenAccount,
    userTokenAccount: seeds.userTokenATA,
    vault,
    vaultAuthority,
    userAccess: null,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the accounts object for the vaultTransfer instruction.
 */
export function buildVaultTransferAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  sourceVaultTokenAccount: PublicKey,
  sourceVault: PublicKey,
  sourceVaultAuthority: PublicKey,
  destinationVaultTokenAccount: PublicKey
) {
  return {
    caller,
    sourceVaultTokenAccount,
    sourceVault,
    sourceVaultAuthority,
    destinationVaultTokenAccount,
    proposal: seeds.proposal,
    governanceState: seeds.governanceState,
    userAccess: null,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the accounts object for the contractToVault instruction.
 */
export function buildContractToVaultAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  vault: PublicKey,
  vaultTokenAccount: PublicKey
) {
  return {
    caller,
    sourceAuthority: seeds.contractTokenOwner,
    sourceTokenAccount: seeds.contractTokenAccount,

    vaultTokenAccount,
    vault,
    userAccess: null,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}
