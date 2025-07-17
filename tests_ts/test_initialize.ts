import { program, authorityKeypair } from "./utils/env";
import { deriveSeeds, formatLogs, requestMoreCompute } from "./utils/helpers";
import { buildInitializeTokenAccounts, buildInitializeEconomyAccounts, buildInitializeSplTokenAccounts, buildInitializeFoundersVestingAccounts } from "./accounts/initialize";
import { test } from "node:test";
import assert from "assert";

const seeds = deriveSeeds(program.programId, authorityKeypair.publicKey);

test("Initialize Soccial Token - Full Initialization Flow", async () => {

  try {

    // Step 1: Initialize Token
 
    const txSig1 = await program.methods
      .initializeToken()
      .accounts(buildInitializeTokenAccounts(seeds, authorityKeypair.publicKey))
      .signers([authorityKeypair])
      .rpc();
    console.log("⏳ [Step 1] initialize_token:", txSig1);
    assert.ok(txSig1, "Transaction signature should be returned for initialize_token");
    console.log("✅ [Step 1] Done");
  
    // Step 2: Initialize Economy
    const txSig2 = await program.methods
      .initializeEconomy()
      .accounts(buildInitializeEconomyAccounts(seeds, authorityKeypair.publicKey))
      .signers([authorityKeypair])
      .rpc();
    
    console.log("⏳ [Step 2] initialize_economy:", txSig2);
    assert.ok(txSig2, "Transaction signature should be returned for initialize_economy");
    console.log("✅ [Step 2] Done");
     
   
    // Step 3: Initialize SPL Token
    const txSig3 = await program.methods
      .initializeSplToken()
      .accounts(buildInitializeSplTokenAccounts(seeds, authorityKeypair.publicKey))
      .preInstructions(requestMoreCompute(600_000))
      .signers([authorityKeypair])
      .rpc();

    console.log("⏳ [Step 3] initialize_spl_token:", txSig3);
    assert.ok(txSig3, "Transaction signature should be returned for initialize_spl_token");
    console.log("✅ [Step 3] Done");
    

    // Step 4: Initialize Founders Vesting
    const txSig4 = await program.methods
      .initializeFoundersVesting()
      .accounts(buildInitializeFoundersVestingAccounts(seeds, authorityKeypair.publicKey))
      .signers([authorityKeypair])
      .rpc();

    console.log("⏳ [Step 4] initialize_founders_vesting:", txSig4);
    assert.ok(txSig4, "Transaction signature should be returned for initialize_founders_vesting");
     console.log("✅ [Step 4] Done");
    
  } catch (error: any) {
    console.error("❌ Test failed!");

    formatLogs(error);

    throw error; // Re-throw to not mark test as passed
  }
});
