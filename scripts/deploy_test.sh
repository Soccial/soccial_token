#!/bin/bash

# check if everything is ok
# Usage: ./scripts/deploy_test.sh localnet|devnet|testnet|mainnet

# Go to the project root (one level up from /scripts)
cd "$(dirname "$0")/.." || exit

ENV=$1
ENV_FILE=".env/${ENV}.env"
KEYS_DIR="./.deploy-keys"
KEYPAIR_PATH="$KEYS_DIR/soccial_token-${ENV}-keypair.json"
LIB_RS_PATH="programs/soccial_token/src/lib.rs"
ANCHOR_TOML_PATH="Anchor.toml"

# Ensure keys dir exists
mkdir -p "$KEYS_DIR"

if [[ -z "$ENV" ]]; then
  echo "❌ Missing environment. Usage: ./scripts/deploy_test.sh localnet|devnet|testnet|mainnet"
  exit 1
fi

if [[ "$ENV" != "localnet" && "$ENV" != "devnet" && "$ENV" != "testnet" && "$ENV" != "mainnet" ]]; then
  echo "❌ Invalid environment: $ENV (must be localnet, devnet, testnet or mainnet)"
  exit 1
fi

echo "🔍 Checking environment: $ENV"
echo ""

# -------------------
# Step 1: Check required files
# -------------------

missing=0

# ENV
if [[ "$ENV" != "localnet" ]]; then
  if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Missing environment file: $ENV_FILE"
    missing=1
  else
    echo "✅ Found env file: $ENV_FILE"
    source "$ENV_FILE"
  fi
else
  echo "ℹ️  Skipping env file for localnet"
fi

# lib.rs
if [ ! -f "$LIB_RS_PATH" ]; then
  echo "❌ Missing lib.rs: $LIB_RS_PATH"
  missing=1
else
  echo "✅ Found lib.rs"
fi

# Anchor.toml
if [ ! -f "$ANCHOR_TOML_PATH" ]; then
  echo "❌ Missing Anchor.toml: $ANCHOR_TOML_PATH"
  missing=1
else
  echo "✅ Found Anchor.toml"
fi

# Keypair
if [ ! -f "$KEYPAIR_PATH" ]; then
  echo "⚠️  Keypair not found yet: $KEYPAIR_PATH (will be generated during deploy)"
else
  echo "✅ Found keypair: $KEYPAIR_PATH"
fi


# Wallet (only required for remote environments)
if [[ "$ENV" != "localnet" ]]; then
  if [[ -z "$ANCHOR_WALLET" ]]; then
    echo "❌ ANCHOR_WALLET not defined in $ENV_FILE"
    missing=1
  elif [ ! -f "$ANCHOR_WALLET" ]; then
    echo "❌ Wallet file not found: $ANCHOR_WALLET"
    missing=1
  else
    echo "✅ Wallet file exists: $ANCHOR_WALLET"

    # Show address
    WALLET_ADDRESS=$(solana address -k "$ANCHOR_WALLET")
    echo "🏦 Wallet address: $WALLET_ADDRESS"

    # Show balance
    if [[ -n "$ANCHOR_PROVIDER_URL" ]]; then
      WALLET_BALANCE=$(solana balance "$ANCHOR_WALLET" --url "$ANCHOR_PROVIDER_URL")
    else
      WALLET_BALANCE=$(solana balance "$ANCHOR_WALLET")
    fi
    echo "💰 Wallet balance: $WALLET_BALANCE"
  fi
else
  echo "ℹ️  Skipping wallet check for localnet"
fi

# -------------------
# Step 2: Summary
# -------------------

if [ "$missing" -eq 1 ]; then
  echo ""
  echo "❌ Some required files are missing or invalid. Please fix the issues above."
  exit 1
fi

echo ""
echo "✅ All checks passed! You're ready to deploy."
echo ""
echo "👉 Run the actual deployment with:"
echo ""
echo "  bash ./scripts/deploy.sh $ENV"
echo ""
