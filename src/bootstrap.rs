use std::{path::Path, process::Command};

use anyhow::{Context, Result};

use crate::config::StarterConfig;

#[derive(Debug)]
pub struct BootstrapReport {
    pub git_initialized: bool,
    pub initial_commit_created: bool,
    pub github_repo_created: bool,
    pub remote_pushed: bool,
}

pub fn run(project_dir: &Path, config: &StarterConfig) -> Result<BootstrapReport> {
    let mut report = BootstrapReport {
        git_initialized: false,
        initial_commit_created: false,
        github_repo_created: false,
        remote_pushed: false,
    };

    if config.bootstrap.init_git && !project_dir.join(".git").exists() {
        run_git(project_dir, &["init", "-b", "main"])?;
        report.git_initialized = true;
    }
    if config.bootstrap.initial_commit
        && project_dir.join(".git").exists()
        && has_uncommitted_files(project_dir)?
    {
        run_git(project_dir, &["add", "."])?;
        run_git_with_identity(
            project_dir,
            &["commit", "-m", "chore: initialize project scaffold"],
        )?;
        report.initial_commit_created = true;
    }
    if config.github.enabled && config.github.create_repo {
        if try_create_github_repo(project_dir, config)? {
            report.github_repo_created = true;
            if config.github.push_after_create {
                if try_push_main(project_dir)? {
                    report.remote_pushed = true;
                }
            }
        }
    }

    Ok(report)
}

fn has_uncommitted_files(project_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_dir)
        .output()
        .context("failed to inspect git status")?;
    Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn run_git(project_dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(project_dir)
        .status()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("git {} failed with {}", args.join(" "), status)
    }
}

fn run_git_with_identity(project_dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(["-c", "user.name=revi", "-c", "user.email=revi@local"])
        .args(args)
        .current_dir(project_dir)
        .status()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("git {} failed with {}", args.join(" "), status)
    }
}

fn try_create_github_repo(project_dir: &Path, config: &StarterConfig) -> Result<bool> {
    let repo = config
        .github
        .repo
        .as_deref()
        .unwrap_or(&config.project.slug);
    let target = match &config.github.owner {
        Some(owner) => format!("{owner}/{repo}"),
        None => repo.to_string(),
    };
    let status = Command::new("gh")
        .args([
            "repo",
            "create",
            &target,
            "--source=.",
            "--private",
            "--push=false",
        ])
        .current_dir(project_dir)
        .status();
    match status {
        Ok(code) if code.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}

fn try_push_main(project_dir: &Path) -> Result<bool> {
    let status = Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(project_dir)
        .status();
    match status {
        Ok(code) if code.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}
