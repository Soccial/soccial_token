import { program, authorityKeypair } from "./utils/env";
import { deriveSeeds, formatLogs } from "./utils/helpers";
import { describe, test } from "node:test";



// Derive seeds for the authority (owner)
const seeds = deriveSeeds(program.programId, authorityKeypair.publicKey);

describe("Test Token State", () => {
  test("Test Token State", async () => {
    try {
    
      const tokenState = await program.account.tokenState.fetch(seeds.tokenState);

      console.log(tokenState);
      console.log(tokenState.core.mint.toBase58());
      

    } catch (error: any) {
      console.error("❌ Test failed!");
      formatLogs(error);
      throw error;
    }
  });
});