import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the CreateUserAta instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param payer - The PublicKey paying for the ATA creation.
 * @param user - The PublicKey of the user receiving the ATA.
 * @returns An object containing all accounts required by CreateUserAta.
 */
export function buildCreateUserAtaAccounts(seeds: AccountSeeds, payer: PublicKey, user: PublicKey) {
  return {
    payer: payer,
    user: user,
    mint: seeds.tokenMint,
    userAta: seeds.userTokenATA,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
    rent: SYSVAR_RENT_PUBKEY,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
  };
}

/**
 * Builds the account object for the ManageUser instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller managing the user.
 * @param targetUser - The PublicKey of the target user being managed.
 * @param targetUserAccess - The PDA address of the target user's access account.
 * @returns An object containing all accounts required by ManageUser.
 */
export function buildManageUserAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  targetUser: PublicKey,
  targetUserAccess: PublicKey
) {
  return {
    caller: caller,
    userAccess: null,
    targetUser: targetUser,
    targetUserAccess: targetUserAccess,
    tokenState: seeds.tokenState,
    systemProgram: SystemProgram.programId,
  };
}

/**
 * Builds the account object for the ManageUserRemove instruction.
 *
 * @param seeds - The derived PDA seeds.
 * @param caller - The PublicKey of the caller removing user access.
 * @param targetUser - The PublicKey of the target user whose access is being removed.
 * @param targetUserAccess - The PDA address of the target user's access account.
 * @returns An object containing all accounts required by ManageUserRemove.
 */
export function buildManageUserRemoveAccounts(
  seeds: AccountSeeds,
  caller: PublicKey,
  targetUser: PublicKey,
  targetUserAccess: PublicKey
) {
  return {
    caller: caller,
    userAccess: null,
    targetUser: targetUser,
    targetUserAccess: targetUserAccess,
    tokenState: seeds.tokenState,
    systemProgram: SystemProgram.programId,
  };
}
