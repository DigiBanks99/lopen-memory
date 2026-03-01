mod db;
mod models;
mod output;
mod resolve;
mod skill;
mod state;

use clap::{Parser, Subcommand};
use std::process;

const DEFAULT_DB: &str = "/.lopen-memory/lopen-memory.db";

fn db_path(override_path: Option<&String>) -> String {
    if let Some(p) = override_path {
        return p.clone();
    }
    if let Ok(p) = std::env::var("LOPEN_MEMORY_DB") {
        return p;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}{}", home, DEFAULT_DB)
}

// ── Top-level CLI ─────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "lopen-memory",
    about = "Persistent structured memory store for LLM coding agents",
    long_about = "Persistent structured memory store for LLM coding agents.\n\n\
        lopen-memory gives AI coding agents a durable, queryable record of what they're \
        building and what they've learned. It organises work into a four-level hierarchy:\n\n  \
        Project  →  Module  →  Feature  →  Task\n\n\
        Each level carries a `description` (the stable goal statement that rarely changes) \
        and `details` (evolving working notes, design decisions, and context that updates as \
        work progresses).\n\n\
        Every item moves through a lifecycle:\n\n  \
        Draft → Planning → Building → Complete → Amending\n\n\
        Research entries form a cross-cutting knowledge store — notes, investigations, \
        discoveries, and reference material that can be linked to any project, module, \
        feature, or task so context is never lost.\n\n\
        Use `lopen-memory <command> --help` for details on each command."
)]
struct Cli {
    /// Override the database file path
    #[arg(long, global = true)]
    db: Option<String>,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// A project maps to a single codebase or repository. Create one per repo. Contains modules
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// A major bounded area of concern within a project (e.g. auth, payments). Contains features
    Module {
        #[command(subcommand)]
        action: ModuleAction,
    },
    /// A single discrete deliverable within a module — something that can be built, tested, and shipped independently. Contains tasks
    Feature {
        #[command(subcommand)]
        action: FeatureAction,
    },
    /// A single concrete implementation step within a feature — the smallest unit of tracked work
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Reference material (specs, benchmarks, findings) that can be linked to any project, module, feature, or task
    Research {
        #[command(subcommand)]
        action: ResearchAction,
    },
    /// Install or manage the SKILL.md agent skill file that helps LLM agents discover and use lopen-memory
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

// ── Project actions ───────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ProjectAction {
    /// Register a new project. One project = one codebase or repository. Provide a unique name, the absolute path to the repo root on disk, and optionally a one-sentence description of the project's purpose
    Add {
        /// Unique slug identifying this project (used in all commands to reference it)
        name: String,
        /// Absolute filesystem path to the root of the codebase or repository
        path: String,
        /// Stable one-sentence description of the project's purpose
        description: Option<String>,
    },
    /// List all registered projects, optionally filtered to only completed or only incomplete ones
    List {
        /// Show only completed projects
        #[arg(long, conflicts_with = "incomplete")]
        completed: bool,
        /// Show only incomplete (active) projects
        #[arg(long, conflicts_with = "completed")]
        incomplete: bool,
    },
    /// Display full details for a project including its description, path, completion status, and all child modules with their current lifecycle states
    Show {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
    },
    /// Change a project's slug name. Does not affect child modules or linked research
    Rename {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
        /// The new slug name for the project
        new_name: String,
    },
    /// Replace the project's description — a stable one-sentence statement of the project's purpose. Update only when the goal itself changes
    SetDescription {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
        /// Stable one-sentence statement of the project's purpose — should still make sense months later
        description: String,
    },
    /// Update the absolute filesystem path associated with this project
    SetPath {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
        /// New absolute filesystem path to associate with this project
        path: String,
    },
    /// Mark a project as complete. Use when all work in the project is finished
    Complete {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
    },
    /// Reopen a previously completed project for further work
    Reopen {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
    },
    /// Delete a project. Use --cascade to also delete all child modules, features, and tasks. Without --cascade, removal fails if the project has children. Linked research records are never deleted — only the association is removed
    Remove {
        /// Project name or numeric ID
        #[arg(long)]
        project: String,
        /// Also delete all child modules, features, and tasks. Without this flag, removal fails if children exist
        #[arg(long)]
        cascade: bool,
    },
}

