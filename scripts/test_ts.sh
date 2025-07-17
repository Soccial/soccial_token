#!/bin/bash
# ==============================================================================
# Soccial Token – TypeScript Test Runner Script
#
# Usage:
#   ./scripts/test_ts.sh [test_name] [environment]
#
# Description:
#   This script runs TypeScript integration tests for the Soccial Token program
#   using `tsx`, allowing you to run either all tests or a specific test file.
#   Optionally, you can specify the Solana cluster environment (localnet, devnet, testnet, mainnet).
#
# Arguments:
#   [test_name]         (optional) Name of the test file (without `.ts` extension).
#                       Example: `init_economy` will match `tests_ts/init_economy.ts`.
#
#   [environment]       (optional) Solana cluster environment to target.
#                       Options: `localnet`, `devnet`, `testnet`, `mainnet`.
#                       Defaults to `localnet` if not provided.
#
# Examples:
#   ./scripts/test_ts.sh
#       → Runs all TypeScript tests on localnet.
#
#   ./scripts/test_ts.sh init_spl_token
#       → Runs only `tests_ts/init_spl_token.ts` on localnet.
#
#   ./scripts/test_ts.sh devnet
#       → Runs all TypeScript tests on devnet.
#
#   ./scripts/test_ts.sh add_admin testnet
#       → Runs only `tests_ts/add_admin.ts` on testnet.
#
# Requirements:
#   - Node.js and Yarn must be installed.
#   - `tsx` must be available (installed via Yarn).
#   - Test files must be located in the `tests_ts/` directory.
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

set -e

FIRST_ARG=$1
SECOND_ARG=$2

# Default environment
ENV="localnet"
TEST_NAME=""

# If only one argument is provided, determine if it is an environment or a test name
if [[ -n "$FIRST_ARG" && -z "$SECOND_ARG" ]]; then
  if [[ "$FIRST_ARG" == "localnet" || "$FIRST_ARG" == "devnet" || "$FIRST_ARG" == "testnet" || "$FIRST_ARG" == "mainnet" ]]; then
    ENV="$FIRST_ARG"
  else
    TEST_NAME="$FIRST_ARG"
  fi
elif [[ -n "$FIRST_ARG" && -n "$SECOND_ARG" ]]; then
  TEST_NAME="$FIRST_ARG"
  ENV="$SECOND_ARG"
fi

# Export environment variable to be accessible inside tests
export TEST_ENV="$ENV"

# Add "test_" prefix if missing
if [[ "$TEST_NAME" != test_* ]]; then
  TEST_NAME="test_$TEST_NAME"
fi

# Run a specific test if test name is defined
if [[ -n "$TEST_NAME" ]]; then
  TEST_NAME=${TEST_NAME%.ts}
  TEST_PATH="tests_ts/${TEST_NAME}.ts"

  if [[ ! -f "$TEST_PATH" ]]; then
    echo "❌ Test file not found: $TEST_PATH"
    exit 1
  fi

  echo "🧪 Running test file: $TEST_PATH for environment: $TEST_ENV"
  yarn tsx --test "$TEST_PATH"
else
  echo "🧪 Running all tests for environment: $TEST_ENV"
  yarn tsx --test tests_ts/**/*.ts
fi
