use std::process::Command as StdCommand;

use anyhow::{bail, Context, Result};

pub const NOT_IN_REPO_HINT: &str =
    "not in a git repository - run 'sgit init' or cd into a repo first";
pub const NO_STAGED_HINT: &str = "nothing to commit - use 'sgit stage' to stage changes first";

pub fn run_git(args: &[&str]) -> Result<()> {
    let output = StdCommand::new("git")
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to execute git {} - is git installed?",
                args.join(" ")
            )
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            print!("{}", stdout);
        }
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = suggest_hint_for_git_error(&stderr, args);
        bail!(
            "git {} failed:{}{}",
            args.join(" "),
            format_stderr(&stderr),
            hint
        );
    }
}

pub fn run_git_quiet(args: &[&str]) -> Result<()> {
    let output = StdCommand::new("git")
        .args(args)
        .output()
        .with_context(|| {
            format!(
                "failed to execute git {} - is git installed?",
                args.join(" ")
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = suggest_hint_for_git_error(&stderr, args);
        bail!(
            "git {} failed:{}{}",
            args.join(" "),
            format_stderr(&stderr),
            hint
        );
    }
}

pub fn run_git_silent(args: &[&str]) -> Result<()> {
    let output = StdCommand::new("git")
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "failed to execute git {} - is git installed?",
                args.join(" ")
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = suggest_hint_for_git_error(&stderr, args);
        bail!(
            "git {} failed:{}{}",
            args.join(" "),
            format_stderr(&stderr),
            hint
        );
    }
}

pub fn run_git_in_dir_silent(args: &[&str], dir: &str) -> Result<()> {
    let output = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "failed to execute git {} in {} - is git installed?",
                args.join(" "),
                dir
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let hint = suggest_hint_for_git_error(&stderr, args);
        bail!(
            "git {} failed:{}{}",
            args.join(" "),
            format_stderr(&stderr),
            hint
        );
    }
}

pub fn check_in_repo() -> Result<()> {
    StdCommand::new("git")
        .args(["rev-parse", "--git-dir"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to execute git - is git installed?")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("{}", NOT_IN_REPO_HINT))
}

fn format_stderr(stderr: &str) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("\n  {}", trimmed)
    }
}

fn suggest_hint_for_git_error(stderr: &str, args: &[&str]) -> String {
    let stderr_lower = stderr.to_lowercase();
    let cmd = args.first().copied().unwrap_or("");

    if stderr_lower.contains("not a git repository") {
        return format!("\n  hint: {}", NOT_IN_REPO_HINT);
    }

    if cmd == "commit"
        && (stderr_lower.contains("nothing to commit")
            || stderr_lower.contains("no changes added to commit")
            || stderr_lower.contains("nothing added to commit"))
    {
        return format!("\n  hint: {}", NO_STAGED_HINT);
    }

    if cmd == "push" {
        if stderr_lower.contains("no upstream branch") {
            return "\n  hint: set upstream with 'git push -u origin <branch>' or use 'sgit push' from a tracked branch".to_string();
        }
        if stderr_lower.contains("rejected") {
            return "\n  hint: remote has new commits - try 'sgit pull' first, then push again"
                .to_string();
        }
        if stderr_lower.contains("could not resolve host") || stderr_lower.contains("network") {
            return "\n  hint: check your network connection".to_string();
        }
    }

    if cmd == "pull" {
        if stderr_lower.contains("there is no tracking information") {
            return "\n  hint: branch has no upstream - try 'git branch --set-upstream-to=origin/<branch>'".to_string();
        }
        if stderr_lower.contains("conflict") {
            return "\n  hint: resolve merge conflicts, then commit the resolution".to_string();
        }
    }

    if cmd == "checkout" || cmd == "switch" {
        if stderr_lower.contains("would be overwritten") {
            return "\n  hint: commit or stash your changes before switching branches".to_string();
        }
        if stderr_lower.contains("did not match") {
            return "\n  hint: branch name may be misspelled - check 'sgit branch' for available branches".to_string();
        }
    }

    if cmd == "branch" && stderr_lower.contains("already exists") {
        return "\n  hint: branch name already in use, choose a different name".to_string();
    }

    if stderr_lower.contains("permission denied") {
        return "\n  hint: check file permissions or run with appropriate privileges".to_string();
    }

    String::new()
}