// ── Module actions ────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ModuleAction {
    /// Create a new module within a project. Modules represent major bounded areas of concern — think domain, subsystem, or large workstream (e.g. auth, payments, reporting)
    Add {
        /// Parent project name or numeric ID
        #[arg(long)]
        project: String,
        /// Unique slug identifying this module within its parent project
        name: String,
        /// Stable one-sentence description of what this area of the codebase covers
        description: Option<String>,
    },
    /// List all modules in a project, optionally filtered by lifecycle state (Draft, Planning, Building, Complete, Amending)
    List {
        /// Parent project name or numeric ID
        #[arg(long)]
        project: String,
        /// Filter by lifecycle state: Draft, Planning, Building, Complete, or Amending
        #[arg(long)]
        state: Option<String>,
    },
    /// Display full details for a module including its description, details, lifecycle state, and all child features with their states
    Show {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
    },
    /// Change a module's slug name. Does not affect child features or linked research
    Rename {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// The new slug name for the module
        new_name: String,
    },
    /// Replace the module's description — a stable one-sentence statement of what this area of the codebase covers. Update only when the scope itself changes
    SetDescription {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Stable one-sentence statement of what this module covers — should still make sense months later
        description: String,
    },
    /// Replace the module's working notes entirely. Use for implementation approach, design decisions, constraints, and evolving context. Fully overwritten on each call — there is no append mode
    SetDetails {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Implementation notes, design decisions, and evolving context. Fully replaces existing details
        details: String,
    },
    /// Move a module to a new lifecycle state: Draft, Planning, Building, Complete, or Amending. Transition children first — complete all features before completing the module
    Transition {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Target lifecycle state: Draft, Planning, Building, Complete, or Amending
        state: String,
    },
    /// Delete a module. Use --cascade to also delete all child features and tasks. Without --cascade, removal fails if children exist. Linked research is never deleted
    Remove {
        /// Module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Also delete all child features and tasks. Without this flag, removal fails if children exist
        #[arg(long)]
        cascade: bool,
    },
}

// ── Feature actions ───────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum FeatureAction {
    /// Create a new feature within a module. Features are single discrete deliverables — something that can be described as "the ability to X" with a clear done state
    Add {
        /// Parent module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Unique slug identifying this feature within its parent module
        name: String,
        /// Stable one-sentence goal — describe it as "the ability to X"
        description: Option<String>,
    },
    /// List all features in a module, optionally filtered by lifecycle state (Draft, Planning, Building, Complete, Amending)
    List {
        /// Parent module name or numeric ID
        #[arg(long)]
        module: String,
        /// Disambiguate by project name or ID if the module name is not unique
        #[arg(long)]
        project: Option<String>,
        /// Filter by lifecycle state: Draft, Planning, Building, Complete, or Amending
        #[arg(long)]
        state: Option<String>,
    },
    /// Display full details for a feature including its description, details, lifecycle state, and all child tasks with their states
    Show {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Disambiguate by project name or ID (used with --module)
        #[arg(long)]
        project: Option<String>,
    },
    /// Change a feature's slug name. Does not affect child tasks or linked research
    Rename {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// The new slug name for the feature
        new_name: String,
    },
    /// Replace the feature's description — a stable one-sentence goal like "the ability to X". Update only when the goal itself changes
    SetDescription {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Stable one-sentence goal statement — should still make sense months later without context
        description: String,
    },
    /// Replace the feature's working notes entirely. Use for implementation approach, constraints, decisions, and links to relevant code. Fully overwritten on each call
    SetDetails {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Implementation notes, design decisions, and evolving context. Fully replaces existing details
        details: String,
    },
    /// Move a feature to a new lifecycle state: Draft, Planning, Building, Complete, or Amending. Complete all child tasks before completing the feature
    Transition {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Target lifecycle state: Draft, Planning, Building, Complete, or Amending
        state: String,
    },
    /// Delete a feature. Use --cascade to also delete all child tasks. Without --cascade, removal fails if tasks exist. Linked research is never deleted
    Remove {
        /// Feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Also delete all child tasks. Without this flag, removal fails if tasks exist
        #[arg(long)]
        cascade: bool,
    },
}

