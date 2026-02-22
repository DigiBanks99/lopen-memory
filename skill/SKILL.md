---
name: lopen-memory
description: "Use this skill whenever you need to manage projects, modules, features, tasks, or research using the lopen-memory CLI. Triggers include: any request to create, list, show, update, or remove a project, module, feature, or task; any request to add, search, link, or manage research; any request to track work state or transition a module/feature/task through its lifecycle (Draft, Planning, Building, Complete, Amending). Use this skill before running any lopen-memory command to ensure correct syntax and avoid mistakes."
---

# lopen-memory Skill

## Overview

`lopen-memory` is a CLI tool that tracks software projects and their breakdown across four levels: **Project → Module → Feature → Task**. Research is a root-level entity that can be linked to any of those levels via bridge tables.

All input is plain text positional arguments or named flags. No JSON input. Output is plain text by default; add `--json` for machine-readable output.

**Binary location:** `~/.local/bin/lopen-memory`
**Database default:** `~/.lopen-memory/lopen-memory.db`
**Override database:** `LOPEN_MEMORY_DB=/path/to/db lopen-memory ...` or `--db /path/to/db`

---

## Quick Reference

| Resource | Key Verbs |
|---|---|
| `project` | `add`, `list`, `show`, `rename`, `set-description`, `set-path`, `complete`, `reopen`, `remove` |
| `module` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `feature` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `task` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `research` | `add`, `list`, `show`, `rename`, `set-description`, `set-content`, `set-source`, `set-researched-at`, `search`, `link`, `unlink`, `links`, `remove` |

---

## State Machine

Modules, features, and tasks all share this lifecycle. Transitions are validated and invalid ones are rejected.

```
Draft → Planning → Building → Complete → Amending → Draft
```

Any state may also reset directly to `Draft`.

| From | Allowed To |
|---|---|
| Draft | Planning, Draft |
| Planning | Building, Draft |
| Building | Complete, Draft |
| Complete | Amending, Draft |
| Amending | Draft |

Transitioning to the current state is a silent no-op.

---

## Project Commands

```bash
# Add
lopen-memory project add <name> <path> [description]
lopen-memory project add my-app /home/user/my-app "Core application rewrite"

# List (optional filters, mutually exclusive)
lopen-memory project list
lopen-memory project list --completed
lopen-memory project list --incomplete

# Show (includes modules and linked research)
lopen-memory project show --project <name|id>

# Update
lopen-memory project rename --project <name|id> <new-name>
lopen-memory project set-description --project <name|id> "<description>"
lopen-memory project set-path --project <name|id> <new-path>

# Lifecycle
lopen-memory project complete --project <name|id>
lopen-memory project reopen --project <name|id>

# Remove (--cascade removes all child modules, features, tasks)
lopen-memory project remove --project <name|id>
lopen-memory project remove --project <name|id> --cascade
```

---

## Module Commands

```bash
# Add (initial state is always Draft)
lopen-memory module add --project <name|id> <name> [description]

# List (optional state filter)
lopen-memory module list --project <name|id>
lopen-memory module list --project <name|id> --state Draft

# Show (includes features and linked research)
lopen-memory module show --module <name|id>
lopen-memory module show --module <name|id> --project <name|id>

# Update
lopen-memory module rename --module <name|id> <new-name>
lopen-memory module set-description --module <name|id> "<description>"
lopen-memory module set-details --module <name|id> "<details>"

# Transition
lopen-memory module transition --module <name|id> <state>
lopen-memory module transition --module auth --project my-app Planning

# Remove
lopen-memory module remove --module <name|id>
lopen-memory module remove --module <name|id> --cascade
```

---

## Feature Commands

```bash
# Add
lopen-memory feature add --module <name|id> <name> [description]
lopen-memory feature add --module auth --project my-app login-flow "User login"

# List
lopen-memory feature list --module <name|id>
lopen-memory feature list --module <name|id> --state Building

# Show (includes tasks and linked research)
lopen-memory feature show --feature <name|id>
lopen-memory feature show --feature <name|id> --module <name|id>

# Update
lopen-memory feature rename --feature <name|id> <new-name>
lopen-memory feature set-description --feature <name|id> "<description>"
lopen-memory feature set-details --feature <name|id> "<details>"

# Transition
lopen-memory feature transition --feature <name|id> <state>

# Remove
lopen-memory feature remove --feature <name|id>
lopen-memory feature remove --feature <name|id> --cascade
```

---

## Task Commands

```bash
# Add
lopen-memory task add --feature <name|id> <name> [description]

# List
lopen-memory task list --feature <name|id>
lopen-memory task list --feature <name|id> --state Draft

# Show (includes linked research)
lopen-memory task show --task <name|id>
lopen-memory task show --task <name|id> --feature <name|id>

# Update
lopen-memory task rename --task <name|id> <new-name>
lopen-memory task set-description --task <name|id> "<description>"
lopen-memory task set-details --task <name|id> "<details>"

# Transition
lopen-memory task transition --task <name|id> <state>

# Remove (no --cascade needed; tasks have no children)
lopen-memory task remove --task <name|id>
```

