// accounts/governance.ts

import { PublicKey, SystemProgram, SYSVAR_CLOCK_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the voteOnProposal instruction.
 */
export function buildVoteOnProposalAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  proposalKey: PublicKey,
  voteReceiptKey: PublicKey,
) {
  return {
    caller,
    governance_state: seeds.governanceState,
    proposal_account: proposalKey,
    user_access: seeds.userAccess,
    token_mint: seeds.tokenMint,
    user_token_account: seeds.userTokenATA,
    vote_receipt: voteReceiptKey,
    token_state: seeds.tokenState,
    token_program: TOKEN_PROGRAM_ID,
    system_program: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };
}

/**
 * Builds the account object for the createProposal instruction.
 */
export function buildCreateProposalAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  proposalKey: PublicKey,
) {
  return {
    caller,
    governance_state: seeds.governanceState,
    proposal_account: proposalKey,
    user_access: seeds.userAccess,
    token_state: seeds.tokenState,
    system_program: SystemProgram.programId,
  };
}

/**
 * Builds the account object for the finalizeProposal instruction.
 */
export function buildFinalizeProposalAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  proposalKey: PublicKey,
) {
  return {
    caller,
    governance_state: seeds.governanceState,
    proposal_account: proposalKey,
    user_access: seeds.userAccess,
    token_state: seeds.tokenState,
    system_program: SystemProgram.programId,
    clock: SYSVAR_CLOCK_PUBKEY,
  };
}

/**
 * Builds the account object for the manageGovernance instruction.
 */
export function buildManageGovernanceAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
) {
  return {
    caller,
    governance_state: seeds.governanceState,
    user_access: seeds.userAccess,
    token_state: seeds.tokenState,
    system_program: SystemProgram.programId,
  };
}