// ── Task actions ──────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum TaskAction {
    /// Create a new task within a feature. Tasks are the smallest unit of tracked work — single concrete implementation steps that can be completed one at a time
    Add {
        /// Parent feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Unique slug identifying this task within its parent feature
        name: String,
        /// Stable one-sentence description of what this implementation step achieves
        description: Option<String>,
    },
    /// List all tasks in a feature, optionally filtered by lifecycle state (Draft, Planning, Building, Complete, Amending)
    List {
        /// Parent feature name or numeric ID
        #[arg(long)]
        feature: String,
        /// Disambiguate by module name or ID if the feature name is not unique
        #[arg(long)]
        module: Option<String>,
        /// Filter by lifecycle state: Draft, Planning, Building, Complete, or Amending
        #[arg(long)]
        state: Option<String>,
    },
    /// Display full details for a task including its description, details, and current lifecycle state
    Show {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
        /// Disambiguate by module name or ID (used with --feature)
        #[arg(long)]
        module: Option<String>,
    },
    /// Change a task's slug name
    Rename {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
        /// The new slug name for the task
        new_name: String,
    },
    /// Replace the task's description — a stable one-sentence statement of what this step achieves. Update only when the goal itself changes
    SetDescription {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
        /// Stable one-sentence statement of what this step achieves
        description: String,
    },
    /// Replace the task's working notes entirely. Use for implementation specifics, blockers, and evolving context. Fully overwritten on each call
    SetDetails {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
        /// Implementation specifics, blockers, and evolving context. Fully replaces existing details
        details: String,
    },
    /// Move a task to a new lifecycle state: Draft, Planning, Building, Complete, or Amending. Complete tasks before completing their parent feature
    Transition {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
        /// Target lifecycle state: Draft, Planning, Building, Complete, or Amending
        state: String,
    },
    /// Delete a task permanently. This does not affect sibling tasks or the parent feature
    Remove {
        /// Task name or numeric ID
        #[arg(long)]
        task: String,
        /// Disambiguate by feature name or ID if the task name is not unique
        #[arg(long)]
        feature: Option<String>,
    },
}

// ── Research actions ──────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ResearchAction {
    /// Create a new research record. Research captures reference material — specs, benchmarks, investigations, findings — that informed or should inform decisions. Always search before creating to avoid duplicates
    Add {
        /// Unique slug identifying this research record
        name: String,
        /// One sentence: what this research covers and why it is relevant
        description: Option<String>,
    },
    /// List all research records, optionally filtered to those not updated within a given number of days (stale)
    List {
        /// Only show records not updated within this many days
        #[arg(long)]
        stale_days: Option<i64>,
    },
    /// Display full details for a research record including its description, content, source, researched_at date, and all linked work entities
    Show {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
    },
    /// Change a research record's slug name
    Rename {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// The new slug name for the research record
        new_name: String,
    },
    /// Replace the research description — one sentence covering what this research is about and why it is relevant
    SetDescription {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// One sentence covering what this research is about and why it matters
        description: String,
    },
    /// Replace the research content — the full findings, notes, conclusions, and key facts. Automatically updates researched_at to now unless --no-update-date is passed
    SetContent {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// Full findings — notes, conclusions, key facts, and quotes from the source material
        content: String,
        /// Do not update researched_at when setting content
        #[arg(long)]
        no_update_date: bool,
    },
    /// Set or update the source reference — a URL, RFC number, paper title, or citation
    SetSource {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// URL, RFC number, paper title, or other citation for the source material
        source: String,
    },
    /// Manually override the researched_at timestamp. Use when importing research done on a known prior date
    SetResearchedAt {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// ISO 8601 date or datetime (e.g. 2025-01-15 or 2025-01-15T10:30:00Z)
        date: String,
    },
    /// Full-text search across research names, descriptions, content, and sources. Optionally filter to stale records not updated within N days
    Search {
        /// Search keyword matched against research name, description, content, and source
        term: String,
        /// Only include records not updated within this many days
        #[arg(long)]
        stale_days: Option<i64>,
    },
    /// Associate a research record with a work entity. Exactly one of --project, --module, --feature, or --task must be provided. Linking the same pair twice is a no-op
    Link {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// Link to this project (exactly one of --project, --module, --feature, --task required)
        #[arg(long)]
        project: Option<String>,
        /// Link to this module
        #[arg(long)]
        module: Option<String>,
        /// Link to this feature
        #[arg(long)]
        feature: Option<String>,
        /// Link to this task
        #[arg(long)]
        task: Option<String>,
    },
    /// Remove the association between a research record and a work entity. Exactly one of --project, --module, --feature, or --task must be provided. Unlinking a non-existent pair is a no-op
    Unlink {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
        /// Unlink from this project (exactly one of --project, --module, --feature, --task required)
        #[arg(long)]
        project: Option<String>,
        /// Unlink from this module
        #[arg(long)]
        module: Option<String>,
        /// Unlink from this feature
        #[arg(long)]
        feature: Option<String>,
        /// Unlink from this task
        #[arg(long)]
        task: Option<String>,
    },
    /// List all work entities (projects, modules, features, tasks) currently linked to a research record
    Links {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
    },
    /// Delete a research record and all its link associations. Linked work entities are never affected — only the bridge rows are removed
    Remove {
        /// Research record name or numeric ID
        #[arg(long)]
        research: String,
    },
}

