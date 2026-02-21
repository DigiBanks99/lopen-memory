use crate::output;
use crate::state::{validate_transition, State};
use rusqlite::{params, Connection};
use serde_json::{json, Value};

pub struct Feature {
    pub id: i64,
    pub module_id: i64,
    pub name: String,
    pub description: String,
    pub details: String,
    pub state: String,
    pub last_worked_on: String,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load(conn: &Connection, id: i64) -> Result<Feature, String> {
    conn.query_row(
        "SELECT id, module_id, name, description, details, state, last_worked_on FROM features WHERE id=?1",
        params![id],
        |r| Ok(Feature {
            id: r.get(0)?, module_id: r.get(1)?, name: r.get(2)?,
            description: r.get(3)?, details: r.get(4)?, state: r.get(5)?, last_worked_on: r.get(6)?,
        }),
    )
    .map_err(|_| format!("feature not found: {}", id))
}

fn feature_to_json(f: &Feature) -> Value {
    json!({
        "id": f.id, "module_id": f.module_id, "name": f.name,
        "description": f.description, "details": f.details,
        "state": f.state, "last_worked_on": f.last_worked_on,
    })
}

fn module_name(conn: &Connection, module_id: i64) -> String {
    conn.query_row(
        "SELECT name FROM modules WHERE id=?1",
        params![module_id],
        |r| r.get(0),
    )
    .unwrap_or_default()
}

pub fn add(conn: &Connection, module_id: i64, name: &str, description: &str, json: bool) -> i32 {
    let name = name.trim();
    if name.is_empty() {
        output::err("name must not be empty");
        return 1;
    }
    let mname = module_name(conn, module_id);
    let ts = now();
    match conn.execute(
        "INSERT INTO features (module_id, name, description, last_worked_on) VALUES (?1,?2,?3,?4)",
        params![module_id, name, description, ts],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            if json {
                output::print_json(&load(conn, id).map(|f| feature_to_json(&f)).unwrap());
            } else {
                output::print_plain(&format!(
                    "added feature {}: {} (module: {})",
                    id, name, mname
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

pub fn list(conn: &Connection, module_id: i64, state_filter: Option<&str>, json: bool) -> i32 {
    let features: Vec<Feature> = if let Some(s) = state_filter {
        let mut stmt = conn.prepare(
            "SELECT id, module_id, name, description, details, state, last_worked_on FROM features WHERE module_id=?1 AND state=?2 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![module_id, s], |r| {
            Ok(Feature {
                id: r.get(0)?,
                module_id: r.get(1)?,
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
            "SELECT id, module_id, name, description, details, state, last_worked_on FROM features WHERE module_id=?1 ORDER BY id"
        ).unwrap();
        stmt.query_map(params![module_id], |r| {
            Ok(Feature {
                id: r.get(0)?,
                module_id: r.get(1)?,
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
    if features.is_empty() {
        output::print_plain("no features found");
        return 0;
    }
    if json {
        output::print_json(&Value::Array(
            features.iter().map(feature_to_json).collect(),
        ));
    } else {
        for f in &features {
            println!(
                "{:<4} {:<20} {:<12} {}",
                f.id, f.name, f.state, f.last_worked_on
            );
        }
    }
    0
}

pub fn show(conn: &Connection, id: i64, json: bool) -> i32 {
    let f = match load(conn, id) {
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    let mname = module_name(conn, f.module_id);

    let mut tstmt = conn
        .prepare("SELECT id, name, state FROM tasks WHERE feature_id=?1 ORDER BY id")
        .unwrap();
    let tasks: Vec<(i64, String, String)> = tstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let mut rstmt = conn
        .prepare(
            "SELECT r.id, r.name, r.description FROM research r
         JOIN research_features rf ON rf.research_id=r.id
         WHERE rf.feature_id=?1 ORDER BY r.id",
        )
        .unwrap();
    let research: Vec<(i64, String, String)> = rstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    if json {
        let mut v = feature_to_json(&f);
        v["module"] = Value::String(mname);
        v["tasks"] = Value::Array(
            tasks
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
        println!("{}", output::field("id", &f.id.to_string()));
        println!("{}", output::field("name", &f.name));
        println!("{}", output::field("module", &mname));
        println!("{}", output::field("description", &f.description));
        println!("{}", output::field("details", &f.details));
        println!("{}", output::field("state", &f.state));
        println!("{}", output::field("last_worked_on", &f.last_worked_on));
        if !tasks.is_empty() {
            println!();
            println!("tasks:");
            for (tid, tname, tstate) in &tasks {
                println!("  {:<4} {:<20} {}", tid, tname, tstate);
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
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE features SET name=?1, last_worked_on=?2 WHERE id=?3",
        params![new_name, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|f| feature_to_json(&f)).unwrap());
    } else {
        output::print_plain(&format!(
            "renamed feature {}: {} → {}",
            id, old.name, new_name
        ));
    }
    0
}

pub fn set_description(conn: &Connection, id: i64, desc: &str, json: bool) -> i32 {
    let f = match load(conn, id) {
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE features SET description=?1, last_worked_on=?2 WHERE id=?3",
        params![desc, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|f| feature_to_json(&f)).unwrap());
    } else {
        output::print_plain(&format!("updated description for feature: {}", f.name));
    }
    0
}

pub fn set_details(conn: &Connection, id: i64, details: &str, json: bool) -> i32 {
    let f = match load(conn, id) {
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    conn.execute(
        "UPDATE features SET details=?1, last_worked_on=?2 WHERE id=?3",
        params![details, now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|f| feature_to_json(&f)).unwrap());
    } else {
        output::print_plain(&format!("updated details for feature: {}", f.name));
    }
    0
}

pub fn transition(conn: &Connection, id: i64, to_state: &State, json: bool) -> i32 {
    let f = match load(conn, id) {
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    match validate_transition(&f.state, to_state) {
        Err(e) => {
            output::err(&format!("{} for feature {}", e, f.name));
            return 1;
        }
        Ok(false) => return 0,
        Ok(true) => {}
    }
    let from = f.state.clone();
    conn.execute(
        "UPDATE features SET state=?1, last_worked_on=?2 WHERE id=?3",
        params![to_state.to_string(), now(), id],
    )
    .unwrap();
    if json {
        output::print_json(&load(conn, id).map(|f| feature_to_json(&f)).unwrap());
    } else {
        output::print_plain(&format!("feature {}: {} → {}", f.name, from, to_state));
    }
    0
}

pub fn remove(conn: &Connection, id: i64, cascade: bool, json: bool) -> i32 {
    let f = match load(conn, id) {
        Ok(f) => f,
        Err(e) => {
            output::err(&e);
            return 1;
        }
    };
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM tasks WHERE feature_id=?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    if count > 0 && !cascade {
        output::err(&format!(
            "feature has {} task(s); pass --cascade to remove them",
            count
        ));
        return 1;
    }
    conn.execute("DELETE FROM features WHERE id=?1", params![id])
        .unwrap();
    if json {
        output::print_json(&json!({"deleted": true, "id": id}));
    } else {
        output::print_plain(&format!("removed feature {}: {}", id, f.name));
    }
    0
}
