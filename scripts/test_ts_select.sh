#!/bin/bash
# ========================================================================
# Soccial Token – Interactive TS Test Selector
#
# Usage:
#   yarn test:select-ts [filter_prefix] [--log level]
#
# Description:
#   This script allows you to interactively select and run TypeScript
#   integration tests for the Soccial Token program.
#
#   It scans all files in `tests_ts/` that start with `test_`, optionally
#   filtered by a prefix (e.g., `mint`, `liquidity`, etc.).
#
#   After listing the matching tests with index numbers, you can select one
#   by entering its corresponding number. The script will then run it using
#   `yarn test`, and display the full command used for easy copy/paste.
#
# Arguments:
#   [filter_prefix]      (optional) Limits the selection to test files that
#                        start with this prefix. If the prefix doesn't begin
#                        with `test_`, it will be added automatically.
#
#   --log [level]        (optional) Enables logging via environment variables.
#
# Examples:
#   yarn test:select-ts
#       → Lists and runs any test file manually selected
#
#   yarn test:select-ts mint
#       → Lists all files starting with test_mint and prompts to run one
#
#   yarn test:select-ts liquidity --log debug
#       → Lists liquidity-related tests and runs the selected one with logging
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ========================================================================

set -e

TEST_DIR="tests_ts"
EXTRA_ARGS=()
LOG_LEVEL=""
FILTER_PREFIX=""

# -----------------------------
# Parse arguments
# -----------------------------
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

# Detect optional test name prefix filter
if [[ ${#POSITIONAL_ARGS[@]} -ge 1 ]]; then
  FILTER_PREFIX="${POSITIONAL_ARGS[0]}"
  # Prepend test_ if missing
  [[ "$FILTER_PREFIX" != test_* ]] && FILTER_PREFIX="test_$FILTER_PREFIX"
fi

# -----------------------------
# Locate test files
# -----------------------------
if [[ -n "$FILTER_PREFIX" ]]; then
  FILES=($(find "$TEST_DIR" -type f -name "${FILTER_PREFIX}*.ts" | sort))
else
  FILES=($(find "$TEST_DIR" -type f -name 'test_*.ts' | sort))
fi

if [ ${#FILES[@]} -eq 0 ]; then
  echo "❌ No test files found in $TEST_DIR matching '$FILTER_PREFIX'"
  exit 1
fi

# -----------------------------
# List available test files
# -----------------------------
echo "🧪 Available tests:"
for i in "${!FILES[@]}"; do
  file_name=$(basename "${FILES[$i]}")
  display_name=${file_name#test_}
  display_name=${display_name%.ts}
  echo "  [$i] $display_name"
done

echo -n "👉 Enter test number to run: "
read -r TEST_INDEX

# Validate index
if ! [[ "$TEST_INDEX" =~ ^[0-9]+$ ]] || [ "$TEST_INDEX" -ge "${#FILES[@]}" ]; then
  echo "❌ Invalid selection."
  exit 1
fi

# -----------------------------
# Run the selected test
# -----------------------------
SELECTED_FILE="${FILES[$TEST_INDEX]}"
BASENAME=$(basename "$SELECTED_FILE")

echo "🚀 Running test: $BASENAME"
echo
echo "📋 Command:"
if [[ -n "$LOG_LEVEL" ]]; then
  echo "LOG_LEVEL=$LOG_LEVEL yarn test $BASENAME
else
  echo "yarn testts $BASENAME
fi
echo

if [[ -n "$LOG_LEVEL" ]]; then
  LOG_LEVEL="$LOG_LEVEL" yarn testts $BASENAME
else
  yarn testts $BASENAME
fi