// ── Skill actions ─────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum SkillAction {
    /// Copy SKILL.md to the agent skills directory so LLM coding agents can discover and use lopen-memory. The skill file teaches agents the correct command syntax and usage patterns
    Install {
        /// Override the skills directory path (default: ~/.agents/skills/lopen-memory)
        #[arg(long)]
        skills_dir: Option<String>,
    },
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let path = db_path(cli.db.as_ref());
    let conn = match db::open(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to open database: {}", e);
            process::exit(2);
        }
    };
    let json = cli.json;

    let code = match cli.command {
        Commands::Project { action } => handle_project(&conn, action, json),
        Commands::Module { action } => handle_module(&conn, action, json),
        Commands::Feature { action } => handle_feature(&conn, action, json),
        Commands::Task { action } => handle_task(&conn, action, json),
        Commands::Research { action } => handle_research(&conn, action, json),
        Commands::Skill { action } => handle_skill(action, json),
    };

    process::exit(code);
}

// ── Project handler ───────────────────────────────────────────────────────────

fn handle_project(conn: &rusqlite::Connection, action: ProjectAction, json: bool) -> i32 {
    use models::project;
    match action {
        ProjectAction::Add {
            name,
            path,
            description,
        } => project::add(conn, &name, &path, &description.unwrap_or_default(), json),

        ProjectAction::List {
            completed,
            incomplete,
        } => {
            let filter = if completed {
                Some(true)
            } else if incomplete {
                Some(false)
            } else {
                None
            };
            project::list(conn, filter, json)
        }

        ProjectAction::Show { project } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::show(conn, id, json)
        }

        ProjectAction::Rename { project, new_name } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::rename(conn, id, &new_name, json)
        }

        ProjectAction::SetDescription {
            project,
            description,
        } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::set_description(conn, id, &description, json)
        }

        ProjectAction::SetPath { project, path } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::set_path(conn, id, &path, json)
        }

        ProjectAction::Complete { project } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::set_completed(conn, id, true, json)
        }

        ProjectAction::Reopen { project } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::set_completed(conn, id, false, json)
        }

        ProjectAction::Remove { project, cascade } => {
            let id = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            project::remove(conn, id, cascade, json)
        }
    }
}

// ── Module handler ────────────────────────────────────────────────────────────

