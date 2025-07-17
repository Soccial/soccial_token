import { createRequire } from "module";
import { Connection, PublicKey, Keypair, Transaction, VersionedTransaction } from "@solana/web3.js";
import { AnchorProvider, Wallet, Program, Idl } from "@coral-xyz/anchor";

// Load environment configuration
const env = process.env.TEST_ENV || "localnet";
const require = createRequire(import.meta.url);
const config = require(`../../test_config/${env}.config.ts`);

const TEAM1_PUBLIC_KEY = config.TEAM1_PUBLIC_KEY as PublicKey;
const TEAM2_PUBLIC_KEY = config.TEAM2_PUBLIC_KEY as PublicKey;

// Import local IDL and types
import idlJson from "../../target/idl/soccial_token.json" assert { type: "json" };
import { SoccialToken } from "../../target/types/soccial_token"; 

// Define Test Account type
type TestAccount = {
  publicKey: string;
  secretKey: number[];
};

// Destructure config values
const {
  ENVIRONMENT,
  RPC_URL,
  AUTHORITY_KEYPAIR_PATH,
  AUTHORITY_PUBLIC_KEY,
  PROGRAM_ID,
  TEST_ACCOUNTS,
  getKeypairFromSecret,
  getAuthorityKeypair,
} = config as {
  ENVIRONMENT: string;
  RPC_URL: string;
  AUTHORITY_KEYPAIR_PATH: string;
  AUTHORITY_PUBLIC_KEY: PublicKey;
  PROGRAM_ID: PublicKey;
  TEST_ACCOUNTS: Record<string, TestAccount>;
  getKeypairFromSecret: (secret: number[]) => Keypair;
  getAuthorityKeypair: () => Keypair;
};

// Custom Wallet adapter for Anchor using a Keypair
class KeypairWallet implements Wallet {
  constructor(readonly payer: Keypair) {}

  async signTransaction<T extends Transaction | VersionedTransaction>(tx: T): Promise<T> {
    if (tx instanceof Transaction) {
      tx.partialSign(this.payer);
      return tx;
    } else {
      tx.sign([this.payer]);
      return tx;
    }
  }

  async signAllTransactions<T extends Transaction | VersionedTransaction>(txs: T[]): Promise<T[]> {
    return txs.map((tx) => {
      if (tx instanceof Transaction) {
        tx.partialSign(this.payer);
      } else {
        tx.sign([this.payer]);
      }
      return tx;
    });
  }

  get publicKey() {
    return this.payer.publicKey;
  }
}

// Initialize basic Solana connections
export const connection = new Connection(RPC_URL, "confirmed");
export const authorityKeypair = getAuthorityKeypair();
export const programId = new PublicKey(PROGRAM_ID.toBase58());
export const provider = new AnchorProvider(
  connection,
  new KeypairWallet(authorityKeypair),
  { commitment: "confirmed" }
);

// Create a ready-to-use Program instance
export const program = new Program<SoccialToken>(
  idlJson as Idl,
  provider
);

// Utility to fetch a test Keypair by label
export const getTestKeypair = (label: string): Keypair => {
  const account = TEST_ACCOUNTS[label];
  if (!account) {
    throw new Error(`Test account "${label}" not found in TEST_ACCOUNTS`);
  }
  return getKeypairFromSecret(account.secretKey);
};

// The token_state PDA
export const [tokenStatePDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_state")],
  program.programId
);

// Export additional environment details
export {
  ENVIRONMENT,
  RPC_URL,
  AUTHORITY_KEYPAIR_PATH,
  AUTHORITY_PUBLIC_KEY,
  TEST_ACCOUNTS,
  TEAM1_PUBLIC_KEY,
  TEAM2_PUBLIC_KEY,
  
};

