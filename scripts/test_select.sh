#!/bin/bash
# ========================================================================
# Soccial Token – Interactive Test Selector (with function explorer)
#
# Usage:
#   yarn test:select [filter_prefix] [--log level]
#
# Description:
#   This script allows you to interactively select and run Rust integration
#   tests for the Soccial Token program.
#
#   It scans all files in `programs/soccial_token/tests/` that start with
#   `test_`, optionally filtered by a prefix (e.g., `vesting_`).
#
#   After selecting a file, it runs the test using `cargo test-sbf`,
#   and then lists all test functions inside that file (e.g., #[test], #[tokio::test]).
#   It also displays the full `yarn test` command for copy/paste convenience.
#
# Arguments:
#   [filter_prefix]      (optional) Limits the selection to test files that
#                        start with this prefix. If the prefix doesn't begin
#                        with `test_`, it will be added automatically.
#
#   --log [level]        (optional) Enables logging via the RUST_LOG variable.
#                        Useful for debugging specific tests or modules.
#
# Examples:
#   yarn test:select
#       → Lists and runs any test file manually selected
#
#   yarn test:select vesting_
#       → Lists all files starting with test_vesting_ and prompts to run one
#
#   yarn test:select vote_ --log debug
#       → Lists vote-related tests and enables logging for the selected run
#
# Requirements:
#   - `cargo test-sbf` must be installed
#   - Test files must be named `test_*.rs`
#   - Tests must reside in `programs/soccial_token/tests/`
#   - Yarn must be installed and script mapped to `test:select` in package.json
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ========================================================================


set -e

TEST_DIR="programs/soccial_token/tests"
RUST_LOG_SET=false
FILTER_PREFIX=""

# -----------------------------
# Parse arguments
# -----------------------------
POSITIONAL_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --log|-l)
      export RUST_LOG="$2"
      RUST_LOG_SET=true
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
  [[ "$FILTER_PREFIX" != test_* ]] && FILTER_PREFIX="test_$FILTER_PREFIX"
fi

# -----------------------------
# Locate test files
# -----------------------------
if [[ -n "$FILTER_PREFIX" ]]; then
  FILES=($(find "$TEST_DIR" -type f -name "${FILTER_PREFIX}*.rs" | sort))
else
  FILES=($(find "$TEST_DIR" -type f -name 'test_*.rs' | sort))
fi

if [ ${#FILES[@]} -eq 0 ]; then
  echo "❌ No test files found in $TEST_DIR matching '$FILTER_PREFIX'"
  exit 1
fi

# -----------------------------
# List available test files
# -----------------------------
echo "🧪 Available test files:"
for i in "${!FILES[@]}"; do
  file_name=$(basename "${FILES[$i]}")
  display_name=${file_name#test_}
  display_name=${display_name%.rs}
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
BASENAME=$(basename "$SELECTED_FILE" .rs)

echo
echo "🚀 Running test file: $BASENAME.rs"
echo

cargo test-sbf --features dev --test "$BASENAME" -- --nocapture

# Show equivalent command
echo
echo -n "📋 Command: "
if $RUST_LOG_SET; then
  echo "RUST_LOG=$RUST_LOG yarn test $BASENAME"
else
  echo "yarn test $BASENAME"
fi

# -----------------------------
# List test functions in file
# -----------------------------
echo
echo "🔍 Functions available in $BASENAME.rs:"

FUNCTION_NAMES=()
FLAG_NEXT=false

while IFS= read -r line || [[ -n "$line" ]]; do
  if [[ "$line" =~ ^[[:space:]]*#\[.*test.*\] ]]; then
    FLAG_NEXT=true
    continue
  fi

  if [[ "$FLAG_NEXT" = true ]]; then
    if [[ "$line" =~ fn[[:space:]]+([a-zA-Z0-9_]+) || "$line" =~ async[[:space:]]+fn[[:space:]]+([a-zA-Z0-9_]+) || "$line" =~ pub[[:space:]]+(async[[:space:]]+)?fn[[:space:]]+([a-zA-Z0-9_]+) ]]; then
      func_name=$(echo "$line" | sed -E 's/.*fn[[:space:]]+([a-zA-Z0-9_]+).*/\1/')
      FUNCTION_NAMES+=("$func_name")
    fi
    FLAG_NEXT=false
  fi
done < "$SELECTED_FILE"

if [ ${#FUNCTION_NAMES[@]} -eq 0 ]; then
  echo "  (no #[test] functions found)"
else
  for name in "${FUNCTION_NAMES[@]}"; do
    echo "  → $name"
  done
  echo
  echo "📋 To run a specific function:"
  echo "   yarn test $BASENAME <function_name>"
fi
