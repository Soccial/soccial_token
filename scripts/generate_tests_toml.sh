#!/bin/bash

# ==============================================================================
# Soccial Token – Cargo.toml Test Injector
#
# Usage:
#   ./scripts/inject_tests.sh
#
# Description:
#   Automatically scans the integration test folder and injects
#   `[[test]]` entries into `Cargo.toml` under the `[dev-dependencies]` section.
#
#   It looks for the marker lines:
#     # BEGIN GENERATED TESTS
#     # END GENERATED TESTS
#   And replaces everything in between with fresh entries based on the files
#   found in `programs/soccial_token/tests/*.rs`.
#
# Notes:
#   - Makes a backup of the original `Cargo.toml` as `Cargo.toml.bak`
#   - Requires the markers to already exist in `Cargo.toml`
#
# Example marker block in Cargo.toml:
#
#     # BEGIN GENERATED TESTS
#     # END GENERATED TESTS
#
# Project: Soccial Token
# Author: Paulo Rodrigues
# License: MIT
# ==============================================================================

CARGO_TOML="programs/soccial_token/Cargo.toml"
TEST_DIR="programs/soccial_token/tests"
BEGIN_MARKER="# BEGIN GENERATED TESTS"
END_MARKER="# END GENERATED TESTS"

# Check if test directory exists
if [ ! -d "$TEST_DIR" ]; then
  echo "❌ Error: Test directory '$TEST_DIR' does not exist."
  exit 1
fi

# Check if both markers exist in Cargo.toml
if ! grep -q "$BEGIN_MARKER" "$CARGO_TOML" || ! grep -q "$END_MARKER" "$CARGO_TOML"; then
  echo "❌ Error: Both markers must exist in $CARGO_TOML:"
  echo "   $BEGIN_MARKER"
  echo "   $END_MARKER"
  exit 1
fi

# Backup original
cp "$CARGO_TOML" "$CARGO_TOML.bak"

# Create temporary output file
TMP_FILE=$(mktemp)
inside_block=false

while IFS= read -r line; do
  if [[ "$line" == "$BEGIN_MARKER" ]]; then
    echo "$line" >> "$TMP_FILE"
    inside_block=true

    # Inject generated entries
    for file in "$TEST_DIR"/*.rs; do
      filename=$(basename "$file")
      test_name="${filename%.rs}"

      cat <<EOF >> "$TMP_FILE"

[[test]]
name = "$test_name"
path = "tests/$filename"
required-features = ["dev"]
EOF
    done

    continue
  fi

  if [[ "$line" == "$END_MARKER" ]]; then
    echo "$line" >> "$TMP_FILE"
    inside_block=false
    continue
  fi

  # Write only lines that are outside the generated block
  if ! $inside_block; then
    echo "$line" >> "$TMP_FILE"
  fi
done < "$CARGO_TOML"

# Replace original file
mv "$TMP_FILE" "$CARGO_TOML"

echo "✅ Successfully updated test entries in $CARGO_TOML"
