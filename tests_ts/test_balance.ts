import { connection, authorityKeypair, program } from "./utils/env";
import { deriveSeeds } from "./utils/helpers";
import { PublicKey } from "@solana/web3.js";
import { describe, test } from "node:test";

// Derive seeds for the authority (owner)
const seeds = deriveSeeds(program.programId, authorityKeypair.publicKey);

/**
 * Fetches the token account balance (in base units, not formatted).
 *
 * @param tokenAccount - The PublicKey of the token account.
 * @returns Promise resolving to the token amount as a string (base units).
 */
async function getTokenAccountBalance(tokenAccount: PublicKey): Promise<string> {
  const accountInfo = await connection.getParsedAccountInfo(tokenAccount);
  const data = accountInfo.value?.data;

  if (data && typeof data === "object" && "parsed" in data) {
    return data.parsed.info.tokenAmount.amount || "0"; // BASE UNITS
  } else {
    return "0";
  }
}

/**
 * Formats a number string with thousand separators.
 *
 * @param amount - The amount as a string.
 * @returns Formatted string with commas.
 */
function formatWithThousandsSeparator(amount: string): string {
  return Number(amount).toLocaleString("en-US");
}

/**
 * Pads a string to a given length.
 *
 * @param str - The string to pad.
 * @param length - The desired length.
 * @returns The padded string.
 */
function padRight(str: string, length: number): string {
  return str + " ".repeat(Math.max(length - str.length, 0));
}

describe("Vaults Balance Report", () => {
  test("Print the current balance of all vaults (formatted and aligned)", async () => {
    const vaults = [
      { name: "Offchain Reserve Vault", account: seeds.offchainReserveVaultTokenAccount },
      { name: "Liquidity Vault", account: seeds.liquidityVaultTokenAccount },
      { name: "Revenue Vault", account: seeds.revenueVaultTokenAccount },
      { name: "Rewards Vault", account: seeds.rewardsVaultTokenAccount },
      { name: "Airdrop Vault", account: seeds.airdropVaultTokenAccount },
      { name: "Vesting Vault", account: seeds.vestingVaultTokenAccount },
      { name: "Insurance Vault", account: seeds.insuranceVaultTokenAccount },
      { name: "Treasury Vault", account: seeds.treasuryVaultTokenAccount },
      { name: "Reserved Supply Vault", account: seeds.reservedSupplyVaultTokenAccount },
    ];

    const maxNameLength = vaults.reduce((max, vault) => Math.max(max, vault.name.length), 0);

    console.log("▶ Vaults Balance Report");
    console.log("──────────────────────────────────────────────");

    for (const vault of vaults) {
      try {
        // Fetch raw balance in base units
        const balanceRaw = await getTokenAccountBalance(vault.account);

        // Convert from base units to tokens (assuming a function like `unitsToTokens`)
        const balanceInTokens = unitsToTokens(balanceRaw);

        // Format the balance with thousands separator
        const formattedBalance = formatWithThousandsSeparator(balanceInTokens.toString());

        // Ensure the vault name is properly padded
        const paddedName = padRight(`💰 ${vault.name}:`, maxNameLength + 4);

        // Log the vault name and its balance in tokens
        console.log(`${paddedName} ${formattedBalance} tokens`);
      } catch (error) {
        console.error(`❌ Failed to fetch balance for ${vault.name}:`, error);
      }
    }

    console.log("──────────────────────────────────────────────");
  });
});

function unitsToTokens(units: string | number): number {
  const TOKEN_DECIMAL = 9;
  const divisor = Math.pow(10, TOKEN_DECIMAL);

  const unitsAsNumber = typeof units === "string" ? parseFloat(units) : units;

  return unitsAsNumber / divisor;
}
