// accounts/airdrop.ts

import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the manageAirdrop instruction.
 */
export function buildManageAirdropAccounts(seeds: AccountSeeds, caller: PublicKey, recipient: PublicKey) {
  return {
    caller,
    token_state: seeds.tokenState,
    user_access: seeds.userAccess,
    airdrop_vault: seeds.airdropVault,
    airdrop_vault_token_account: seeds.airdropVaultTokenAccount,
    mint: seeds.tokenMint,
    recipient_token_account: seeds.authorityTokenAccount, // pode mudar dependendo do contexto
    recipient,
    token_program: TOKEN_PROGRAM_ID,
  };
}
