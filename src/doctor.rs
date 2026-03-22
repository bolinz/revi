use std::process::Command;

pub fn run() -> String {
    let checks = [
        ("git", &["--version"][..]),
        ("gh", &["--version"][..]),
        ("python3", &["--version"][..]),
        ("node", &["--version"][..]),
        ("npm", &["--version"][..]),
        ("cargo", &["--version"][..]),
    ];
    let mut out = String::new();
    for (tool, args) in checks {
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
            Ok(output) => out.push_str(&format!("[warn] {tool}: exited with {}\n", output.status)),
            Err(_) => out.push_str(&format!("[warn] {tool}: not found\n")),
        }
    }
    out.trim_end().to_string()
}
