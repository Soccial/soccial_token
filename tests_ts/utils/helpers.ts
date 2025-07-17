// tests_ts/utils/helpers.ts
import { Keypair, PublicKey, Transaction, TransactionInstruction, sendAndConfirmTransaction, ComputeBudgetProgram } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { connection, programId } from "./env.js";
import { createHash } from "crypto";
import { Buffer } from "buffer";
import { TEAM1_PUBLIC_KEY, TEAM2_PUBLIC_KEY } from "./env"; 

/**
 * Sends a TransactionInstruction and waits for confirmation.
 *
 * @param instruction - The instruction to send.
 * @param signer - The signer Keypair who pays for the transaction.
 * @returns The transaction signature.
 */
export async function sendIx(
  instruction: TransactionInstruction,
  signer: Keypair
): Promise<string> {
  const tx = new Transaction().add(instruction);

  const signature = await sendAndConfirmTransaction(
    connection,
    tx,
    [signer],
    {
      skipPreflight: false,
      commitment: "confirmed",
    }
  );

  console.log(`✅ Transaction confirmed: ${signature}`);
  return signature;
}

/**
 * Calculates the 8-byte Anchor discriminator for a given instruction name.
 *
 * @param name - The instruction name (e.g., "emit_system_log").
 * @returns A Buffer with the first 8 bytes of SHA256("global:name").
 */
export function getAnchorDiscriminator(name: string): Buffer {
  const preimage = `global:${name}`;
  const hash = createHash("sha256").update(preimage).digest();
  return hash.slice(0, 8);
}

/**
 * Derives the PDA for a user_access account.
 *
 * @param authority - The authority PublicKey.
 * @returns The PDA PublicKey.
 */
export function getUserAccessPDA(authority: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("user_access"), authority.toBuffer()],
    programId
  );
  return pda;
}

/**
 * Derives the PDA for the token_state account.
 *
 * @returns The PDA PublicKey.
 */
export function getTokenStatePDA(): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_state")],
    programId
  );
  return pda;
}

/**
 * Derives a PDA synchronously using the given seeds and programId.
 *
 * @param seeds - The seeds used for PDA derivation.
 * @param programId - The program ID.
 * @returns The derived PDA PublicKey.
 */
export function findPdaSync(seeds: (Buffer | Uint8Array)[], programId: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(seeds, programId);
  return pda;
}

/**
 * Interface representing all PDA seeds and token accounts required by the Soccial Token program.
 */
export interface AccountSeeds {
  userAccess: PublicKey;
  vestingSchedule: PublicKey;
  vestingSchedulePDA: PublicKey;
  userAdopterInfo: PublicKey;
  stakingAccount: PublicKey;
  stakingState: PublicKey;
  governanceState: PublicKey;
  tokenState: PublicKey;
  tokenMint: PublicKey;
  mintAuthority: PublicKey;
  mintAuthorityBump: number;
  userTokenATA: PublicKey;
  authorityTokenAccount: PublicKey;

  contractTokenAccount: PublicKey;
  contractTokenOwner: PublicKey;


  offchainReserveVault: PublicKey;
  liquidityVault: PublicKey;
  stakingVault: PublicKey;
  revenueVault: PublicKey;
  rewardsVault: PublicKey;
  airdropVault: PublicKey;
  reservedSupplyVault: PublicKey;
  vestingVault: PublicKey;
  insuranceVault: PublicKey;
  treasuryVault: PublicKey;

  offchainReserveVaultTokenAccount: PublicKey;
  liquidityVaultTokenAccount: PublicKey;
  stakingVaultTokenAccount: PublicKey;
  revenueVaultTokenAccount: PublicKey;
  rewardsVaultTokenAccount: PublicKey;
  airdropVaultTokenAccount: PublicKey;
  reservedSupplyVaultTokenAccount: PublicKey;
  vestingVaultTokenAccount: PublicKey;
  insuranceVaultTokenAccount: PublicKey;
  treasuryVaultTokenAccount: PublicKey;

  offchainReserveVaultAuthority: PublicKey;
  liquidityVaultAuthority: PublicKey;
  stakingVaultAuthority: PublicKey;
  revenueVaultAuthority: PublicKey;
  rewardsVaultAuthority: PublicKey;
  airdropVaultAuthority: PublicKey;
  reservedSupplyVaultAuthority: PublicKey;
  vestingVaultAuthority: PublicKey;
  insuranceVaultAuthority: PublicKey;
  treasuryVaultAuthority: PublicKey;

  vestingState: PublicKey;
  team1VestingSchedule: PublicKey;
  team2VestingSchedule: PublicKey;

  proposal: PublicKey;
  metadata: PublicKey;
  tokenMetadataProgram: PublicKey;
  sysvarInstructions: PublicKey;

}

/**
 * Helper to safely derive an ATA with correct Token Program.
 */
