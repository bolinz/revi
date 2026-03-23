use std::process::Command;

pub fn run() -> String {
    let mut out = String::new();
    for (tool, args) in [
        ("git", &["--version"][..]),
        ("gh", &["--version"][..]),
        ("python3", &["--version"][..]),
        ("node", &["--version"][..]),
        ("npm", &["--version"][..]),
        ("cargo", &["--version"][..]),
    ] {
        let status = Command::new(tool).args(args).output();
        match status {
            Ok(output) if output.status.success() => {
                let line = String::from_utf8_lossy(&output.stdout);
                let fallback = String::from_utf8_lossy(&output.stderr);
                let text = if line.trim().is_empty() {
                    fallback.trim()
                } else {
                    line.trim()
                };
                out.push_str(&format!("[ok] {tool}: {text}\n"));
            }
            Ok(output) => out.push_str(&format!(
                "[warn] {tool}: exited with {}{}\n",
                output.status,
                optional_hint(tool)
            )),
            Err(_) => out.push_str(&format!("[warn] {tool}: not found{}\n", optional_hint(tool))),
        }
    }
    for tool in ["codex", "claude", "gemini"] {
        if tool_in_path(tool) {
            out.push_str(&format!(
                "[ok] {tool}: found in PATH (optional AI coding CLI)\n"
            ));
        } else {
            out.push_str(&format!(
                "[warn] {tool}: not found (optional AI coding CLI; scaffold generation still works)\n"
            ));
        }
    }
    out.trim_end().to_string()
}

fn optional_hint(_tool: &str) -> &'static str {
    ""
}

fn tool_in_path(tool: &str) -> bool {
    Command::new("sh")
        .args(["-lc", &format!("command -v {tool}")])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
