use crate::output;
use crate::state::{validate_transition, State};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

pub struct Module {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub description: String,
    pub details: String,
    pub state: String,
    pub last_worked_on: String,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load(conn: &Connection, id: i64) -> Result<Module, String> {
    conn.query_row(
        "SELECT id, project_id, name, description, details, state, last_worked_on FROM modules WHERE id=?1",
        params![id],
        |r| Ok(Module {
            id: r.get(0)?,
            project_id: r.get(1)?,
            name: r.get(2)?,
            description: r.get(3)?,
            details: r.get(4)?,
            state: r.get(5)?,
            last_worked_on: r.get(6)?,
        }),
    )
    .map_err(|_| format!("module not found: {}", id))
}

fn module_to_json(m: &Module) -> Value {
    json!({
        "id": m.id,
        "project_id": m.project_id,
        "name": m.name,
        "description": m.description,
        "details": m.details,
        "state": m.state,
        "last_worked_on": m.last_worked_on,
    })
}

pub fn add(conn: &Connection, project_id: i64, name: &str, description: &str, json: bool) -> i32 {
    let name = name.trim();
    if name.is_empty() {
        output::err("name must not be empty");
        return 1;
    }
    // get project name for output
    let project_name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE id=?1",
            params![project_id],
            |r| r.get(0),
        )
        .unwrap_or_default();
    let ts = now();
    match conn.execute(
        "INSERT INTO modules (project_id, name, description, last_worked_on) VALUES (?1,?2,?3,?4)",
        params![project_id, name, description, ts],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            if json {
                output::print_json(&load(conn, id).map(|m| module_to_json(&m)).unwrap());
            } else {
                output::print_plain(&format!(
                    "added module {}: {} (project: {})",
                    id, name, project_name
                ));
            }
            0
        }
        Err(e) => {
            output::err(&e.to_string());
            2
        }
    }
}

pub fn list(conn: &Connection, project_id: i64, state_filter: Option<&str>, json: bool) -> i32 {
    let modules: Vec<Module> = if let Some(s) = state_filter {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, details, state, last_worked_on FROM modules WHERE project_id=?1 AND state=?2 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![project_id, s], |r| {
            Ok(Module {
                id: r.get(0)?,
                project_id: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                details: r.get(4)?,
                state: r.get(5)?,
                last_worked_on: r.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, project_id, name, description, details, state, last_worked_on FROM modules WHERE project_id=?1 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![project_id], |r| {
            Ok(Module {
                id: r.get(0)?,
                project_id: r.get(1)?,
                name: r.get(2)?,
                description: r.get(3)?,
                details: r.get(4)?,
                state: r.get(5)?,
                last_worked_on: r.get(6)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    };
    if modules.is_empty() {
        output::print_plain("no modules found");
        return 0;
    }
    if json {
        output::print_json(&Value::Array(modules.iter().map(module_to_json).collect()));
    } else {
        for m in &modules {
            println!(
                "{:<4} {:<20} {:<12} {}",
                m.id, m.name, m.state, m.last_worked_on
            );
        }
    }
    0
}

pub fn show(conn: &Connection, id: i64, json: bool) -> i32 {
    let m = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    let project_name: String = conn
        .query_row(
            "SELECT name FROM projects WHERE id=?1",
            params![m.project_id],
            |r| r.get(0),
        )
        .unwrap_or_default();

    let mut fstmt = conn
        .prepare("SELECT id, name, state FROM features WHERE module_id=?1 ORDER BY id")
        .unwrap();
    let features: Vec<(i64, String, String)> = fstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut rstmt = conn
        .prepare(
            "SELECT r.id, r.name, r.description FROM research r
         JOIN research_modules rm ON rm.research_id=r.id
         WHERE rm.module_id=?1 ORDER BY r.id",
        )
        .unwrap();
    let research: Vec<(i64, String, String)> = rstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    if json {
        let mut v = module_to_json(&m);
        v["project"] = Value::String(project_name);
        v["features"] = Value::Array(
            features
                .iter()
                .map(|(id, name, state)| json!({"id": id, "name": name, "state": state}))
                .collect(),
        );
        v["research"] = Value::Array(
            research
                .iter()
                .map(|(id, name, desc)| json!({"id": id, "name": name, "description": desc}))
                .collect(),
        );
        output::print_json(&v);
    } else {
        println!("{}", output::field("id", &m.id.to_string()));
        println!("{}", output::field("name", &m.name));
        println!("{}", output::field("project", &project_name));
        println!("{}", output::field("description", &m.description));
        println!("{}", output::field("details", &m.details));
        println!("{}", output::field("state", &m.state));
        println!("{}", output::field("last_worked_on", &m.last_worked_on));
        if !features.is_empty() {
            println!();
            println!("features:");
            for (fid, fname, fstate) in &features {
                println!("  {:<4} {:<20} {}", fid, fname, fstate);
            }
        }
        if !research.is_empty() {
            println!();
            println!("research:");
            for (rid, rname, rdesc) in &research {
                println!("  {:<4} {:<20} {}", rid, rname, rdesc);
            }
        }
    }
    0
}

pub fn rename(conn: &Connection, id: i64, new_name: &str, json: bool) -> i32 {
    let new_name = new_name.trim();
    if new_name.is_empty() {
        output::err("name must not be empty");
        return 1;
    }
    let old = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE modules SET name=?1, last_worked_on=?2 WHERE id=?3",
        params![new_name, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|m| module_to_json(&m)).unwrap());
    } else {
        output::print_plain(&format!(
            "renamed module {}: {} → {}",
            id, old.name, new_name
        ));
    }
    0
}

pub fn set_description(conn: &Connection, id: i64, desc: &str, json: bool) -> i32 {
    let m = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE modules SET description=?1, last_worked_on=?2 WHERE id=?3",
        params![desc, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|m| module_to_json(&m)).unwrap());
    } else {
        output::print_plain(&format!("updated description for module: {}", m.name));
    }
    0
}

pub fn set_details(conn: &Connection, id: i64, details: &str, json: bool) -> i32 {
    let m = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE modules SET details=?1, last_worked_on=?2 WHERE id=?3",
        params![details, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|m| module_to_json(&m)).unwrap());
    } else {
        output::print_plain(&format!("updated details for module: {}", m.name));
    }
    0
}

pub fn transition(conn: &Connection, id: i64, to_state: &State, json: bool) -> i32 {
    let m = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    match validate_transition(&m.state, to_state) {
        Err(e) => {
            output::err(&format!("{} for module {}", e, m.name));
            return 1;
        }
        Ok(false) => return 0, // no-op
        Ok(true) => {}
    }
    let from = m.state.clone();
    conn.execute(
        "UPDATE modules SET state=?1, last_worked_on=?2 WHERE id=?3",
        params![to_state.to_string(), now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|m| module_to_json(&m)).unwrap());
    } else {
        output::print_plain(&format!("module {}: {} → {}", m.name, from, to_state));
    }
    0
}

pub fn remove(conn: &Connection, id: i64, cascade: bool, json: bool) -> i32 {
    let m = match load(conn, id) {
        Ok(m) => m,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM features WHERE module_id=?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if count > 0 && !cascade {
        output::err(&format!(
            "module has {} feature(s); pass --cascade to remove them",
            count
        ));
        return 1;
    }
    conn.execute("DELETE FROM modules WHERE id=?1", params![id])
        .unwrap();
    if json {
        output::print_json(&json!({"deleted": true, "id": id}));
    } else {
        output::print_plain(&format!("removed module {}: {}", id, m.name));
    }
    0
}
