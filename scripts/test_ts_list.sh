#!/bin/bash

# ==============================================================================
# Soccial Token – TypeScript Test Lister
#
# Usage:
#   ./scripts/test_ts_list.sh
#
# Description:
#   Lists all available TypeScript integration test files inside the `tests_ts/` directory.
#   This helps developers quickly discover which test modules are available for execution.
#
# Behavior:
#   - Scans the `tests_ts/` folder recursively for `.ts` files.
#   - Extracts the filename (without extension) and displays it in a readable format.
#
# Notes:
#   - To run a specific test, use the companion script: `test_ts.sh`
#   - Alternatively, use the Yarn shortcut: `yarn testts`
#
# Examples:
#   ./scripts/test_ts_list.sh
#       → Lists all available TypeScript test files
#
#   yarn testts devnet test_vars
#       → Runs `tests_ts/test_vars.ts` in devnet environment
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

echo "🔍 Listing available TypeScript tests:"
echo

find tests_ts -name "*.ts" -type f | while read -r file; do
  name=$(basename "$file" .ts)
  echo "🧪 $name"
done

echo
echo "📦 To run a specific test:"
echo "👉 ./scripts/test_ts.sh <environment> <testName>"
echo "   (e.g., ./scripts/test_ts.sh devnet test_vars)"
echo
echo "📦 Or using Yarn:"
echo "👉 yarn testts devnet test_vars"
