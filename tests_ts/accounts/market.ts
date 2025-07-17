// accounts/market.ts

import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { SystemProgram } from "@solana/web3.js";
import { AccountSeeds } from "../utils/helpers";

/**
 * Builds the account object for the buyTokens instruction.
 */
export function buildBuyTokensAccounts(seeds: AccountSeeds, caller: PublicKey) {
  return {
    caller,
    liquidityVault: seeds.liquidityVault,
    liquidityVaultTokenAccount: seeds.liquidityVaultTokenAccount,
    buyerTokenAccount: seeds.userTokenATA,
    rewardsVault: seeds.rewardsVault,
    rewardsVaultTokenAccount: seeds.rewardsVaultTokenAccount,
    revenueVault: seeds.revenueVault,
    revenueVaultTokenAccount: seeds.revenueVaultTokenAccount,
    airdropVault: seeds.airdropVault,
    airdropVaultTokenAccount: seeds.airdropVaultTokenAccount,
    tokenMint: seeds.tokenMint,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the depositTokens instruction.
 */
export function buildDepositTokensAccounts(seeds: AccountSeeds, caller: PublicKey, destinationAuthority: PublicKey) {
  return {
    caller,
    offchainReserveVault: seeds.offchainReserveVault,
    offchainReserveVaultTokenAccount: seeds.offchainReserveVaultTokenAccount,
    destinationAuthority,
    destinationTokenAccount: seeds.userTokenATA, // assuming ATA is reused
    rewardsVault: seeds.rewardsVault,
    rewardsVaultTokenAccount: seeds.rewardsVaultTokenAccount,
    revenueVault: seeds.revenueVault,
    revenueVaultTokenAccount: seeds.revenueVaultTokenAccount,
    airdropVault: seeds.airdropVault,
    airdropVaultTokenAccount: seeds.airdropVaultTokenAccount,
    tokenMint: seeds.tokenMint,
    userAccess: seeds.userAccess,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the transferTokens instruction.
 */
export function buildTransferTokensAccounts(seeds: AccountSeeds, caller: PublicKey, sender: PublicKey) {
  return {
    caller,
    rewardsVault: seeds.rewardsVault,
    rewardsVaultTokenAccount: seeds.rewardsVaultTokenAccount,
    revenueVault: seeds.revenueVault,
    revenueVaultTokenAccount: seeds.revenueVaultTokenAccount,
    airdropVault: seeds.airdropVault,
    airdropVaultTokenAccount: seeds.airdropVaultTokenAccount,
    senderTokenAccount: seeds.userTokenATA,
    sender,
    userAccess: seeds.userAccess,
    recipientTokenAccount: seeds.authorityTokenAccount, // replace as needed
    tokenMint: seeds.tokenMint,
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

/**
 * Builds the account object for the feeDistribution instruction.
 */
export function buildFeeDistributionAccounts(seeds: AccountSeeds, authority: PublicKey) {
  return {
    tokenState: seeds.tokenState,
    tokenProgram: TOKEN_PROGRAM_ID,
    sourceTokenAccount: seeds.contractTokenAccount,
    rewardsVaultTokenAccount: seeds.rewardsVaultTokenAccount,
    airdropVaultTokenAccount: seeds.airdropVaultTokenAccount,
    revenueVaultTokenAccount: seeds.revenueVaultTokenAccount,
    authority,
  };
}