---

## Research Commands

```bash
# Add
lopen-memory research add <name> [description]
lopen-memory research add jwt-rfc "IETF JSON Web Token specification"

# List (optional staleness filter)
lopen-memory research list
lopen-memory research list --stale-days 180

# Show (includes full content and all links)
lopen-memory research show --research <name|id>

# Search (case-insensitive across name, description, content, source)
lopen-memory research search <term>
lopen-memory research search jwt
lopen-memory research search authentication --stale-days 180

# Update
lopen-memory research rename --research <name|id> <new-name>
lopen-memory research set-description --research <name|id> "<description>"
lopen-memory research set-content --research <name|id> "<content>"
lopen-memory research set-content --research <name|id> "<content>" --no-update-date
lopen-memory research set-source --research <name|id> "<source>"
lopen-memory research set-researched-at --research <name|id> <YYYY-MM-DD>

# Link to work entities (exactly one target per call)
lopen-memory research link --research <name|id> --project <name|id>
lopen-memory research link --research <name|id> --module <name|id>
lopen-memory research link --research <name|id> --feature <name|id>
lopen-memory research link --research <name|id> --task <name|id>

# Unlink
lopen-memory research unlink --research <name|id> --project <name|id>
lopen-memory research unlink --research <name|id> --module <name|id>
lopen-memory research unlink --research <name|id> --feature <name|id>
lopen-memory research unlink --research <name|id> --task <name|id>

# List all links for a research record
lopen-memory research links --research <name|id>

# Remove (does NOT delete linked work entities)
lopen-memory research remove --research <name|id>
```

---

## Identifier Resolution

All `--project`, `--module`, `--feature`, `--task`, and `--research` flags accept either a **name** or a numeric **ID**.

- Integer IDs are globally unique per table and never need a parent flag.
- Names are resolved within the scope of the supplied parent flag.
- If a name is ambiguous (exists under multiple parents), the command fails with an error asking you to narrow scope with a parent flag.
- Research names are globally unique and never need a parent flag.

```bash
# These are equivalent if ID 3 is the auth module
lopen-memory module show --module auth --project my-app
lopen-memory module show --module 3
```

---

## Output Behaviour

- **Plain text** by default — one record per line for lists, labelled fields for show views.
- **JSON** via `--json` flag on any command.
- **Errors** always go to stderr. stdout is always clean.
- **Exit codes:** `0` = success, `1` = input/user error, `2` = internal/db error.
- **Nothing found** always prints an explicit message — never silence.

---

## Key Rules for Correct Usage

- `description` is the stable goal statement. Use `set-description` to update it.
- `details` is evolving working notes. Use `set-details` to replace it entirely.
- `set-content` on research also updates `researched_at` to now unless `--no-update-date` is passed.
- `research link` requires exactly one of `--project`, `--module`, `--feature`, or `--task`.
- Linking the same pair twice is a no-op. Unlinking a non-existent pair is a no-op.
- Removing a project/module/feature/task does NOT delete linked research — only the bridge rows are removed.
- Removing research does NOT affect any linked work entities.
- `--cascade` on remove deletes all descendants. Without it, remove fails if children exist.
---
name: lopen-memory
description: "Use this skill whenever you need to manage projects, modules, features, tasks, or research using the lopen-memory CLI. Triggers include: any request to create, list, show, update, or remove a project, module, feature, or task; any request to add, search, link, or manage research; any request to track work state or transition a module/feature/task through its lifecycle (Draft, Planning, Building, Complete, Amending). Use this skill before running any lopen-memory command to ensure correct syntax and avoid mistakes."
---

# lopen-memory Skill

## Overview

`lopen-memory` is a CLI tool that tracks software projects and their breakdown across four levels: **Project → Module → Feature → Task**. Research is a root-level entity that can be linked to any of those levels via bridge tables.

All input is plain text positional arguments or named flags. No JSON input. Output is plain text by default; add `--json` for machine-readable output.

**Binary location:** `~/.local/bin/lopen-memory`
**Database default:** `~/.lopen-memory/lopen-memory.db`
**Override database:** `LOPEN_MEMORY_DB=/path/to/db lopen-memory ...` or `--db /path/to/db`

---

## Quick Reference

| Resource | Key Verbs |
|---|---|
| `project` | `add`, `list`, `show`, `rename`, `set-description`, `set-path`, `complete`, `reopen`, `remove` |
| `module` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `feature` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `task` | `add`, `list`, `show`, `rename`, `set-description`, `set-details`, `transition`, `remove` |
| `research` | `add`, `list`, `show`, `rename`, `set-description`, `set-content`, `set-source`, `set-researched-at`, `search`, `link`, `unlink`, `links`, `remove` |