fn handle_module(conn: &rusqlite::Connection, action: ModuleAction, json: bool) -> i32 {
    use models::module;
    match action {
        ModuleAction::Add {
            project,
            name,
            description,
        } => {
            let pid = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::add(conn, pid, &name, &description.unwrap_or_default(), json)
        }

        ModuleAction::List { project, state } => {
            let pid = match resolve::resolve_project(conn, &project) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::list(conn, pid, state.as_deref(), json)
        }

        ModuleAction::Show { module, project } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::show(conn, mid, json)
        }

        ModuleAction::Rename {
            module,
            project,
            new_name,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::rename(conn, mid, &new_name, json)
        }

        ModuleAction::SetDescription {
            module,
            project,
            description,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::set_description(conn, mid, &description, json)
        }

        ModuleAction::SetDetails {
            module,
            project,
            details,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::set_details(conn, mid, &details, json)
        }

        ModuleAction::Transition {
            module,
            project,
            state,
        } => {
            let to_state = match state.parse::<state::State>() {
                Ok(s) => s,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::transition(conn, mid, &to_state, json)
        }

        ModuleAction::Remove {
            module,
            project,
            cascade,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            module::remove(conn, mid, cascade, json)
        }
    }
}

// ── Feature handler ───────────────────────────────────────────────────────────

fn handle_feature(conn: &rusqlite::Connection, action: FeatureAction, json: bool) -> i32 {
    use models::feature;
    match action {
        FeatureAction::Add {
            module,
            project,
            name,
            description,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::add(conn, mid, &name, &description.unwrap_or_default(), json)
        }

        FeatureAction::List {
            module,
            project,
            state,
        } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) {
                Ok(p) => p,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve::resolve_module(conn, &module, pid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::list(conn, mid, state.as_deref(), json)
        }

        FeatureAction::Show {
            feature,
            module,
            project: _,
        } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::show(conn, fid, json)
        }

        FeatureAction::Rename {
            feature,
            module,
            new_name,
        } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::rename(conn, fid, &new_name, json)
        }

        FeatureAction::SetDescription {
            feature,
            module,
            description,
        } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::set_description(conn, fid, &description, json)
        }

        FeatureAction::SetDetails {
            feature,
            module,
            details,
        } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::set_details(conn, fid, &details, json)
        }

        FeatureAction::Transition {
            feature,
            module,
            state,
        } => {
            let to_state = match state.parse::<state::State>() {
                Ok(s) => s,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::transition(conn, fid, &to_state, json)
        }

        FeatureAction::Remove {
            feature,
            module,
            cascade,
        } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) {
                Ok(m) => m,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve::resolve_feature(conn, &feature, mid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            feature::remove(conn, fid, cascade, json)
        }
    }
}

// ── Task handler ──────────────────────────────────────────────────────────────

fn handle_task(conn: &rusqlite::Connection, action: TaskAction, json: bool) -> i32 {
    use models::task;
    match action {
        TaskAction::Add {
            feature,
            module: _,
            name,
            description,
        } => {
            let fid = match resolve::resolve_feature(conn, &feature, None) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::add(conn, fid, &name, &description.unwrap_or_default(), json)
        }

        TaskAction::List {
            feature,
            module: _,
            state,
        } => {
            let fid = match resolve::resolve_feature(conn, &feature, None) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::list(conn, fid, state.as_deref(), json)
        }

        TaskAction::Show {
            task,
            feature,
            module: _,
        } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::show(conn, tid, json)
        }

        TaskAction::Rename {
            task,
            feature,
            new_name,
        } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::rename(conn, tid, &new_name, json)
        }

        TaskAction::SetDescription {
            task,
            feature,
            description,
        } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::set_description(conn, tid, &description, json)
        }

        TaskAction::SetDetails {
            task,
            feature,
            details,
        } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::set_details(conn, tid, &details, json)
        }

        TaskAction::Transition {
            task,
            feature,
            state,
        } => {
            let to_state = match state.parse::<state::State>() {
                Ok(s) => s,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::transition(conn, tid, &to_state, json)
        }

        TaskAction::Remove { task, feature } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) {
                Ok(f) => f,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let tid = match resolve::resolve_task(conn, &task, fid) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            task::remove(conn, tid, json)
        }
    }
}

// ── Research handler ──────────────────────────────────────────────────────────

