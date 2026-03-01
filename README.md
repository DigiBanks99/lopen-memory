# lopen-memory

A CLI utility for tracking software projects, modules, features, tasks, and research — backed by a local SQLite database. Designed for use by LLM agents.

## Install

From a release:

```bash
mkdir -p ~/.local/share/lopen-memory ~/.local/bin

curl -L https://github.com/DigiBanks99/lopen-memory/releases/latest/download/lopen-memory-x86_64-unknown-linux-gnu \
  -o ~/.local/share/lopen-memory/lopen-memory

chmod +x ~/.local/share/lopen-memory/lopen-memory

ln -sf ~/.local/share/lopen-memory/lopen-memory ~/.local/bin/lopen-memory
```

Or from source:

```bash
cargo install --path .
```

Or copy `target/release/lopen-memory` to somewhere on your `$PATH`.

## Build

Requires Rust (stable). The SQLite library is bundled statically — no system dependencies needed.

```bash
cargo build --release
# Binary at: target/release/lopen-memory
```

## Database

Default location: `~/.lopen-memory/lopen-memory.db`

Override via environment variable or flag:

```bash
LOPEN_MEMORY_DB=/tmp/test.db lopen-memory project list
lopen-memory --db /tmp/test.db project list
```

## Quick Start

```bash
# Projects
lopen-memory project add my-app /home/user/my-app "Core application"
lopen-memory project list
lopen-memory project show --project my-app

# Modules
lopen-memory module add --project my-app auth "Authentication system"
lopen-memory module transition --module auth --project my-app Planning

# Features
lopen-memory feature add --module auth login-flow "User login and session creation"

# Tasks
lopen-memory task add --feature login-flow implement-jwt "Implement JWT issuance"

# Research
lopen-memory research add jwt-rfc "The IETF JSON Web Token specification"
lopen-memory research set-source --research jwt-rfc "https://datatracker.ietf.org/doc/html/rfc7519"
lopen-memory research link --research jwt-rfc --module auth
lopen-memory research search jwt
```

## Output

Plain text by default. Add `--json` for JSON output on any command.

## Hierarchy

```bash
Project → Module → Feature → Task
Research (root-level, linked to any entity via bridge tables)
```

## Testing pre-commit hook

The pre-commit script at `scripts/pre-commit.sh` expects the same JSON payload that the agent passes to `runTerminalCommand`. To force the hook to run locally, pipe the payload into the script while mimicking the `git commit` command, for example:

```
printf '{"tool_name":"runTerminalCommand","tool_input":"git commit -m \"test\""}' | ./scripts/pre-commit.sh
```
