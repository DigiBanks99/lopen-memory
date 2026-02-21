mod db;
mod state;
mod resolve;
mod output;
mod models;

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
#[command(name = "lopen-memory", about = "Project and research memory tracker")]
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
    /// Manage projects
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Manage modules
    Module {
        #[command(subcommand)]
        action: ModuleAction,
    },
    /// Manage features
    Feature {
        #[command(subcommand)]
        action: FeatureAction,
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Manage research
    Research {
        #[command(subcommand)]
        action: ResearchAction,
    },
}

// ── Project actions ───────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ProjectAction {
    /// Add a new project
    Add {
        name: String,
        path: String,
        description: Option<String>,
    },
    /// List projects
    List {
        #[arg(long, conflicts_with = "incomplete")]
        completed: bool,
        #[arg(long, conflicts_with = "completed")]
        incomplete: bool,
    },
    /// Show a project
    Show {
        #[arg(long)]
        project: String,
    },
    /// Rename a project
    Rename {
        #[arg(long)]
        project: String,
        new_name: String,
    },
    /// Set project description
    SetDescription {
        #[arg(long)]
        project: String,
        description: String,
    },
    /// Set project path
    SetPath {
        #[arg(long)]
        project: String,
        path: String,
    },
    /// Mark a project as complete
    Complete {
        #[arg(long)]
        project: String,
    },
    /// Reopen a completed project
    Reopen {
        #[arg(long)]
        project: String,
    },
    /// Remove a project
    Remove {
        #[arg(long)]
        project: String,
        #[arg(long)]
        cascade: bool,
    },
}

// ── Module actions ────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ModuleAction {
    Add {
        #[arg(long)]
        project: String,
        name: String,
        description: Option<String>,
    },
    List {
        #[arg(long)]
        project: String,
        #[arg(long)]
        state: Option<String>,
    },
    Show {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
    },
    Rename {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        new_name: String,
    },
    SetDescription {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        description: String,
    },
    SetDetails {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        details: String,
    },
    Transition {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        state: String,
    },
    Remove {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        cascade: bool,
    },
}

// ── Feature actions ───────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum FeatureAction {
    Add {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        name: String,
        description: Option<String>,
    },
    List {
        #[arg(long)]
        module: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },
    Show {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        project: Option<String>,
    },
    Rename {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        new_name: String,
    },
    SetDescription {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        description: String,
    },
    SetDetails {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        details: String,
    },
    Transition {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        state: String,
    },
    Remove {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        cascade: bool,
    },
}

// ── Task actions ──────────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum TaskAction {
    Add {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        name: String,
        description: Option<String>,
    },
    List {
        #[arg(long)]
        feature: String,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },
    Show {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
        #[arg(long)]
        module: Option<String>,
    },
    Rename {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
        new_name: String,
    },
    SetDescription {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
        description: String,
    },
    SetDetails {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
        details: String,
    },
    Transition {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
        state: String,
    },
    Remove {
        #[arg(long)]
        task: String,
        #[arg(long)]
        feature: Option<String>,
    },
}

// ── Research actions ──────────────────────────────────────────────────────────

#[derive(Subcommand)]
enum ResearchAction {
    Add {
        name: String,
        description: Option<String>,
    },
    List {
        #[arg(long)]
        stale_days: Option<i64>,
    },
    Show {
        #[arg(long)]
        research: String,
    },
    Rename {
        #[arg(long)]
        research: String,
        new_name: String,
    },
    SetDescription {
        #[arg(long)]
        research: String,
        description: String,
    },
    SetContent {
        #[arg(long)]
        research: String,
        content: String,
        /// Do not update researched_at when setting content
        #[arg(long)]
        no_update_date: bool,
    },
    SetSource {
        #[arg(long)]
        research: String,
        source: String,
    },
    SetResearchedAt {
        #[arg(long)]
        research: String,
        date: String,
    },
    Search {
        term: String,
        #[arg(long)]
        stale_days: Option<i64>,
    },
    Link {
        #[arg(long)]
        research: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        feature: Option<String>,
        #[arg(long)]
        task: Option<String>,
    },
    Unlink {
        #[arg(long)]
        research: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        module: Option<String>,
        #[arg(long)]
        feature: Option<String>,
        #[arg(long)]
        task: Option<String>,
    },
    Links {
        #[arg(long)]
        research: String,
    },
    Remove {
        #[arg(long)]
        research: String,
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
        Commands::Module  { action } => handle_module(&conn, action, json),
        Commands::Feature { action } => handle_feature(&conn, action, json),
        Commands::Task    { action } => handle_task(&conn, action, json),
        Commands::Research{ action } => handle_research(&conn, action, json),
    };

