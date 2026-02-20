use rusqlite::{Connection, params};
use serde_json::{json, Value};
use crate::output;

pub struct Research {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub content: String,
    pub source: String,
    pub researched_at: String,
    pub created_at: String,
    pub updated_at: String,
}

fn now() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load(conn: &Connection, id: i64) -> Result<Research, String> {
    conn.query_row(
        "SELECT id, name, description, content, source, researched_at, created_at, updated_at FROM research WHERE id=?1",
        params![id],
        |r| Ok(Research {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            content: r.get(3)?, source: r.get(4)?, researched_at: r.get(5)?,
            created_at: r.get(6)?, updated_at: r.get(7)?,
        }),
    )
    .map_err(|_| format!("research not found: {}", id))
}

fn research_to_json(r: &Research) -> Value {
    json!({
        "id": r.id, "name": r.name, "description": r.description,
        "content": r.content, "source": r.source,
        "researched_at": r.researched_at,
        "created_at": r.created_at, "updated_at": r.updated_at,
    })
}

fn parse_date(s: &str) -> Result<String, String> {
    // Accept YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ
    if s.len() == 10 {
        // validate rough format
        if s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            return Ok(format!("{}T00:00:00Z", s));
        }
    } else if s.len() == 20 && s.ends_with('Z') {
        return Ok(s.to_string());
    }
    Err(format!("invalid date format '{}'; use YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ", s))
}

pub fn add(conn: &Connection, name: &str, description: &str, json: bool) -> i32 {
    let name = name.trim();
    if name.is_empty() { output::err("name must not be empty"); return 1; }
    let ts = now();
    match conn.execute(
        "INSERT INTO research (name, description, researched_at, created_at, updated_at) VALUES (?1,?2,?3,?3,?3)",
        params![name, description, ts],
    ) {
        Ok(_) => {
            let id = conn.last_insert_rowid();
            if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
            else { output::print_plain(&format!("added research {}: {}", id, name)); }
            0
        }
        Err(e) => { output::err(&e.to_string()); 2 }
    }
}

pub fn list(conn: &Connection, stale_days: Option<i64>, json: bool) -> i32 {
    let records: Vec<Research> = if let Some(days) = stale_days {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, source, researched_at, created_at, updated_at
             FROM research
             WHERE researched_at < datetime('now', ?1)
             ORDER BY id"
        ).unwrap();
        let cutoff = format!("-{} days", days);
        stmt.query_map(params![cutoff], |r| Ok(Research {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            content: r.get(3)?, source: r.get(4)?, researched_at: r.get(5)?,
            created_at: r.get(6)?, updated_at: r.get(7)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, source, researched_at, created_at, updated_at FROM research ORDER BY id"
        ).unwrap();
        stmt.query_map([], |r| Ok(Research {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            content: r.get(3)?, source: r.get(4)?, researched_at: r.get(5)?,
            created_at: r.get(6)?, updated_at: r.get(7)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    };

    if records.is_empty() { output::print_plain("no research found"); return 0; }
    if json {
        output::print_json(&Value::Array(records.iter().map(research_to_json).collect()));
    } else {
        for r in &records {
            let date = &r.researched_at[..10];
            println!("{:<4} {:<24} {}  {}", r.id, r.name, date, r.description);
        }
    }
    0
}

pub fn show(conn: &Connection, id: i64, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };

    // Gather all links
    struct Link { kind: String, entity_id: i64, name: String, context: String }
    let mut links: Vec<Link> = Vec::new();

    // projects
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name FROM projects p JOIN research_projects rp ON rp.project_id=p.id WHERE rp.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((row.get::<_,i64>(0)?, row.get::<_,String>(1)?))).unwrap().filter_map(|r| r.ok()) {
        links.push(Link { kind: "project".into(), entity_id: row.0, name: row.1, context: String::new() });
    }

    // modules
    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, p.name FROM modules m JOIN research_modules rm ON rm.module_id=m.id JOIN projects p ON p.id=m.project_id WHERE rm.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))).unwrap().filter_map(|r| r.ok()) {
        links.push(Link { kind: "module".into(), entity_id: row.0, name: row.1, context: row.2 });
    }

    // features
    let mut stmt = conn.prepare(
        "SELECT f.id, f.name, m.name, p.name FROM features f
         JOIN research_features rf ON rf.feature_id=f.id
         JOIN modules m ON m.id=f.module_id
         JOIN projects p ON p.id=m.project_id
         WHERE rf.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((
        row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?, row.get::<_,String>(3)?
    ))).unwrap().filter_map(|r| r.ok()) {
        links.push(Link { kind: "feature".into(), entity_id: row.0, name: row.1, context: format!("{} > {}", row.3, row.2) });
    }

    // tasks
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, f.name, m.name, p.name FROM tasks t
         JOIN research_tasks rt ON rt.task_id=t.id
         JOIN features f ON f.id=t.feature_id
         JOIN modules m ON m.id=f.module_id
         JOIN projects p ON p.id=m.project_id
         WHERE rt.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((
        row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?,
        row.get::<_,String>(3)?, row.get::<_,String>(4)?
    ))).unwrap().filter_map(|r| r.ok()) {
        links.push(Link { kind: "task".into(), entity_id: row.0, name: row.1, context: format!("{} > {} > {}", row.4, row.3, row.2) });
    }

    if json {
        let mut v = research_to_json(&r);
        v["linked_to"] = Value::Array(links.iter().map(|l| json!({
            "type": l.kind, "id": l.entity_id, "name": l.name, "context": l.context
        })).collect());
        output::print_json(&v);
    } else {
        println!("{}", output::field("id", &r.id.to_string()));
        println!("{}", output::field("name", &r.name));
        println!("{}", output::field("description", &r.description));
        println!("{}", output::field("source", &r.source));
        println!("{}", output::field("researched_at", &r.researched_at));
        println!("{}", output::field("created_at", &r.created_at));
        println!("{}", output::field("updated_at", &r.updated_at));
        if !r.content.is_empty() {
            println!();
            println!("content:");
            println!("{}", output::indent_content(&r.content));
        }
        if !links.is_empty() {
            println!();
            println!("linked to:");
            for l in &links {
                if l.context.is_empty() {
                    println!("  {:<10} {:<4} {}", l.kind, l.entity_id, l.name);
                } else {
                    println!("  {:<10} {:<4} {:<24} ({})", l.kind, l.entity_id, l.name, l.context);
                }
            }
        }
    }
    0
}