function safeGetAta(
  mint: PublicKey,
  owner: PublicKey,
  allowOwnerOffCurve: boolean
): PublicKey {
  return getAssociatedTokenAddressSync(
    mint,
    owner,
    allowOwnerOffCurve,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
}

/**
 * Derives all required PDAs and ATAs for the Soccial Token program using fixed seeds.
 * 
 * @param programId - The program ID of the deployed Soccial Token contract.
 * @param caller - The public key of the caller (user).
 * @returns A fully populated AccountSeeds object.
 */
export function deriveSeeds(programId: PublicKey, caller: PublicKey): AccountSeeds {
  function derive(name: Buffer, extra: PublicKey | null): [PublicKey, number] {
    return extra
      ? PublicKey.findProgramAddressSync([name, extra.toBuffer()], programId)
      : PublicKey.findProgramAddressSync([name], programId);
  }

  /**
   * Derives a PDA using a custom seed array (e.g., for vesting with multiple buffers).
   */
  function deriveCustom(seeds: Buffer[]): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(seeds, programId);
  }

  // Core PDA derivations
  const [userAccess] = derive(Buffer.from("user_access"), caller);
  const [vestingSchedule] = derive(Buffer.from("vesting_schedule"), null);
  const [vestingSchedulePDA] = derive(Buffer.from("vesting_schedule"), caller);
  const [userAdopterInfo] = derive(Buffer.from("user_adopter"), caller);
  const [stakingAccount] = derive(Buffer.from("staking_account"), caller);
  const [stakingState] = derive(Buffer.from("staking_state"), null);
  const [governanceState] = derive(Buffer.from("governance_state"), null);
  const [tokenState] = derive(Buffer.from("token_state"), null);
  const [tokenMint] = derive(Buffer.from("token_mint"), null);
  const [mintAuthority, mintAuthorityBump] = derive(Buffer.from("mint_authority"), null);

  // PDA used as fallback token account owner (to receive tokens accidentally sent to the program)
  const [contractTokenOwner] = derive(Buffer.from("contract_token_owner"), null);
  const contractTokenAccount = safeGetAta(tokenMint, contractTokenOwner, true);

  // Associated Token Accounts (ATAs) - Always allowOwnerOffCurve = true when owner is a PDA
  const userTokenATA = safeGetAta(tokenMint, caller, false);
  const authorityTokenAccount = safeGetAta(tokenMint, mintAuthority, true);

  // Vault PDAs
  const [offchainReserveVault] = derive(Buffer.from("offchain_reserve_vault"), null);
  const [liquidityVault] = derive(Buffer.from("liquidity_vault"), null);
  const [stakingVault] = derive(Buffer.from("staking_vault"), null);
  const [revenueVault] = derive(Buffer.from("revenue_vault"), null);
  const [rewardsVault] = derive(Buffer.from("rewards_vault"), null);
  const [airdropVault] = derive(Buffer.from("airdrop_vault"), null);
  const [reservedSupplyVault] = derive(Buffer.from("reserved_supply_vault"), null);
  const [vestingVault] = derive(Buffer.from("vesting_vault"), null);
  const [insuranceVault] = derive(Buffer.from("insurance_vault"), null);
  const [treasuryVault] = derive(Buffer.from("treasury_vault"), null);

  // Vauls ATAs
  const offchainReserveVaultTokenAccount = safeGetAta(tokenMint, offchainReserveVault, true);
  const liquidityVaultTokenAccount = safeGetAta(tokenMint, liquidityVault, true);
  const stakingVaultTokenAccount = safeGetAta(tokenMint, stakingVault, true);
  const revenueVaultTokenAccount = safeGetAta(tokenMint, revenueVault, true);
  const rewardsVaultTokenAccount = safeGetAta(tokenMint, rewardsVault, true);
  const airdropVaultTokenAccount = safeGetAta(tokenMint, airdropVault, true);
  const reservedSupplyVaultTokenAccount = safeGetAta(tokenMint, reservedSupplyVault, true);
  const vestingVaultTokenAccount = safeGetAta(tokenMint, vestingVault, true);
  const insuranceVaultTokenAccount = safeGetAta(tokenMint, insuranceVault, true);
  const treasuryVaultTokenAccount = safeGetAta(tokenMint, treasuryVault, true);

  // Vault Authorities (optional but matching the Rust structure)
  const [offchainReserveVaultAuthority] = derive(Buffer.from("offchain_reserve_vault"), null);
  const [liquidityVaultAuthority] = derive(Buffer.from("liquidity_vault"), null);
  const [stakingVaultAuthority] = derive(Buffer.from("staking_vault"), null);
  const [revenueVaultAuthority] = derive(Buffer.from("revenue_vault"), null);
  const [rewardsVaultAuthority] = derive(Buffer.from("rewards_vault"), null);
  const [airdropVaultAuthority] = derive(Buffer.from("airdrop_vault"), null);
  const [reservedSupplyVaultAuthority] = derive(Buffer.from("reserved_supply_vault"), null);
  const [vestingVaultAuthority] = derive(Buffer.from("vesting_vault"), null);
  const [insuranceVaultAuthority] = derive(Buffer.from("insurance_vault"), null);
  const [treasuryVaultAuthority] = derive(Buffer.from("treasury_vault"), null);

  const [vestingState] = derive(Buffer.from("vesting_state"), null);

  const [proposal] = derive(Buffer.from("proposal"), null);

  const [team1VestingSchedule] = deriveVestingSchedule(TEAM1_PUBLIC_KEY, 1);
  const [team2VestingSchedule] = deriveVestingSchedule(TEAM2_PUBLIC_KEY, 2)

  // Metaplex Token Metadata program ID
  const tokenMetadataProgram = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

  // Metadata PDA (["metadata", token_metadata_program_id, mint])
  const [metadata] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      tokenMetadataProgram.toBuffer(),
      tokenMint.toBuffer(),
    ],
    tokenMetadataProgram
  );

  // Sysvar Instructions (hardcoded address)
  const sysvarInstructions = new PublicKey("Sysvar1nstructions1111111111111111111111111");

  
  return {
    userAccess,
    vestingSchedule,
    vestingSchedulePDA,
    userAdopterInfo,
    stakingAccount,
    stakingState,
    governanceState,
    tokenState,
    tokenMint,
    mintAuthority,
    mintAuthorityBump,
    userTokenATA,
    authorityTokenAccount,

    contractTokenAccount,
    contractTokenOwner,

    offchainReserveVault,
    liquidityVault,
    stakingVault,
    revenueVault,
    rewardsVault,
    airdropVault,
    reservedSupplyVault,
    vestingVault,
    insuranceVault,
    treasuryVault,

    offchainReserveVaultTokenAccount,
    liquidityVaultTokenAccount,
    stakingVaultTokenAccount,
    revenueVaultTokenAccount,
    rewardsVaultTokenAccount,
    airdropVaultTokenAccount,
    reservedSupplyVaultTokenAccount,
    vestingVaultTokenAccount,
    insuranceVaultTokenAccount,
    treasuryVaultTokenAccount,

    offchainReserveVaultAuthority,
    liquidityVaultAuthority,
    stakingVaultAuthority,
    revenueVaultAuthority,
    rewardsVaultAuthority,
    airdropVaultAuthority,
    reservedSupplyVaultAuthority,
    vestingVaultAuthority,
    insuranceVaultAuthority,
    treasuryVaultAuthority,

    vestingState,
    team1VestingSchedule,
    team2VestingSchedule,
    proposal,
    metadata,
    tokenMetadataProgram,
    sysvarInstructions,
    
  };
}

