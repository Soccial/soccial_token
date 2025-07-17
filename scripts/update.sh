#!/bin/bash

# ==============================================================================
# Soccial Token – Program Upgrade Script
#
# Usage:
#   ./scripts/update.sh devnet|testnet|mainnet
#   
#
# Description:
#   This script upgrades the deployed Soccial Token smart contract on a
#   specified Solana cluster using Anchor's `upgrade` command.
#
# Behavior:
#   - Loads environment configuration from the appropriate `.env` file.
#   - Validates wallet and program keypair existence.
#   - Builds the latest version of the program.
#   - Performs a program upgrade to the specified cluster.
#
# Arguments:
#   devnet|testnet|mainnet  → Target Solana environment for the upgrade.
#
# Requirements:
#   - Anchor CLI installed
#   - `solana` CLI installed and configured
#   - Environment `.env` files located in `.env/` (e.g., `.env/devnet.env`)
#   - Program keypair in `target/deploy/`
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

# Go to the project root (one level up from /scripts)
cd "$(dirname "$0")/.." || exit

ENV=$1
ENV_FILE=".env/${ENV}.env"
KEYPAIR_PATH="target/deploy/soccial_token-${ENV}-keypair.json"
PROGRAM_SO_PATH="target/deploy/soccial_token.so"

# -------------------
# Step 0: Validate input
# -------------------
if [[ -z "$ENV" ]]; then
  echo "❌ Missing environment. Usage: ./scripts/update.sh devnet|testnet|mainnet"
  exit 1
fi

echo "🌍 Selected environment: $ENV"
echo "📄 Loading environment from: $ENV_FILE"

if [ ! -f "$ENV_FILE" ]; then
  echo "❌ Env file not found: $ENV_FILE"
  exit 1
fi

# -------------------
# Step 1: Load env vars
# -------------------
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

# -------------------
# Step 2: Verify keypair exists
# -------------------
if [ ! -f "$KEYPAIR_PATH" ]; then
  echo "❌ Program keypair not found: $KEYPAIR_PATH"
  exit 1
fi

PROGRAM_ID=$(solana address -k "$KEYPAIR_PATH")
echo "📛 Program ID: $PROGRAM_ID"
echo ""

# -------------------
# Step 3: Build
# -------------------
echo "🔨 Building updated program..."
anchor build
if [ $? -ne 0 ]; then
  echo "❌ Build failed."
  exit 1
fi

# -------------------
# Step 4: Upgrade
# -------------------
echo "⚙️  Upgrading Soccial Token ($ENV)..."
anchor upgrade --program-id "$PROGRAM_ID" "$PROGRAM_SO_PATH"
if [ $? -ne 0 ]; then
  echo "❌ Upgrade failed."
  exit 1
fi

# -------------------
# Done!
# -------------------
echo ""
echo "✅ Upgrade complete!"
echo "📦 Program updated at: $PROGRAM_ID"
echo "🔗 View on Solana Explorer:"
echo "   https://explorer.solana.com/address/$PROGRAM_ID?cluster=$ENV"
