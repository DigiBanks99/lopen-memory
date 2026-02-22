use crate::output;
use std::fs;
use std::path::PathBuf;

/// The SKILL.md content is embedded into the binary at compile time.
/// The path `../skill/SKILL.md` is relative to src/skill.rs, i.e. it
/// resolves to `skill/SKILL.md` at the repository root.
const SKILL_CONTENT: &str = include_str!("../skill/SKILL.md");

const DEFAULT_SKILLS_DIR: &str = "/.agents/skills";

fn skills_dir(override_path: Option<&String>) -> PathBuf {
    if let Some(p) = override_path {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("AGENTS_SKILLS_DIR") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(format!("{}{}", home, DEFAULT_SKILLS_DIR))
}

pub fn install(override_path: Option<&String>, json: bool) -> i32 {
    let base = skills_dir(override_path);
    let skill_dir = base.join("lopen-memory");

    if let Err(e) = fs::create_dir_all(&skill_dir) {
        output::err(&format!(
            "failed to create directory {}: {}",
            skill_dir.display(),
            e
        ));
        return 2;
    }

    let dest = skill_dir.join("SKILL.md");

    if let Err(e) = fs::write(&dest, SKILL_CONTENT) {
        output::err(&format!("failed to write {}: {}", dest.display(), e));
        return 2;
    }

    if json {
        crate::output::print_json(&serde_json::json!({
            "installed": true,
            "path": dest.display().to_string()
        }));
    } else {
        output::print_plain(&format!("skill installed: {}", dest.display()));
    }

    0
}
