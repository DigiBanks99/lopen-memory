#!/bin/bash

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name')
TOOL_INPUT=$(echo "$INPUT" | jq -r '.tool_input')

echo "Received tool_name: $TOOL_NAME"
if [[ "$TOOL_NAME" != "validate-integrity" ]]; then
    echo "Error: This script is only for the 'validate-integrity' tool."
    exit 1
fi

echo "Running integrity checks..."
echo "Formatting code..."
cargo fmt --all
echo "Linting code..."
cargo clippy --workspace --all-targets -- -D warnings --allow-dirty
echo "Running tests..."
cargo test --all --all-features
echo "Running cargo audit..."
cargo audit --deny warnings
echo "Project integrity verified successfully"