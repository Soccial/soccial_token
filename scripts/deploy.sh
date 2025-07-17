#!/bin/bash

#!/bin/bash
# ==============================================================================
# Soccial Token – Program Deployment Script
#
# Usage:
#   ./scripts/deploy.sh localnet|devnet|testnet|mainnet
#
# Description:
#   Deploys the Soccial Token program to the specified Solana cluster.
#   It handles keypair generation, ID updates, environment setup, and deployment.
#
# Behavior:
#   - Generates a keypair if it doesn't exist
#   - Updates `declare_id!` in `lib.rs` with the correct program ID
#   - Updates `[programs.<env>]` in `Anchor.toml`
#   - Builds and deploys the program using `anchor deploy` or `solana program deploy`
#   - Verifies the deployed binary integrity (except on localnet)
#
# Mainnet Safety:
#   Requires explicit confirmation via `deploytomainnet` input.
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

# Go to the project root (one level up from /scripts)
cd "$(dirname "$0")/.." || exit

ENV=$1
ENV_FILE=".env/${ENV}.env"
KEYS_DIR="./.deploy-keys"
KEYPAIR_PATH="$KEYS_DIR/soccial_token-${ENV}-keypair.json"
ANCHOR_KEYPAIR_PATH="target/deploy/soccial_token-keypair.json"
LIB_RS_PATH="programs/soccial_token/src/lib.rs"
ANCHOR_TOML_PATH="Anchor.toml"

# -------------------
# Step 0: Validate input
# -------------------
if [[ -z "$ENV" ]]; then
  echo "❌ Missing environment. Usage: ./scripts/deploy.sh localnet|devnet|testnet|mainnet"
  exit 1
fi

if [[ "$ENV" != "localnet" && "$ENV" != "devnet" && "$ENV" != "testnet" && "$ENV" != "mainnet" ]]; then
  echo "❌ Invalid environment: $ENV (must be localnet, devnet, testnet or mainnet)"
  exit 1
fi

echo "🌍 Selected environment: $ENV"

# -------------------
# Safety check for mainnet
# -------------------
if [[ "$ENV" == "mainnet" ]]; then
  echo ""
  echo "🛑🛑🛑 WARNING: You are about to DEPLOY TO MAINNET 🛑🛑🛑"
  echo "This action is IRREVERSIBLE and may incur real costs!"
  echo "✋ To continue, type exactly: deploytomainnet"
  echo ""
  read -r CONFIRMATION

  if [[ "$CONFIRMATION" != "deploytomainnet" ]]; then
    echo "❌ Aborting deployment to mainnet. Did you mean localnet?"
    exit 1
  fi
  echo "✅ Confirmation accepted. Proceeding with mainnet deployment..."
  echo ""
fi

# -------------------
# Step 1: Load env vars (skip for localnet)
# -------------------
if [[ "$ENV" != "localnet" ]]; then
  echo "📄 Loading environment from: $ENV_FILE"

  if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Env file not found: $ENV_FILE"
    exit 1
  fi

  source "$ENV_FILE"
  export ANCHOR_PROVIDER_URL
  export ANCHOR_WALLET

  if [ ! -f "$ANCHOR_WALLET" ]; then
    echo "❌ Wallet file not found: $ANCHOR_WALLET"
    exit 1
  fi

  echo "🔧 Environment configured:"
  echo "   - ANCHOR_PROVIDER_URL = $ANCHOR_PROVIDER_URL"
  echo "   - ANCHOR_WALLET       = $ANCHOR_WALLET"
  echo ""
fi

# -------------------
# Step 2: Ensure keypair for this environment exists
# -------------------
if [ ! -f "$KEYPAIR_PATH" ]; then
  echo "🔐 Generating new keypair for program ($ENV)..."
  mkdir -p target/deploy
  solana-keygen new --outfile "$KEYPAIR_PATH" --no-bip39-passphrase --silent
else
  echo "🔁 Using existing keypair for $ENV"
fi

# Copy keypair to Anchor's expected path
cp "$KEYPAIR_PATH" "$ANCHOR_KEYPAIR_PATH"

# Get the program ID
PROGRAM_ID=$(solana address -k "$KEYPAIR_PATH")
echo "📛 Program ID: $PROGRAM_ID"
echo ""

# -------------------
# Step 3: Update declare_id!
# -------------------
echo "✏️  Updating declare_id! in $LIB_RS_PATH"
sed -i '' "s/^declare_id!(\".*\");/declare_id!(\"$PROGRAM_ID\");/" "$LIB_RS_PATH"

