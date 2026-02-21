use crate::output;
use crate::state::{validate_transition, State};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

pub struct Task {
    pub id: i64,
    pub feature_id: i64,
    pub name: String,
    pub description: String,
    pub details: String,
    pub state: String,
    pub last_worked_on: String,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load(conn: &Connection, id: i64) -> Result<Task, String> {
    conn.query_row(
        "SELECT id, feature_id, name, description, details, state, last_worked_on FROM tasks WHERE id=?1",
        params![id],
        |r| Ok(Task {
            id: r.get(0)?, feature_id: r.get(1)?, name: r.get(2)?,
            description: r.get(3)?, details: r.get(4)?, state: r.get(5)?, last_worked_on: r.get(6)?,
        }),
    )
    .map_err(|_| format!("task not found: {}", id))
}

fn task_to_json(t: &Task) -> Value {
    json!({
        "id": t.id, "feature_id": t.feature_id, "name": t.name,
        "description": t.description, "details": t.details,
        "state": t.state, "last_worked_on": t.last_worked_on,
    })
}

fn feature_name(conn: &Connection, feature_id: i64) -> String {
    conn.query_row(
        "SELECT name FROM features WHERE id=?1",
        params![feature_id],
        |r| r.get(0),
    )
    .unwrap_or_default()
}

pub fn add(conn: &Connection, feature_id: i64, name: &str, description: &str, json: bool) -> i32 {
    let name = name.trim();
    if name.is_empty() {
        output::err("name must not be empty");
        return 1;
    }
    let fname = feature_name(conn, feature_id);
    let ts = now();
    match conn.execute(
        "INSERT INTO tasks (feature_id, name, description, last_worked_on) VALUES (?1,?2,?3,?4)",
        params![feature_id, name, description, ts],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            if json {
                output::print_json(&load(conn, id).map(|t| task_to_json(&t)).unwrap());
            } else {
                output::print_plain(&format!("added task {}: {} (feature: {})", id, name, fname));
            }
            0
        }
        Err(e) => {
            output::err(&e.to_string());
            2
        }
    }
}

pub fn list(conn: &Connection, feature_id: i64, state_filter: Option<&str>, json: bool) -> i32 {
    let tasks: Vec<Task> = if let Some(s) = state_filter {
        let mut stmt = conn.prepare(
            "SELECT id, feature_id, name, description, details, state, last_worked_on FROM tasks WHERE feature_id=?1 AND state=?2 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![feature_id, s], |r| {
            Ok(Task {
                id: r.get(0)?,
                feature_id: r.get(1)?,
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
            "SELECT id, feature_id, name, description, details, state, last_worked_on FROM tasks WHERE feature_id=?1 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![feature_id], |r| {
            Ok(Task {
                id: r.get(0)?,
                feature_id: r.get(1)?,
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
    if tasks.is_empty() {
        output::print_plain("no tasks found");
        return 0;
    }
    if json {
        output::print_json(&Value::Array(tasks.iter().map(task_to_json).collect()));
    } else {
        for t in &tasks {
            println!(
                "{:<4} {:<20} {:<12} {}",
                t.id, t.name, t.state, t.last_worked_on
            );
        }
    }
    0
}

pub fn show(conn: &Connection, id: i64, json: bool) -> i32 {
    let t = match load(conn, id) {
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    let fname = feature_name(conn, t.feature_id);

    let mut rstmt = conn
        .prepare(
            "SELECT r.id, r.name, r.description FROM research r
         JOIN research_tasks rt ON rt.research_id=r.id
         WHERE rt.task_id=?1 ORDER BY r.id",
        )
        .unwrap();
    let research: Vec<(i64, String, String)> = rstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    if json {
        let mut v = task_to_json(&t);
        v["feature"] = Value::String(fname);
        v["research"] = Value::Array(
            research
                .iter()
                .map(|(id, name, desc)| json!({"id": id, "name": name, "description": desc}))
                .collect(),
        );
        output::print_json(&v);
    } else {
        println!("{}", output::field("id", &t.id.to_string()));
        println!("{}", output::field("name", &t.name));
        println!("{}", output::field("feature", &fname));
        println!("{}", output::field("description", &t.description));
        println!("{}", output::field("details", &t.details));
        println!("{}", output::field("state", &t.state));
        println!("{}", output::field("last_worked_on", &t.last_worked_on));
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
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE tasks SET name=?1, last_worked_on=?2 WHERE id=?3",
        params![new_name, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|t| task_to_json(&t)).unwrap());
    } else {
        output::print_plain(&format!("renamed task {}: {} → {}", id, old.name, new_name));
    }
    0
}

pub fn set_description(conn: &Connection, id: i64, desc: &str, json: bool) -> i32 {
    let t = match load(conn, id) {
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE tasks SET description=?1, last_worked_on=?2 WHERE id=?3",
        params![desc, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|t| task_to_json(&t)).unwrap());
    } else {
        output::print_plain(&format!("updated description for task: {}", t.name));
    }
    0
}

pub fn set_details(conn: &Connection, id: i64, details: &str, json: bool) -> i32 {
    let t = match load(conn, id) {
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE tasks SET details=?1, last_worked_on=?2 WHERE id=?3",
        params![details, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|t| task_to_json(&t)).unwrap());
    } else {
        output::print_plain(&format!("updated details for task: {}", t.name));
    }
    0
}

pub fn transition(conn: &Connection, id: i64, to_state: &State, json: bool) -> i32 {
    let t = match load(conn, id) {
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    match validate_transition(&t.state, to_state) {
        Err(e) => {
            output::err(&format!("{} for task {}", e, t.name));
            return 1;
        }
        Ok(false) => return 0,
        Ok(true) => {}
    }
    let from = t.state.clone();
    conn.execute(
        "UPDATE tasks SET state=?1, last_worked_on=?2 WHERE id=?3",
        params![to_state.to_string(), now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|t| task_to_json(&t)).unwrap());
    } else {
        output::print_plain(&format!("task {}: {} → {}", t.name, from, to_state));
    }
    0
}

pub fn remove(conn: &Connection, id: i64, json: bool) -> i32 {
    let t = match load(conn, id) {
        Ok(t) => t,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute("DELETE FROM tasks WHERE id=?1", params![id])
        .unwrap();
    if json {
        output::print_json(&json!({"deleted": true, "id": id}));
    } else {
        output::print_plain(&format!("removed task {}: {}", id, t.name));
    }
    0
}
