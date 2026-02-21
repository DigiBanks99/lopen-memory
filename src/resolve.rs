use rusqlite::{params, Connection, Result};

fn is_id(s: &str) -> bool {
    s.parse::<i64>().is_ok()
}

pub fn resolve_project(conn: &Connection, name_or_id: &str) -> Result<i64, String> {
    if is_id(name_or_id) {
        let id: i64 = name_or_id.parse().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE id=?1",
                params![id],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            Ok(id)
        } else {
            Err(format!("project not found: {}", name_or_id))
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT id FROM projects WHERE name=?1")
            .map_err(|e| e.to_string())?;
        let ids: Vec<i64> = stmt
            .query_map(params![name_or_id], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        match ids.len() {
            0 => Err(format!("project not found: {}", name_or_id)),
            1 => Ok(ids[0]),
            _ => Err(format!("project name '{}' is ambiguous", name_or_id)),
        }
    }
}

pub fn resolve_module(
    conn: &Connection,
    name_or_id: &str,
    project_id: Option<i64>,
) -> Result<i64, String> {
    if is_id(name_or_id) {
        let id: i64 = name_or_id.parse().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM modules WHERE id=?1",
                params![id],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            Ok(id)
        } else {
            Err(format!("module not found: {}", name_or_id))
        }
    } else {
        let ids: Vec<i64> = if let Some(pid) = project_id {
            let mut stmt = conn
                .prepare("SELECT id FROM modules WHERE name=?1 AND project_id=?2")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id, pid], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        } else {
            let mut stmt = conn
                .prepare("SELECT id FROM modules WHERE name=?1")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        };
        match ids.len() {
            0 => Err(format!("module not found: {}", name_or_id)),
            1 => Ok(ids[0]),
            _ => Err(format!(
                "module name '{}' is ambiguous; specify --project to narrow scope",
                name_or_id
            )),
        }
    }
}

pub fn resolve_feature(
    conn: &Connection,
    name_or_id: &str,
    module_id: Option<i64>,
) -> Result<i64, String> {
    if is_id(name_or_id) {
        let id: i64 = name_or_id.parse().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM features WHERE id=?1",
                params![id],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            Ok(id)
        } else {
            Err(format!("feature not found: {}", name_or_id))
        }
    } else {
        let ids: Vec<i64> = if let Some(mid) = module_id {
            let mut stmt = conn
                .prepare("SELECT id FROM features WHERE name=?1 AND module_id=?2")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id, mid], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        } else {
            let mut stmt = conn
                .prepare("SELECT id FROM features WHERE name=?1")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        };
        match ids.len() {
            0 => Err(format!("feature not found: {}", name_or_id)),
            1 => Ok(ids[0]),
            _ => Err(format!(
                "feature name '{}' is ambiguous; specify --module to narrow scope",
                name_or_id
            )),
        }
    }
}

pub fn resolve_task(
    conn: &Connection,
    name_or_id: &str,
    feature_id: Option<i64>,
) -> Result<i64, String> {
    if is_id(name_or_id) {
        let id: i64 = name_or_id.parse().unwrap();
        let exists: bool = conn
            .query_row("SELECT COUNT(*) FROM tasks WHERE id=?1", params![id], |r| {
                r.get::<_, i64>(0)
            })
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            Ok(id)
        } else {
            Err(format!("task not found: {}", name_or_id))
        }
    } else {
        let ids: Vec<i64> = if let Some(fid) = feature_id {
            let mut stmt = conn
                .prepare("SELECT id FROM tasks WHERE name=?1 AND feature_id=?2")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id, fid], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        } else {
            let mut stmt = conn
                .prepare("SELECT id FROM tasks WHERE name=?1")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map(params![name_or_id], |r| r.get(0))
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();
            x
        };
        match ids.len() {
            0 => Err(format!("task not found: {}", name_or_id)),
            1 => Ok(ids[0]),
            _ => Err(format!(
                "task name '{}' is ambiguous; specify --feature to narrow scope",
                name_or_id
            )),
        }
    }
}

pub fn resolve_research(conn: &Connection, name_or_id: &str) -> Result<i64, String> {
    if is_id(name_or_id) {
        let id: i64 = name_or_id.parse().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM research WHERE id=?1",
                params![id],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);
        if exists {
            Ok(id)
        } else {
            Err(format!("research not found: {}", name_or_id))
        }
    } else {
        conn.query_row(
            "SELECT id FROM research WHERE name=?1",
            params![name_or_id],
            |r| r.get(0),
        )
        .map_err(|_| format!("research not found: {}", name_or_id))
    }
}