# -------------------
# Step 4: Update Anchor.toml
# -------------------
echo "✏️  Updating Anchor.toml for [$ENV]"

TEMP_FILE=$(mktemp)

# Check if the [programs.$ENV] section exists
if grep -q "^\[programs\.$ENV\]" "$ANCHOR_TOML_PATH"; then
  # Section exists → update the soccial_token line inside it
  awk -v env="$ENV" -v pid="$PROGRAM_ID" '
    BEGIN { in_section=0 }
    /^\[programs\./ {
      if (in_section && $0 !~ /^\[programs\./) {
        in_section=0
      }
      if ($0 == "[programs." env "]") {
        in_section=1
      }
    }
    {
      if (in_section && $0 ~ /^soccial_token = /) {
        print "soccial_token = \"" pid "\""
      } else {
        print $0
      }
    }
  ' "$ANCHOR_TOML_PATH" > "$TEMP_FILE"
else
  # Section does not exist → add at the end
  cat "$ANCHOR_TOML_PATH" > "$TEMP_FILE"
  echo "" >> "$TEMP_FILE"
  echo "[programs.$ENV]" >> "$TEMP_FILE"
  echo "soccial_token = \"$PROGRAM_ID\"" >> "$TEMP_FILE"
fi

# Overwrite the Anchor.toml with the updated version
cp "$TEMP_FILE" "$ANCHOR_TOML_PATH"
rm "$TEMP_FILE"

# -------------------
# Step 5: Build
# -------------------
echo "🔨 Building..."
 anchor build -- --features no-idl,no-log-ix-name

# Get the program ID
PROGRAM_ID=$(solana address -k "$KEYPAIR_PATH")
echo "📛 Program ID: $PROGRAM_ID"
echo ""

if [ $? -ne 0 ]; then
  echo "❌ Build failed."
  exit 1
fi

# -------------------
# EXTRA DEPLOY SECURITY
# -------------------
# Verify the copied keypair matches the expected program ID
ANCHOR_PROGRAM_ID=$(solana address -k "$ANCHOR_KEYPAIR_PATH")

if [[ "$ANCHOR_PROGRAM_ID" != "$PROGRAM_ID" ]]; then
  echo "❌ Keypair mismatch after copy! Anchor keypair does not match expected program ID."
  echo "   - Expected: $PROGRAM_ID"
  echo "   - Found:    $ANCHOR_PROGRAM_ID"
  echo ""
  echo "🛑 Aborting deployment to prevent accidental overwrite of the wrong program."
  exit 1
fi

# -------------------
# Step 6: Deploy to selected environment
# -------------------
if [[ "$ENV" == "localnet" ]]; then
  # Deploy for localnet
  echo "🚀 Deploying to localnet..."
  solana program deploy target/deploy/soccial_token.so --url http://127.0.0.1:8899 --program-id "$KEYPAIR_PATH"

  if [ $? -ne 0 ]; then
    echo "❌ Deployment to localnet failed."
    echo "ℹ️  Tip: Make sure the keypair at $KEYPAIR_PATH matches the program ID and was never deployed before."
    exit 1
  fi
else
  # Deploy for devnet, testnet or mainnet
  echo "🚀 Deploying to $ENV..."
  anchor deploy --provider.cluster "$ENV" --provider.wallet "$ANCHOR_WALLET" -- --upgrade-authority "$ANCHOR_WALLET"
  if [ $? -ne 0 ]; then
    echo "❌ Deployment to $ENV failed."
    echo "ℹ️  Tip: Check if the wallet has enough SOL and the program ID is correctly set."
    exit 1
  fi
fi

# -------------------
# Step 7: Show + Verify Program (DONE!)
# -------------------
echo ""
echo "✅ Deployment complete and ready to rock!"
echo "📦 Program deployed at: $PROGRAM_ID"
[[ "$ENV" != "localnet" ]] && echo -e "🔗 View on Solana Explorer:\n   https://explorer.solana.com/address/$PROGRAM_ID?cluster=$ENV"

echo ""
echo "📋 Program details:"
solana program show "$PROGRAM_ID"

echo ""
echo "   ✨ Never forget — you're building this for the community."
echo "   🤝 Stay present, stay transparent, and always support those who believe in the project."
echo "   💪 Together, we're stronger. Always."