fn handle_research(conn: &rusqlite::Connection, action: ResearchAction, json: bool) -> i32 {
    use models::research;
    match action {
        ResearchAction::Add { name, description } => {
            research::add(conn, &name, &description.unwrap_or_default(), json)
        }

        ResearchAction::List { stale_days } => research::list(conn, stale_days, json),

        ResearchAction::Show { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::show(conn, rid, json)
        }

        ResearchAction::Rename {
            research: r,
            new_name,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::rename(conn, rid, &new_name, json)
        }

        ResearchAction::SetDescription {
            research: r,
            description,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::set_description(conn, rid, &description, json)
        }

        ResearchAction::SetContent {
            research: r,
            content,
            no_update_date,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::set_content(conn, rid, &content, !no_update_date, json)
        }

        ResearchAction::SetSource {
            research: r,
            source,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::set_source(conn, rid, &source, json)
        }

        ResearchAction::SetResearchedAt { research: r, date } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::set_researched_at(conn, rid, &date, json)
        }

        ResearchAction::Search { term, stale_days } => {
            research::search(conn, &term, stale_days, json)
        }

        ResearchAction::Link {
            research: r,
            project,
            module,
            feature,
            task,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let count = [&project, &module, &feature, &task]
                .iter()
                .filter(|x| x.is_some())
                .count();
            if count != 1 {
                output::err(
                    "exactly one of --project, --module, --feature, --task must be provided",
                );
                return 1;
            }
            if let Some(p) = project {
                let pid = match resolve::resolve_project(conn, &p) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::link_project(conn, rid, pid, json)
            } else if let Some(m) = module {
                let mid = match resolve::resolve_module(conn, &m, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::link_module(conn, rid, mid, json)
            } else if let Some(f) = feature {
                let fid = match resolve::resolve_feature(conn, &f, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::link_feature(conn, rid, fid, json)
            } else if let Some(t) = task {
                let tid = match resolve::resolve_task(conn, &t, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::link_task(conn, rid, tid, json)
            } else {
                1
            }
        }

        ResearchAction::Unlink {
            research: r,
            project,
            module,
            feature,
            task,
        } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            let count = [&project, &module, &feature, &task]
                .iter()
                .filter(|x| x.is_some())
                .count();
            if count != 1 {
                output::err(
                    "exactly one of --project, --module, --feature, --task must be provided",
                );
                return 1;
            }
            if let Some(p) = project {
                let pid = match resolve::resolve_project(conn, &p) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::unlink_project(conn, rid, pid, json)
            } else if let Some(m) = module {
                let mid = match resolve::resolve_module(conn, &m, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::unlink_module(conn, rid, mid, json)
            } else if let Some(f) = feature {
                let fid = match resolve::resolve_feature(conn, &f, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::unlink_feature(conn, rid, fid, json)
            } else if let Some(t) = task {
                let tid = match resolve::resolve_task(conn, &t, None) {
                    Ok(i) => i,
                    Err(e) => {
                        output::err(&e);
                        return 1;
                    }
                };
                research::unlink_task(conn, rid, tid, json)
            } else {
                1
            }
        }

        ResearchAction::Links { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::links(conn, rid, json)
        }

        ResearchAction::Remove { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) {
                Ok(i) => i,
                Err(e) => {
                    output::err(&e);
                    return 1;
                }
            };
            research::remove(conn, rid, json)
        }
    }
}

// ── Helper resolvers ──────────────────────────────────────────────────────────

fn resolve_optional_project(
    conn: &rusqlite::Connection,
    s: Option<&str>,
) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(p) => resolve::resolve_project(conn, p).map(Some),
    }
}

fn resolve_optional_module(
    conn: &rusqlite::Connection,
    s: Option<&str>,
) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(m) => resolve::resolve_module(conn, m, None).map(Some),
    }
}

fn resolve_optional_feature(
    conn: &rusqlite::Connection,
    s: Option<&str>,
) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(f) => resolve::resolve_feature(conn, f, None).map(Some),
    }
}

// ── Skill handler ─────────────────────────────────────────────────────────────

fn handle_skill(action: SkillAction, json: bool) -> i32 {
    match action {
        SkillAction::Install { skills_dir } => skill::install(skills_dir.as_ref(), json),
    }
}