---

## State Machine

Modules, features, and tasks all share this lifecycle. Transitions are validated and invalid ones are rejected.

```
Draft → Planning → Building → Complete → Amending → Draft
```

Any state may also reset directly to `Draft`.

| From | Allowed To |
|---|---|
| Draft | Planning, Draft |
| Planning | Building, Draft |
| Building | Complete, Draft |
| Complete | Amending, Draft |
| Amending | Draft |

Transitioning to the current state is a silent no-op.

---

## Project Commands

```bash
lopen-memory project add <name> <path> [description]
lopen-memory project list [--completed] [--incomplete]
lopen-memory project show --project <name|id>
lopen-memory project rename --project <name|id> <new-name>
lopen-memory project set-description --project <name|id> "<description>"
lopen-memory project set-path --project <name|id> <new-path>
lopen-memory project complete --project <name|id>
lopen-memory project reopen --project <name|id>
lopen-memory project remove --project <name|id> [--cascade]
```

---

## Module Commands

```bash
lopen-memory module add --project <name|id> <name> [description]
lopen-memory module list --project <name|id> [--state <state>]
lopen-memory module show --module <name|id> [--project <name|id>]
lopen-memory module rename --module <name|id> [--project <name|id>] <new-name>
lopen-memory module set-description --module <name|id> [--project <name|id>] "<description>"
lopen-memory module set-details --module <name|id> [--project <name|id>] "<details>"
lopen-memory module transition --module <name|id> [--project <name|id>] <state>
lopen-memory module remove --module <name|id> [--project <name|id>] [--cascade]
```

---

## Feature Commands

```bash
lopen-memory feature add --module <name|id> [--project <name|id>] <name> [description]
lopen-memory feature list --module <name|id> [--project <name|id>] [--state <state>]
lopen-memory feature show --feature <name|id> [--module <name|id>]
lopen-memory feature rename --feature <name|id> [--module <name|id>] <new-name>
lopen-memory feature set-description --feature <name|id> [--module <name|id>] "<description>"
lopen-memory feature set-details --feature <name|id> [--module <name|id>] "<details>"
lopen-memory feature transition --feature <name|id> [--module <name|id>] <state>
lopen-memory feature remove --feature <name|id> [--module <name|id>] [--cascade]
```

---

## Task Commands

```bash
lopen-memory task add --feature <name|id> <name> [description]
lopen-memory task list --feature <name|id> [--state <state>]
lopen-memory task show --task <name|id> [--feature <name|id>]
lopen-memory task rename --task <name|id> [--feature <name|id>] <new-name>
lopen-memory task set-description --task <name|id> [--feature <name|id>] "<description>"
lopen-memory task set-details --task <name|id> [--feature <name|id>] "<details>"
lopen-memory task transition --task <name|id> [--feature <name|id>] <state>
lopen-memory task remove --task <name|id> [--feature <name|id>]
```

---

## Research Commands

```bash
lopen-memory research add <name> [description]
lopen-memory research list [--stale-days <n>]
lopen-memory research show --research <name|id>
lopen-memory research rename --research <name|id> <new-name>
lopen-memory research set-description --research <name|id> "<description>"
lopen-memory research set-content --research <name|id> "<content>" [--no-update-date]
lopen-memory research set-source --research <name|id> "<source>"
lopen-memory research set-researched-at --research <name|id> <YYYY-MM-DD>
lopen-memory research search <term> [--stale-days <n>]
lopen-memory research link --research <name|id> --project <name|id>
lopen-memory research link --research <name|id> --module <name|id>
lopen-memory research link --research <name|id> --feature <name|id>
lopen-memory research link --research <name|id> --task <name|id>
lopen-memory research unlink --research <name|id> --project <name|id>
lopen-memory research unlink --research <name|id> --module <name|id>
lopen-memory research unlink --research <name|id> --feature <name|id>
lopen-memory research unlink --research <name|id> --task <name|id>
lopen-memory research links --research <name|id>
lopen-memory research remove --research <name|id>
```

---

## Identifier Resolution

All flags accept either a name or a numeric ID. Integer IDs are globally unique per table. Names are resolved within the scope of the supplied parent flag. Research names are globally unique.

---

## Output Behaviour

- Plain text by default. `--json` for JSON output.
- Errors go to stderr. stdout is always clean.
- Exit codes: `0` success, `1` input error, `2` internal error.
- Nothing found always prints an explicit message, never silence.

---

## Key Rules

- `description` is the stable goal. `details` is evolving working notes.
- `set-content` also updates `researched_at` unless `--no-update-date` is passed.
- `research link` requires exactly one of `--project`, `--module`, `--feature`, `--task`.
- Removing a work entity does NOT delete linked research.
- Removing research does NOT affect linked work entities.
- `--cascade` deletes all descendants. Without it, remove fails if children exist.