pub fn rename(conn: &Connection, id: i64, new_name: &str, json: bool) -> i32 {
    let new_name = new_name.trim();
    if new_name.is_empty() { output::err("name must not be empty"); return 1; }
    let old = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    match conn.execute("UPDATE research SET name=?1, updated_at=?2 WHERE id=?3",
        params![new_name, now(), id]) {
        Ok(_) => {
            if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
            else { output::print_plain(&format!("renamed research {}: {} → {}", id, old.name, new_name)); }
            0
        }
        Err(e) => { output::err(&e.to_string()); 2 }
    }
}

pub fn set_description(conn: &Connection, id: i64, desc: &str, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    conn.execute("UPDATE research SET description=?1, updated_at=?2 WHERE id=?3",
        params![desc, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
    else { output::print_plain(&format!("updated description for research: {}", r.name)); }
    0
}

pub fn set_content(conn: &Connection, id: i64, content: &str, update_date: bool, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let ts = now();
    if update_date {
        conn.execute("UPDATE research SET content=?1, researched_at=?2, updated_at=?2 WHERE id=?3",
            params![content, ts, id]).unwrap();
    } else {
        conn.execute("UPDATE research SET content=?1, updated_at=?2 WHERE id=?3",
            params![content, ts, id]).unwrap();
    }
    if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
    else { output::print_plain(&format!("updated content for research: {}", r.name)); }
    0
}

pub fn set_source(conn: &Connection, id: i64, source: &str, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    conn.execute("UPDATE research SET source=?1, updated_at=?2 WHERE id=?3",
        params![source, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
    else { output::print_plain(&format!("updated source for research: {}", r.name)); }
    0
}

pub fn set_researched_at(conn: &Connection, id: i64, date_str: &str, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let ts = match parse_date(date_str) {
        Ok(t) => t,
        Err(e) => { output::err(&e); return 1; }
    };
    conn.execute("UPDATE research SET researched_at=?1, updated_at=?2 WHERE id=?3",
        params![ts, now(), id]).unwrap();
    if json { output::print_json(&load(conn, id).map(|r| research_to_json(&r)).unwrap()); }
    else { output::print_plain(&format!("updated researched_at for research: {} → {}", r.name, ts)); }
    0
}

pub fn search(conn: &Connection, term: &str, stale_days: Option<i64>, json: bool) -> i32 {
    let pattern = format!("%{}%", term.to_lowercase());
    let records: Vec<Research> = if let Some(days) = stale_days {
        let cutoff = format!("-{} days", days);
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, source, researched_at, created_at, updated_at
             FROM research
             WHERE (LOWER(name) LIKE ?1 OR LOWER(description) LIKE ?1 OR LOWER(content) LIKE ?1 OR LOWER(source) LIKE ?1)
               AND researched_at < datetime('now', ?2)
             ORDER BY id"
        ).unwrap();
        stmt.query_map(params![pattern, cutoff], |r| Ok(Research {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            content: r.get(3)?, source: r.get(4)?, researched_at: r.get(5)?,
            created_at: r.get(6)?, updated_at: r.get(7)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, name, description, content, source, researched_at, created_at, updated_at
             FROM research
             WHERE LOWER(name) LIKE ?1 OR LOWER(description) LIKE ?1 OR LOWER(content) LIKE ?1 OR LOWER(source) LIKE ?1
             ORDER BY id"
        ).unwrap();
        stmt.query_map(params![pattern], |r| Ok(Research {
            id: r.get(0)?, name: r.get(1)?, description: r.get(2)?,
            content: r.get(3)?, source: r.get(4)?, researched_at: r.get(5)?,
            created_at: r.get(6)?, updated_at: r.get(7)?,
        })).unwrap().filter_map(|r| r.ok()).collect()
    };

    if records.is_empty() {
        output::print_plain(&format!("no research found matching: {}", term));
        return 0;
    }
    if json {
        output::print_json(&Value::Array(records.iter().map(research_to_json).collect()));
    } else {
        for r in &records {
            let date = &r.researched_at[..10];
            println!("{:<4} {:<24} {}  {}", r.id, r.name, date, r.description);
        }
    }
    0
}

pub fn link_project(conn: &Connection, research_id: i64, project_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let pname: String = conn.query_row("SELECT name FROM projects WHERE id=?1", params![project_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("INSERT OR IGNORE INTO research_projects (research_id, project_id) VALUES (?1,?2)",
        params![research_id, project_id]).unwrap();
    if json { output::print_json(&json!({"linked": true, "research": r.name, "project": pname})); }
    else { output::print_plain(&format!("linked research {} → project: {}", r.name, pname)); }
    0
}

pub fn link_module(conn: &Connection, research_id: i64, module_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let mname: String = conn.query_row("SELECT name FROM modules WHERE id=?1", params![module_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("INSERT OR IGNORE INTO research_modules (research_id, module_id) VALUES (?1,?2)",
        params![research_id, module_id]).unwrap();
    if json { output::print_json(&json!({"linked": true, "research": r.name, "module": mname})); }
    else { output::print_plain(&format!("linked research {} → module: {}", r.name, mname)); }
    0
}

pub fn link_feature(conn: &Connection, research_id: i64, feature_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let fname: String = conn.query_row("SELECT name FROM features WHERE id=?1", params![feature_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("INSERT OR IGNORE INTO research_features (research_id, feature_id) VALUES (?1,?2)",
        params![research_id, feature_id]).unwrap();
    if json { output::print_json(&json!({"linked": true, "research": r.name, "feature": fname})); }
    else { output::print_plain(&format!("linked research {} → feature: {}", r.name, fname)); }
    0
}

pub fn link_task(conn: &Connection, research_id: i64, task_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let tname: String = conn.query_row("SELECT name FROM tasks WHERE id=?1", params![task_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("INSERT OR IGNORE INTO research_tasks (research_id, task_id) VALUES (?1,?2)",
        params![research_id, task_id]).unwrap();
    if json { output::print_json(&json!({"linked": true, "research": r.name, "task": tname})); }
    else { output::print_plain(&format!("linked research {} → task: {}", r.name, tname)); }
    0
}

pub fn unlink_project(conn: &Connection, research_id: i64, project_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let pname: String = conn.query_row("SELECT name FROM projects WHERE id=?1", params![project_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("DELETE FROM research_projects WHERE research_id=?1 AND project_id=?2",
        params![research_id, project_id]).unwrap();
    if json { output::print_json(&json!({"unlinked": true, "research": r.name, "project": pname})); }
    else { output::print_plain(&format!("unlinked research {} from project: {}", r.name, pname)); }
    0
}

pub fn unlink_module(conn: &Connection, research_id: i64, module_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let mname: String = conn.query_row("SELECT name FROM modules WHERE id=?1", params![module_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("DELETE FROM research_modules WHERE research_id=?1 AND module_id=?2",
        params![research_id, module_id]).unwrap();
    if json { output::print_json(&json!({"unlinked": true, "research": r.name, "module": mname})); }
    else { output::print_plain(&format!("unlinked research {} from module: {}", r.name, mname)); }
    0
}

pub fn unlink_feature(conn: &Connection, research_id: i64, feature_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let fname: String = conn.query_row("SELECT name FROM features WHERE id=?1", params![feature_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("DELETE FROM research_features WHERE research_id=?1 AND feature_id=?2",
        params![research_id, feature_id]).unwrap();
    if json { output::print_json(&json!({"unlinked": true, "research": r.name, "feature": fname})); }
    else { output::print_plain(&format!("unlinked research {} from feature: {}", r.name, fname)); }
    0
}

pub fn unlink_task(conn: &Connection, research_id: i64, task_id: i64, json: bool) -> i32 {
    let r = match load(conn, research_id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    let tname: String = conn.query_row("SELECT name FROM tasks WHERE id=?1", params![task_id], |r| r.get(0))
        .unwrap_or_default();
    conn.execute("DELETE FROM research_tasks WHERE research_id=?1 AND task_id=?2",
        params![research_id, task_id]).unwrap();
    if json { output::print_json(&json!({"unlinked": true, "research": r.name, "task": tname})); }
    else { output::print_plain(&format!("unlinked research {} from task: {}", r.name, tname)); }
    0
}

pub fn links(conn: &Connection, id: i64, json: bool) -> i32 {
    match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    // Reuse show's logic but only print links section
    // Build link list same as show
    struct Link { kind: String, entity_id: i64, name: String, context: String }
    let mut lnks: Vec<Link> = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT p.id, p.name FROM projects p JOIN research_projects rp ON rp.project_id=p.id WHERE rp.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((row.get::<_,i64>(0)?, row.get::<_,String>(1)?))).unwrap().filter_map(|r| r.ok()) {
        lnks.push(Link { kind: "project".into(), entity_id: row.0, name: row.1, context: String::new() });
    }

    let mut stmt = conn.prepare(
        "SELECT m.id, m.name, p.name FROM modules m JOIN research_modules rm ON rm.module_id=m.id JOIN projects p ON p.id=m.project_id WHERE rm.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))).unwrap().filter_map(|r| r.ok()) {
        lnks.push(Link { kind: "module".into(), entity_id: row.0, name: row.1, context: row.2 });
    }

    let mut stmt = conn.prepare(
        "SELECT f.id, f.name, m.name, p.name FROM features f
         JOIN research_features rf ON rf.feature_id=f.id
         JOIN modules m ON m.id=f.module_id
         JOIN projects p ON p.id=m.project_id
         WHERE rf.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((
        row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?, row.get::<_,String>(3)?
    ))).unwrap().filter_map(|r| r.ok()) {
        lnks.push(Link { kind: "feature".into(), entity_id: row.0, name: row.1, context: format!("{} > {}", row.3, row.2) });
    }

    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, f.name, m.name, p.name FROM tasks t
         JOIN research_tasks rt ON rt.task_id=t.id
         JOIN features f ON f.id=t.feature_id
         JOIN modules m ON m.id=f.module_id
         JOIN projects p ON p.id=m.project_id
         WHERE rt.research_id=?1"
    ).unwrap();
    for row in stmt.query_map(params![id], |row| Ok((
        row.get::<_,i64>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?,
        row.get::<_,String>(3)?, row.get::<_,String>(4)?
    ))).unwrap().filter_map(|r| r.ok()) {
        lnks.push(Link { kind: "task".into(), entity_id: row.0, name: row.1, context: format!("{} > {} > {}", row.4, row.3, row.2) });
    }

    if lnks.is_empty() {
        output::print_plain("no links found for this research");
        return 0;
    }
    if json {
        output::print_json(&Value::Array(lnks.iter().map(|l| json!({
            "type": l.kind, "id": l.entity_id, "name": l.name, "context": l.context
        })).collect()));
    } else {
        for l in &lnks {
            if l.context.is_empty() {
                println!("  {:<10} {:<4} {}", l.kind, l.entity_id, l.name);
            } else {
                println!("  {:<10} {:<4} {:<24} ({})", l.kind, l.entity_id, l.name, l.context);
            }
        }
    }
    0
}

pub fn remove(conn: &Connection, id: i64, json: bool) -> i32 {
    let r = match load(conn, id) { Ok(r) => r, Err(e) => { output::err(&e); return 1; } };
    conn.execute("DELETE FROM research WHERE id=?1", params![id]).unwrap();
    if json { output::print_json(&json!({"deleted": true, "id": id})); }
    else { output::print_plain(&format!("removed research {}: {}", id, r.name)); }
    0
}