    process::exit(code);
}

// ── Project handler ───────────────────────────────────────────────────────────

fn handle_project(conn: &rusqlite::Connection, action: ProjectAction, json: bool) -> i32 {
    use models::project;
    match action {
        ProjectAction::Add { name, path, description } =>
            project::add(conn, &name, &path, &description.unwrap_or_default(), json),

        ProjectAction::List { completed, incomplete } => {
            let filter = if completed { Some(true) } else if incomplete { Some(false) } else { None };
            project::list(conn, filter, json)
        }

        ProjectAction::Show { project } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::show(conn, id, json)
        }

        ProjectAction::Rename { project, new_name } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::rename(conn, id, &new_name, json)
        }

        ProjectAction::SetDescription { project, description } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::set_description(conn, id, &description, json)
        }

        ProjectAction::SetPath { project, path } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::set_path(conn, id, &path, json)
        }

        ProjectAction::Complete { project } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::set_completed(conn, id, true, json)
        }

        ProjectAction::Reopen { project } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::set_completed(conn, id, false, json)
        }

        ProjectAction::Remove { project, cascade } => {
            let id = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            project::remove(conn, id, cascade, json)
        }
    }
}

// ── Module handler ────────────────────────────────────────────────────────────

fn handle_module(conn: &rusqlite::Connection, action: ModuleAction, json: bool) -> i32 {
    use models::module;
    match action {
        ModuleAction::Add { project, name, description } => {
            let pid = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::add(conn, pid, &name, &description.unwrap_or_default(), json)
        }

        ModuleAction::List { project, state } => {
            let pid = match resolve::resolve_project(conn, &project) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::list(conn, pid, state.as_deref(), json)
        }

        ModuleAction::Show { module, project } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::show(conn, mid, json)
        }

        ModuleAction::Rename { module, project, new_name } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::rename(conn, mid, &new_name, json)
        }

        ModuleAction::SetDescription { module, project, description } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::set_description(conn, mid, &description, json)
        }

        ModuleAction::SetDetails { module, project, details } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::set_details(conn, mid, &details, json)
        }

        ModuleAction::Transition { module, project, state } => {
            let to_state = match state.parse::<state::State>() { Ok(s) => s, Err(e) => { output::err(&e); return 1; } };
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::transition(conn, mid, &to_state, json)
        }

        ModuleAction::Remove { module, project, cascade } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            module::remove(conn, mid, cascade, json)
        }
    }
}

// ── Feature handler ───────────────────────────────────────────────────────────

fn handle_feature(conn: &rusqlite::Connection, action: FeatureAction, json: bool) -> i32 {
    use models::feature;
    match action {
        FeatureAction::Add { module, project, name, description } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::add(conn, mid, &name, &description.unwrap_or_default(), json)
        }

        FeatureAction::List { module, project, state } => {
            let pid = match resolve_optional_project(conn, project.as_deref()) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve::resolve_module(conn, &module, pid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::list(conn, mid, state.as_deref(), json)
        }

        FeatureAction::Show { feature, module, project: _ } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::show(conn, fid, json)
        }

        FeatureAction::Rename { feature, module, new_name } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::rename(conn, fid, &new_name, json)
        }

        FeatureAction::SetDescription { feature, module, description } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::set_description(conn, fid, &description, json)
        }

        FeatureAction::SetDetails { feature, module, details } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::set_details(conn, fid, &details, json)
        }

        FeatureAction::Transition { feature, module, state } => {
            let to_state = match state.parse::<state::State>() { Ok(s) => s, Err(e) => { output::err(&e); return 1; } };
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::transition(conn, fid, &to_state, json)
        }

        FeatureAction::Remove { feature, module, cascade } => {
            let mid = match resolve_optional_module(conn, module.as_deref()) { Ok(m) => m, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve::resolve_feature(conn, &feature, mid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            feature::remove(conn, fid, cascade, json)
        }
    }
}

// ── Task handler ──────────────────────────────────────────────────────────────

