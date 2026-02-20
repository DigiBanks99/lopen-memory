use rusqlite::{Connection, params};
use serde_json::{json, Value};
use crate::output;

pub struct Project {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub description: String,
    pub completed: bool,
    pub updated_at: String,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load(conn: &Connection, id: i64) -> Result<Project, String> {
    conn.query_row(
        "SELECT id, name, path, description, completed, updated_at FROM projects WHERE id=?1",
        params![id],
        |r| Ok(Project {
            id: r.get(0)?,
            name: r.get(1)?,
            path: r.get(2)?,
            description: r.get(3)?,
            completed: r.get::<_, i64>(4)? != 0,
            updated_at: r.get(5)?,
        }),
    )
    .map_err(|_| format!("project not found: {}", id))
}

pub fn add(conn: &Connection, name: &str, path: &str, description: &str, json: bool) -> i32 {
    let name = name.trim();
    if name.is_empty() { output::err("name must not be empty"); return 1; }
    let ts = now();
    match conn.execute(
        "INSERT INTO projects (name, path, description, updated_at) VALUES (?1,?2,?3,?4)",
        params![name, path, description, ts],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            if json {
                let p = load(conn, id).unwrap();
                output::print_json(&project_to_json(&p));
            } else {
                output::print_plain(&format!("added project {}: {}", id, name));
            }
            0
        }
        Err(e) => { output::err(&e.to_string()); 2 }
    }
}

pub fn list(conn: &Connection, filter: Option<bool>, json: bool) -> i32 {
    let sql = match filter {
        Some(true)  => "SELECT id, name, path, description, completed, updated_at FROM projects WHERE completed=1 ORDER BY id",
        Some(false) => "SELECT id, name, path, description, completed, updated_at FROM projects WHERE completed=0 ORDER BY id",
        None        => "SELECT id, name, path, description, completed, updated_at FROM projects ORDER BY id",
    };
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(e) => { output::err(&e.to_string()); return 2; }
    };
    let projects: Vec<Project> = stmt
        .query_map([], |r| Ok(Project {
            id: r.get(0)?,
            name: r.get(1)?,
            path: r.get(2)?,
            description: r.get(3)?,
            completed: r.get::<_, i64>(4)? != 0,
            updated_at: r.get(5)?,
        }))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    if projects.is_empty() {
        output::print_plain("no projects found");
        return 0;
    }
    if json {
        output::print_json(&Value::Array(projects.iter().map(project_to_json).collect()));
    } else {
        for p in &projects {
            let status = if p.completed { "complete" } else { "incomplete" };
            println!("{:<4} {:<20} {:<40} {}", p.id, p.name, p.path, status);
        }
    }
    0
}

pub fn show(conn: &Connection, id: i64, json: bool) -> i32 {
    let p = match load(conn, id) {
        Ok(p) => p,
        Err(e) => { output::err(&e); return 1; }
    };

    // Load modules
    let mut mstmt = conn.prepare(
        "SELECT id, name, state FROM modules WHERE project_id=?1 ORDER BY id"
    ).unwrap();
    let modules: Vec<(i64, String, String)> = mstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    // Load linked research
    let mut rstmt = conn.prepare(
        "SELECT r.id, r.name, r.description FROM research r
         JOIN research_projects rp ON rp.research_id=r.id
         WHERE rp.project_id=?1 ORDER BY r.id"
    ).unwrap();
    let research: Vec<(i64, String, String)> = rstmt
        .query_map(params![id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    if json {
        let mut v = project_to_json(&p);
        v["modules"] = Value::Array(modules.iter().map(|(id, name, state)| {
            json!({"id": id, "name": name, "state": state})
        }).collect());
        v["research"] = Value::Array(research.iter().map(|(id, name, desc)| {
            json!({"id": id, "name": name, "description": desc})
        }).collect());
        output::print_json(&v);
    } else {
        println!("{}", output::field("id", &p.id.to_string()));
        println!("{}", output::field("name", &p.name));
        println!("{}", output::field("path", &p.path));
        println!("{}", output::field("description", &p.description));
        println!("{}", output::field("completed", if p.completed { "true" } else { "false" }));
        println!("{}", output::field("updated_at", &p.updated_at));
        if !modules.is_empty() {
            println!();
            println!("modules:");
            for (mid, mname, mstate) in &modules {
                println!("  {:<4} {:<20} {}", mid, mname, mstate);
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
    if new_name.is_empty() { output::err("name must not be empty"); return 1; }
    let old = match load(conn, id) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
    match conn.execute("UPDATE projects SET name=?1, updated_at=?2 WHERE id=?3",
        params![new_name, now(), id]) {
        Ok(_) => {
            if json { output::print_json(&load(conn, id).map(|p| project_to_json(&p)).unwrap()); }
            else { output::print_plain(&format!("renamed project {}: {} → {}", id, old.name, new_name)); }
            0
        }
        Err(e) => { output::err(&e.to_string()); 2 }
    }
}

pub fn set_description(conn: &Connection, id: i64, desc: &str, json: bool) -> i32 {
    match load(conn, id) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
    conn.execute("UPDATE projects SET description=?1, updated_at=?2 WHERE id=?3",
        params![desc, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|p| project_to_json(&p)).unwrap()); }
    else { output::print_plain(&format!("updated description for project: {}", load(conn, id).unwrap().name)); }
    0
}

pub fn set_path(conn: &Connection, id: i64, path: &str, json: bool) -> i32 {
    match load(conn, id) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
    conn.execute("UPDATE projects SET path=?1, updated_at=?2 WHERE id=?3",
        params![path, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|p| project_to_json(&p)).unwrap()); }
    else { output::print_plain(&format!("updated path for project: {}", load(conn, id).unwrap().name)); }
    0
}

pub fn set_completed(conn: &Connection, id: i64, completed: bool, json: bool) -> i32 {
    let p = match load(conn, id) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
    conn.execute("UPDATE projects SET completed=?1, updated_at=?2 WHERE id=?3",
        params![completed as i64, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|p| project_to_json(&p)).unwrap()); }
    else {
        let verb = if completed { "marked complete" } else { "reopened" };
        output::print_plain(&format!("project {} {}", p.name, verb));
    }
    0
}

pub fn remove(conn: &Connection, id: i64, cascade: bool, json: bool) -> i32 {
    let p = match load(conn, id) { Ok(p) => p, Err(e) => { output::err(&e); return 1; } };
    // Check for modules
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM modules WHERE project_id=?1", params![id], |r| r.get(0)
    ).unwrap_or(0);
    if count > 0 && !cascade {
        output::err(&format!("project has {} module(s); pass --cascade to remove them", count));
        return 1;
    }
    match conn.execute("DELETE FROM projects WHERE id=?1", params![id]) {
        Ok(_) => {
            if json { output::print_json(&json!({"deleted": true, "id": id})); }
            else { output::print_plain(&format!("removed project {}: {}", id, p.name)); }
            0
        }
        Err(e) => { output::err(&e.to_string()); 2 }
    }
}

fn project_to_json(p: &Project) -> Value {
    json!({
        "id": p.id,
        "name": p.name,
        "path": p.path,
        "description": p.description,
        "completed": p.completed,
        "updated_at": p.updated_at,
    })
}
