# lopen-memory — Decision Guide for LLM Agents

## Purpose of This Document

This document tells you **where to put information** and **why**, so you make correct decisions when using `lopen-memory`. Read this before using the tool for the first time on a project. For command syntax, read `SKILL.md`.

---

## The Hierarchy and What Belongs at Each Level

```
Project
└── Module
    └── Feature
        └── Task
```

### Project

A project maps to a **single codebase or initiative**. One repository = one project. A project has a `path` which should be the absolute path to the root of that codebase on disk.

**Create a project when:** you are starting work on a repository or initiative that does not yet exist in lopen-memory.

**Do not create a project for:** a sub-area of an existing codebase. That is a module.

```bash
lopen-memory project add my-app /home/user/my-app "Rewrite of the core API"
```

---

### Module

A module maps to a **major bounded area of concern** within a project. Think: domain, subsystem, or large workstream. A project will typically have between two and ten modules.

**Good module names:** `auth`, `payments`, `reporting`, `data-pipeline`, `admin-ui`

**Create a module when:** you can clearly articulate an area of the codebase that has its own concerns, data, and team ownership — or when you are about to plan a distinct body of work within a project.

**Do not create a module for:** something small enough to be a single deliverable. That is a feature.

```bash
lopen-memory module add --project my-app auth "Handles all authentication and session management"
```

---

### Feature

A feature maps to a **single discrete deliverable** within a module — something that can be built, tested, and shipped independently. A module will typically have between two and twenty features.

**Good feature names:** `login-flow`, `token-refresh`, `password-reset`, `audit-log`, `export-csv`

**Create a feature when:** you can describe it as "the ability to X" and it has a clear done state.

**Do not create a feature for:** a step required to build something else. That is a task.

```bash
lopen-memory feature add --module auth login-flow "The end-to-end flow for a user logging in and receiving a session token"
```

---

### Task

A task maps to a **single concrete implementation step** within a feature. Tasks are the smallest unit of work. A feature will typically have between two and ten tasks.

**Good task names:** `implement-jwt-issuance`, `write-integration-tests`, `add-rate-limiting`, `update-openapi-spec`

**Create a task when:** you are breaking down a feature into actionable steps that can be completed one at a time.

**Do not create a task for:** something broad enough to have multiple sub-concerns. Decompose it into a feature with tasks instead.

```bash
lopen-memory task add --feature login-flow implement-jwt-issuance "Issue a signed JWT on successful credential validation"
```

---

## description vs details — Use the Right Field

Every module, feature, and task has both a `description` and a `details` field. Using them correctly keeps records clean and stable.

### `description` — the goal

`description` is a **stable, one-sentence statement of what this item is meant to achieve**. It answers: *what is this for?*

- Set it at creation time.
- Do not update it unless the goal itself changes.
- Keep it short — one sentence is ideal.
- It should still make sense six months later without context.

```bash
lopen-memory feature set-description --feature login-flow "Allow a registered user to authenticate and receive a session token"
```

### `details` — the working notes

`details` is **evolving implementation content**. It answers: *how is this being done right now?*

- Replace it freely as work progresses.
- Use it for: implementation approach, constraints, decisions made, links to relevant code, sub-steps not worth making tasks.
- It is fully overwritten on every `set-details` call. There is no append mode.
- It is allowed to be empty.

```bash
lopen-memory feature set-details --feature login-flow "Using HS256 JWT with 1h expiry. Refresh token stored in Redis with 7d TTL. Rate limiting at 10 attempts per minute per IP."
```

**Rule of thumb:** if the information would still be true after the feature is complete, it belongs in `description`. If it could change as you build, it belongs in `details`.

---

## Research — When and How to Use It

Research is for **reference material that informed or should inform decisions**. It is not work to be done — it is knowledge to be recorded and reused.

### When to create a research record

Create a research record when you have:
- Read a specification, RFC, or standard that is relevant to work you are doing
- Benchmarked or evaluated a technology and have findings worth preserving
- Investigated a problem and reached conclusions that future work should know about
- Found an article, paper, or post that significantly shapes an implementation decision

### When NOT to create a research record

Do not use research for:
- Implementation notes (put those in `details`)
- TODO items (put those in tasks)
- General descriptions of what something does (put that in `description`)

### The three content fields

| Field | What to put in it |
|---|---|
| `description` | One sentence: what this research covers and why it is relevant |
| `content` | The full findings — notes, conclusions, key facts, quotes |
| `source` | The URL, RFC number, paper title, or citation |

