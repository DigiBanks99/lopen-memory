use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open(path: &str) -> Result<Connection> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    init_schema(&conn)?;
    Ok(conn)
}

fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS projects (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT    NOT NULL UNIQUE,
            path         TEXT    NOT NULL,
            description  TEXT    NOT NULL DEFAULT '',
            completed    INTEGER NOT NULL DEFAULT 0,
            updated_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS modules (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id     INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name           TEXT    NOT NULL,
            description    TEXT    NOT NULL DEFAULT '',
            details        TEXT    NOT NULL DEFAULT '',
            state          TEXT    NOT NULL DEFAULT 'Draft'
                               CHECK(state IN ('Draft','Planning','Building','Complete','Amending')),
            last_worked_on TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS features (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            module_id      INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
            name           TEXT    NOT NULL,
            description    TEXT    NOT NULL DEFAULT '',
            details        TEXT    NOT NULL DEFAULT '',
            state          TEXT    NOT NULL DEFAULT 'Draft'
                               CHECK(state IN ('Draft','Planning','Building','Complete','Amending')),
            last_worked_on TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            UNIQUE(module_id, name)
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            feature_id     INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
            name           TEXT    NOT NULL,
            description    TEXT    NOT NULL DEFAULT '',
            details        TEXT    NOT NULL DEFAULT '',
            state          TEXT    NOT NULL DEFAULT 'Draft'
                               CHECK(state IN ('Draft','Planning','Building','Complete','Amending')),
            last_worked_on TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            UNIQUE(feature_id, name)
        );

        CREATE TABLE IF NOT EXISTS research (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT    NOT NULL UNIQUE,
            description   TEXT    NOT NULL DEFAULT '',
            content       TEXT    NOT NULL DEFAULT '',
            source        TEXT    NOT NULL DEFAULT '',
            researched_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        );

        CREATE TABLE IF NOT EXISTS research_projects (
            research_id  INTEGER NOT NULL REFERENCES research(id) ON DELETE CASCADE,
            project_id   INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            PRIMARY KEY (research_id, project_id)
        );

        CREATE TABLE IF NOT EXISTS research_modules (
            research_id  INTEGER NOT NULL REFERENCES research(id) ON DELETE CASCADE,
            module_id    INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
            PRIMARY KEY (research_id, module_id)
        );

        CREATE TABLE IF NOT EXISTS research_features (
            research_id  INTEGER NOT NULL REFERENCES research(id) ON DELETE CASCADE,
            feature_id   INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
            PRIMARY KEY (research_id, feature_id)
        );

        CREATE TABLE IF NOT EXISTS research_tasks (
            research_id  INTEGER NOT NULL REFERENCES research(id) ON DELETE CASCADE,
            task_id      INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            PRIMARY KEY (research_id, task_id)
        );
    ")
}
