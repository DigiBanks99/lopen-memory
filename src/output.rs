use serde_json::Value;

pub fn print_plain(s: &str) {
    println!("{}", s);
}

pub fn print_json(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap_or_default());
}

pub fn err(msg: &str) {
    eprintln!("error: {}", msg);
}

/// Format a labelled field line, padding the label to align values.
pub fn field(label: &str, value: &str) -> String {
    format!("{:<16}{}", format!("{}:", label), value)
}

/// Indent multi-line content for show views.
pub fn indent_content(content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }
    content
        .lines()
        .map(|l| format!("  {}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