```bash
lopen-memory research add jwt-rfc "The IETF specification defining JWT structure and validation rules"
lopen-memory research set-source --research jwt-rfc "https://datatracker.ietf.org/doc/html/rfc7519"
lopen-memory research set-content --research jwt-rfc "JWTs consist of three base64url-encoded parts: header, payload, signature. The alg header specifies the signing algorithm. HS256 uses a shared secret; RS256 uses a private/public key pair. The exp claim defines expiry as a Unix timestamp. Validation must check signature, expiry, and issuer."
```

### Linking research to work

After creating a research record, link it to every work entity it is relevant to. This is how `project show`, `module show`, `feature show`, and `task show` surface the research without you needing to search for it.

```bash
lopen-memory research link --research jwt-rfc --module auth
lopen-memory research link --research jwt-rfc --feature login-flow
lopen-memory research link --research jwt-rfc --task implement-jwt-issuance
```

Link at the most specific level that is relevant. If research applies to an entire module, link it to the module. If it only applies to one task, link it to the task. There is no harm in linking at multiple levels.

### Before starting work on a new project

Always search existing research before creating new records. The same research often applies across projects.

```bash
# Find anything relevant to what you are about to work on
lopen-memory research search authentication
lopen-memory research search jwt

# Find research that may be outdated and need re-verification
lopen-memory research list --stale-days 180
lopen-memory research search jwt --stale-days 365
```

If you find a relevant record that is stale, update its content and reset the date:

```bash
lopen-memory research set-content --research jwt-rfc "Updated findings..."
# set-content automatically updates researched_at to now
# Use --no-update-date if you are only fixing a typo, not re-doing the research
```

---

## The State Machine — When to Transition

All modules, features, and tasks move through the same five states. Transition them as work progresses so the state accurately reflects reality at all times.

| State | Transition into this state when |
|---|---|
| `Draft` | The item has been created but active work has not started. This is the initial state. |
| `Planning` | You are actively scoping, designing, or specifying how this will be built. |
| `Building` | Implementation is underway. |
| `Complete` | The item is finished and verified. |
| `Amending` | The item was complete but is being revised due to new requirements or a defect. |

Transition the most specific entity first. When all tasks in a feature are `Complete`, transition the feature. When all features in a module are `Complete`, transition the module.

```bash
lopen-memory task transition --task implement-jwt-issuance Building
lopen-memory task transition --task implement-jwt-issuance Complete
lopen-memory feature transition --feature login-flow Complete
lopen-memory module transition --module auth Complete
```

You can reset any item directly to `Draft` at any time if the scope changes significantly and it needs to be re-planned from scratch.

---

## Deciding What to Show

Use `show` commands to load full context before doing work on an entity. The output includes all child entities and all linked research, giving you everything relevant in one call.

| You are about to work on | Run |
|---|---|
| A project (need the full picture) | `lopen-memory project show --project <n>` |
| A specific module | `lopen-memory module show --module <n>` |
| A specific feature | `lopen-memory feature show --feature <n>` |
| A specific task | `lopen-memory task show --task <n>` |
| A research record | `lopen-memory research show --research <n>` |

---

## Cascade Removal — What Gets Deleted

When you remove an entity with `--cascade`, all descendants are deleted:

- Removing a project with `--cascade` deletes its modules, their features, and all tasks within those features.
- Removing a module with `--cascade` deletes its features and all tasks within those features.
- Removing a feature with `--cascade` deletes its tasks.

**Research is never deleted by cascade.** Only the bridge table rows linking research to the deleted entity are removed. The research record itself always survives. This is intentional — research belongs to no single entity.

Removing a research record with `research remove` does the inverse: it deletes the record and all its bridge rows, but leaves every linked work entity untouched.

---

## Common Mistakes to Avoid

**Putting implementation notes in `description`.**
Description is the goal. It should not contain "we decided to use X because Y". Put that in `details`.

**Creating a task when you need a feature.**
If the thing you are creating has multiple steps, it is a feature. Create the feature, then create tasks within it.

**Creating research and not linking it.**
Unlinked research cannot be discovered through `show` commands. Always link research to at least one work entity immediately after creating it.

**Not searching before creating research.**
Always run `research search <term>` before creating a new record. Duplicate research records create noise and diverge over time.

**Leaving states as `Draft` permanently.**
States are only useful if they are kept accurate. Transition items as work moves forward.
