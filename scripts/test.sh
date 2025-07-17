#!/bin/bash
# ========================================================================
# Soccial Token – Test Runner Script
#
# Usage:
#   yarn test [test_name] [function_name] [--log level]
#
# Description:
#   This script runs Rust integration tests for the Soccial Token program
#   using `cargo test-sbf`, with optional structured logging.
#
# Arguments:
#   [test_name]         (optional) Name of the test file (without `.rs`).
#                       You can write just `add_admin` instead of `test_add_admin`.
#                       The script will automatically prepend `test_` if missing.
#
#   [function_name]     (optional) Specific test function name inside the file.
#                       When provided, only that test function will be executed.
#                       Useful for targeting individual `#[tokio::test]` cases.
#
#   --log [level]       (optional) Sets the RUST_LOG level.
#                       You can specify a general log level (e.g. debug, info, warn, error)
#                       or a module-specific target (e.g. soccial_token=debug).
#
# Examples:
#   yarn test
#       → Runs all tests without logs
#
#   yarn test add_admin
#       → Automatically expands to test_add_admin.rs and runs it
#
#   yarn test init_vesting twice_should_fail
#       → Runs `test_init_vesting_twice_should_fail()` from `test_init_vesting.rs`
#
#   yarn test test_debug_log --log info
#       → Runs a specific test file with RUST_LOG=info
#
#   yarn test vote governance_fails_without_tokens --log soccial_token=debug
#       → Runs a specific test function with module-level logs
#
# Requirements:
#   - `cargo test-sbf` must be installed
#   - Test files must be located in `programs/soccial_token/tests/`
#   - Yarn installed
#
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ========================================================================

# Exit immediately if any command fails
set -e

# Initialize variables
TEST_NAME=""
FUNCTION_NAME=""
LOG_LEVEL=""

# Parse arguments
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --log|-l)
      LOG_LEVEL="$2"
      shift 2
      ;;
    *)
      POSITIONAL_ARGS+=("$1")
      shift
      ;;
  esac
done

# Restore positional arguments
set -- "${POSITIONAL_ARGS[@]}"

# Extract test and function names
if [[ $# -ge 1 ]]; then
  TEST_NAME="$1"
fi

if [[ $# -ge 2 ]]; then
  FUNCTION_NAME="$2"
fi

# Set RUST_LOG if provided
if [[ -n "$LOG_LEVEL" ]]; then
  export RUST_LOG="$LOG_LEVEL"
  echo "🪵 RUST_LOG=$RUST_LOG enabled"
fi

# === Run all tests if no specific test file is provided ===
if [[ -z "$TEST_NAME" ]]; then
  echo "🧪 Running all tests (no specific test file provided)"
  cargo test-sbf --features dev -- --nocapture
  exit 0
fi

# Remove ".rs" extension if present
TEST_NAME=${TEST_NAME%.rs}

# Add "test_" prefix if missing
if [[ "$TEST_NAME" != test_* ]]; then
  TEST_NAME="test_$TEST_NAME"
fi

# Run specific test or file
echo "🧪 Running Rust test file: $TEST_NAME.rs"

if [[ -n "$FUNCTION_NAME" ]]; then
  cargo test-sbf --features dev --test "$TEST_NAME" "$FUNCTION_NAME" -- --nocapture
else
  cargo test-sbf --features dev --test "$TEST_NAME" -- --nocapture
fi