/**
 * Derives a vesting_schedule PDA for a specific participant and vesting ID.
 *
 * @param participant - The PublicKey of the team member.
 * @param vestingId - The numeric vesting ID (u64).
 * @returns A tuple with the derived PDA and bump.
 */
export function deriveVestingSchedule(participant: PublicKey, vestingId: number): [PublicKey, number] {
  const seed1 = Buffer.from("vesting_schedule");
  const seed2 = participant.toBuffer();
  const seed3 = Buffer.alloc(8); // u64 in little-endian
  seed3.writeBigUInt64LE(BigInt(vestingId));

  return PublicKey.findProgramAddressSync([seed1, seed2, seed3], programId);
}

/**
 * Formats and prints transaction logs in a readable way.
 * 
 * @param error - The error object thrown by Anchor or Solana Web3.js.
 */
export function formatLogs(error: any) {
  let logs: string[] = [];

  if (error.logs) {
    logs = error.logs;
  } else if (error.transactionLogs) {
    logs = error.transactionLogs;
  } else if (typeof error.message === 'string' && error.message.includes("\\n")) {
    logs = error.message.split("\\n").map((line: string) => line.replace(/^"+|"+$/g, ""));
  } else {
    console.error("Unknown error format:", error);
    return;
  }

  console.error("🔎 Transaction Logs:");
  for (const line of logs) {
    console.error(line.trim());
  }
}

/**
 * Creates a ComputeBudget instruction to request more compute units.
 *
 * @param units - The number of compute units to request (e.g., 400_000).
 * @param microLamports - The optional price per compute unit (default 1 μLamport).
 * @returns An array of TransactionInstructions to prepend to the transaction.
 */
export function requestMoreCompute(
  units: number,
  microLamports: number = 1
): TransactionInstruction[] {
  return [
    ComputeBudgetProgram.setComputeUnitLimit({ units }),
    ComputeBudgetProgram.setComputeUnitPrice({ microLamports }),
  ];
}

/**
 * Serializes arguments to a Buffer.
 *
 * This implementation encodes the arguments as a JSON array.
 * Adjust the format if your smart contract expects something different.
 *
 * @param args - An array or object representing the instruction arguments.
 * @returns A Buffer containing the serialized arguments.
 */
export function serializeArgs(args: any): Buffer {
  return Buffer.from(JSON.stringify(args), "utf-8");
}