fn handle_task(conn: &rusqlite::Connection, action: TaskAction, json: bool) -> i32 {
    use models::task;
    match action {
        TaskAction::Add { feature, module: _, name, description } => {
            let fid = match resolve::resolve_feature(conn, &feature, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::add(conn, fid, &name, &description.unwrap_or_default(), json)
        }

        TaskAction::List { feature, module: _, state } => {
            let fid = match resolve::resolve_feature(conn, &feature, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::list(conn, fid, state.as_deref(), json)
        }

        TaskAction::Show { task, feature, module: _ } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::show(conn, tid, json)
        }

        TaskAction::Rename { task, feature, new_name } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::rename(conn, tid, &new_name, json)
        }

        TaskAction::SetDescription { task, feature, description } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::set_description(conn, tid, &description, json)
        }

        TaskAction::SetDetails { task, feature, details } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::set_details(conn, tid, &details, json)
        }

        TaskAction::Transition { task, feature, state } => {
            let to_state = match state.parse::<state::State>() { Ok(s) => s, Err(e) => { output::err(&e); return 1; } };
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::transition(conn, tid, &to_state, json)
        }

        TaskAction::Remove { task, feature } => {
            let fid = match resolve_optional_feature(conn, feature.as_deref()) { Ok(f) => f, Err(e) => { output::err(&e); return 1; } };
            let tid = match resolve::resolve_task(conn, &task, fid) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            task::remove(conn, tid, json)
        }
    }
}

// ── Research handler ──────────────────────────────────────────────────────────

fn handle_research(conn: &rusqlite::Connection, action: ResearchAction, json: bool) -> i32 {
    use models::research;
    match action {
        ResearchAction::Add { name, description } =>
            research::add(conn, &name, &description.unwrap_or_default(), json),

        ResearchAction::List { stale_days } =>
            research::list(conn, stale_days, json),

        ResearchAction::Show { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::show(conn, rid, json)
        }

        ResearchAction::Rename { research: r, new_name } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::rename(conn, rid, &new_name, json)
        }

        ResearchAction::SetDescription { research: r, description } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::set_description(conn, rid, &description, json)
        }

        ResearchAction::SetContent { research: r, content, no_update_date } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::set_content(conn, rid, &content, !no_update_date, json)
        }

        ResearchAction::SetSource { research: r, source } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::set_source(conn, rid, &source, json)
        }

        ResearchAction::SetResearchedAt { research: r, date } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::set_researched_at(conn, rid, &date, json)
        }

        ResearchAction::Search { term, stale_days } =>
            research::search(conn, &term, stale_days, json),

        ResearchAction::Link { research: r, project, module, feature, task } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            let count = [&project, &module, &feature, &task].iter().filter(|x| x.is_some()).count();
            if count != 1 {
                output::err("exactly one of --project, --module, --feature, --task must be provided");
                return 1;
            }
            if let Some(p) = project {
                let pid = match resolve::resolve_project(conn, &p) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::link_project(conn, rid, pid, json)
            } else if let Some(m) = module {
                let mid = match resolve::resolve_module(conn, &m, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::link_module(conn, rid, mid, json)
            } else if let Some(f) = feature {
                let fid = match resolve::resolve_feature(conn, &f, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::link_feature(conn, rid, fid, json)
            } else if let Some(t) = task {
                let tid = match resolve::resolve_task(conn, &t, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::link_task(conn, rid, tid, json)
            } else { 1 }
        }

        ResearchAction::Unlink { research: r, project, module, feature, task } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            let count = [&project, &module, &feature, &task].iter().filter(|x| x.is_some()).count();
            if count != 1 {
                output::err("exactly one of --project, --module, --feature, --task must be provided");
                return 1;
            }
            if let Some(p) = project {
                let pid = match resolve::resolve_project(conn, &p) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::unlink_project(conn, rid, pid, json)
            } else if let Some(m) = module {
                let mid = match resolve::resolve_module(conn, &m, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::unlink_module(conn, rid, mid, json)
            } else if let Some(f) = feature {
                let fid = match resolve::resolve_feature(conn, &f, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::unlink_feature(conn, rid, fid, json)
            } else if let Some(t) = task {
                let tid = match resolve::resolve_task(conn, &t, None) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
                research::unlink_task(conn, rid, tid, json)
            } else { 1 }
        }

        ResearchAction::Links { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::links(conn, rid, json)
        }

        ResearchAction::Remove { research: r } => {
            let rid = match resolve::resolve_research(conn, &r) { Ok(i) => i, Err(e) => { output::err(&e); return 1; } };
            research::remove(conn, rid, json)
        }
    }
}

// ── Helper resolvers ──────────────────────────────────────────────────────────

fn resolve_optional_project(conn: &rusqlite::Connection, s: Option<&str>) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(p) => resolve::resolve_project(conn, p).map(Some),
    }
}

fn resolve_optional_module(conn: &rusqlite::Connection, s: Option<&str>) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(m) => resolve::resolve_module(conn, m, None).map(Some),
    }
}

fn resolve_optional_feature(conn: &rusqlite::Connection, s: Option<&str>) -> Result<Option<i64>, String> {
    match s {
        None => Ok(None),
        Some(f) => resolve::resolve_feature(conn, f, None).map(Some),
    }
}
