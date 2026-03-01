#!/bin/bash

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name')
TOOL_INPUT=$(echo "$INPUT" | jq -r '.tool_input')

# Only run the pre-commit checks if the tool being used is "runTerminalCommand" or "run_in_terminal"
if [[ "$TOOL_NAME" != "runTerminalCommand" && "$TOOL_NAME" != "run_in_terminal" ]]; then
    echo '{"continue": 1}'
    exit 0
fi

# Only run if TOOL_INPUT starts with or contains "git commit <anything>"
if [[ "$TOOL_INPUT" != *"git commit"* ]]; then
    echo '{"continue": 1}'
    exit 0
fi


echo "Running pre-commit checks..."
echo "Formatting..."
cargo fmt --all
if [ $? -ne 0 ]; then
    echo '{"continue": 0, "stopReason": "Code formatting failed.", "systemMessage": "Operation blocked by pre-commit hook: code formatting issues detected. Please run `cargo fmt` to fix formatting."}'
    exit 1
fi

echo "Linting..."
cargo clippy --workspace --all-targets -- -D warnings
if [ $? -ne 0 ]; then
    echo '{"continue": 0, "stopReason": "Linting failed.", "systemMessage": "Operation blocked by pre-commit hook: linting issues detected. Please run `cargo clippy` to fix linting errors."}'
    exit 1
fi

echo "Running tests..."
cargo test --all --all-features
if [ $? -ne 0 ]; then
    echo '{"continue": 0, "stopReason": "Tests failed.", "systemMessage": "Operation blocked by pre-commit hook: test failures detected. Please run `cargo test` to fix test errors."}'
    exit 1
fi

echo "Running cargo audit..."
cargo audit --deny warnings
if [ $? -ne 0 ]; then
    echo '{"continue": 0, "stopReason": "Security audit failed.", "systemMessage": "Operation blocked by pre-commit hook: security vulnerabilities detected. Please run `cargo audit` to fix security issues."}'
    exit 1
fi

echo '{"continue": 1}'