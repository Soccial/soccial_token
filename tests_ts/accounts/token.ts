// accounts/contract.ts

import { PublicKey, SystemProgram } from "@solana/web3.js";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the accounts object for the manageContract instruction.
 */
export function buildManageContractAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    tokenState: seeds.tokenState,
    userAccess: seeds.userAccess,
    systemProgram: SystemProgram.programId,
  };
}

/**
 * Builds the accounts object for the manageContractGovernance instruction.
 */
export function buildManageContractGovernanceAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    tokenState: seeds.tokenState,
    userAccess: seeds.userAccess,
    proposal: seeds.proposal,
    governanceState: seeds.governanceState,
    systemProgram: SystemProgram.programId,
  };
}

/**
 * Builds the accounts object for the emitSystemLog instruction.
 */
export function buildEmitSystemLogAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
  };
}

/**
 * Builds the accounts object for the changeUpdateAuthority instruction.
 */
export function buildChangeUpdateAuthorityAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    mintAuthority: seeds.mintAuthority,
    mint: seeds.tokenMint,
    systemProgram: SystemProgram.programId,
    metadata: seeds.metadata,
    tokenMetadataProgram: seeds.tokenMetadataProgram,
    sysvarInstructions: seeds.sysvarInstructions,
    userAccess: null,
    tokenState: seeds.tokenState,
  };
}

