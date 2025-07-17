#!/bin/bash

# ==============================================================================
# Soccial Token – Rust Test Lister
#
# Usage:
#   ./scripts/test_list.sh
#
# Description:
#   Lists all available Rust integration test files inside the
#   `programs/soccial_token/tests/` directory.
#
# Behavior:
#   - Scans the test directory for all `.rs` files.
#   - Extracts and displays each test filename without its extension.
#
# Notes:
#   - These tests are written using `#[tokio::test]` in Rust.
#   - Useful for discovering which test modules are available to run.
#
# Examples:
#   ./scripts/test_list.sh
#       → Outputs: test_add_admin, test_vesting_flow, etc.
#
#   ./scripts/test.sh test_add_admin
#       → Runs the corresponding test file via `cargo test-sbf`
#
#   yarn test test_add_admin
#       → Equivalent via Yarn shortcut
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

TEST_DIR="programs/soccial_token/tests"

echo "🔍 Listing available Rust integration tests in $TEST_DIR:"
echo

find "$TEST_DIR" -name "*.rs" -type f | while read -r file; do
  name=$(basename "$file" .rs)
  echo "🧪 $name"
done

echo
echo "📦 To run a specific test:"
echo "👉 ./scripts/test.sh <testName>"
echo "   (e.g., ./scripts/test.sh test_add_admin)"
echo
echo "📦 Or using Yarn:"
echo "👉 yarn test test_add_admin"
