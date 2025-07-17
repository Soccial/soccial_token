import { test } from "node:test";
import assert from "assert";
import {
  connection,
  ENVIRONMENT,
  RPC_URL,
  AUTHORITY_KEYPAIR_PATH,
  AUTHORITY_PUBLIC_KEY,
  programId,
  TEST_ACCOUNTS,
  authorityKeypair,
} from "./utils/env";
import { buildInitializeTokenAccounts } from "./accounts/initialize";

test("should display loaded environment config values", async (t) => {


  console.log("🌍 ENVIRONMENT:", ENVIRONMENT);
  console.log("🔗 RPC_URL:", RPC_URL);
  console.log("🗝️ AUTHORITY_KEYPAIR_PATH:", AUTHORITY_KEYPAIR_PATH);
  console.log("👤 AUTHORITY_PUBLIC_KEY:", AUTHORITY_PUBLIC_KEY.toBase58());
  console.log("📦 PROGRAM_ID:", programId.toBase58());

  console.log("\n👥 Test Accounts:");
  for (const [label, acc] of Object.entries(TEST_ACCOUNTS)) {
    console.log(`- ${label}: ${acc.publicKey}`);
  }

  console.log("\n🔑 Loaded Authority Keypair:", authorityKeypair.publicKey.toBase58());

  // Basic assertions
  assert.ok(connection, "Connection should be defined");
  assert.ok(authorityKeypair, "Authority keypair should be loaded");
  assert.ok(programId, "Program ID should be defined");
});
