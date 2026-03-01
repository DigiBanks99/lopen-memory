---
name: lopen-memory
description: "Use this skill whenever you need to manage projects, modules, features, tasks, or research using the lopen-memory CLI. Triggers include: any request to create, list, show, update, or remove a project, module, feature, or task; any request to add, search, link, or manage research; any request to track work state or transition a module/feature/task through its lifecycle (Draft, Planning, Building, Complete, Amending). Use this skill before running any lopen-memory command to ensure correct syntax and avoid mistakes."
---

# lopen-memory Skill

## Overview

`lopen-memory` is a CLI tool that tracks software projects and their breakdown across four levels: **Project → Module → Feature → Task**. Research is a root-level entity that can be linked to any of those levels via bridge tables.

All input is plain text positional arguments or named flags. No JSON input. Output is plain text by default; add `--json` for machine-readable output.

## Commands

Use `lopen-memory --help` for a full list of commands and options and their usage. You can also see help for subcommands, e.g. `lopen-memory project --help` or `lopen-memory research link --help`.