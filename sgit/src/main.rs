use std::process::{Command as StdCommand, Stdio};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SgitCommand::Init => run_git(&["init"])?,
        SgitCommand::Stage { targets } => stage_targets(&targets)?,
        SgitCommand::Unstage { targets } => restore_stage(&targets)?,
        SgitCommand::Status => run_git(&["status"])?,
        SgitCommand::Log { short } => {
            if short {
                run_git(&["log", "--oneline", "--decorate", "-n", "20"])?;
            } else {
                run_git(&["log", "--decorate", "-n", "40"])?;
            }
        }
        SgitCommand::Diff { path, staged } => {
            if staged {
                run_git(&["diff", "--staged"])?;
            } else if let Some(path) = path {
                run_git(&["diff", path.as_str()])?;
            } else {
                run_git(&["diff"])?;
            }
        }
        SgitCommand::Branch => run_git(&["branch"])?,
        SgitCommand::Push { remote, branch } => {
            let remote = remote.unwrap_or_else(|| "origin".to_string());
            let branch =
                branch.unwrap_or_else(|| current_branch().unwrap_or_else(|_| "HEAD".to_string()));
            run_git(&["push", remote.as_str(), branch.as_str()])?;
        }
        SgitCommand::Pull { remote, branch } => {
            let mut args_owned = vec!["pull".to_string()];
            if let Some(remote) = remote {
                args_owned.push(remote);
                if let Some(branch) = branch {
                    args_owned.push(branch);
                }
            }

            let args_refs: Vec<&str> = args_owned.iter().map(String::as_str).collect();
            run_git(&args_refs)?;
        }
        SgitCommand::Commit {
            message,
            all,
            staged,
            unstaged,
            push,
            amend,
            no_verify,
        } => {
            let mut should_stage_untracked = false;
            if all {
                run_git(&["add", "-A"])?;
                should_stage_untracked = true;
            } else if unstaged {
                run_git(&["add", "-u"])?;
            }

            if !all && !staged && !unstaged {
                // default to staged only when no scope provided
            } else if staged && (all || unstaged) {
                bail!("cannot combine --staged with --all or --unstaged");
            }

            let mut commit_args = vec!["commit"];
            if amend {
                commit_args.push("--amend");
            }
            if no_verify {
                commit_args.push("--no-verify");
            }
            commit_args.push("-m");
            commit_args.push(message.as_str());

            run_git(&commit_args)?;

            if push {
                let remote = "origin";
                let branch = current_branch().unwrap_or_else(|_| "HEAD".to_string());
                run_git(&["push", remote, branch.as_str()])?;
            }

            if should_stage_untracked {
                println!("All tracked and untracked files staged, commit complete.");
            }
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(
    name = "sgit",
    about = "Blazing fast wrapper for Git with simplified workflows",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: SgitCommand,
}

#[derive(Subcommand)]
enum SgitCommand {
    /// Initialize a new Git repository
    Init,
    /// Stage files (defaults to `.`)
    Stage {
        #[arg(value_name = "PATH", default_value = ".")]
        targets: Vec<String>,
    },
    /// Unstage files or reset staged changes
    Unstage {
        #[arg(value_name = "PATH", default_value = ".")]
        targets: Vec<String>,
    },
    /// Show the current status
    Status,
    /// Commit with a simple interface
    Commit {
        /// Commit message
        #[arg(short, long, value_name = "MSG")]
        message: String,
        /// Stage tracked + untracked before committing
        #[arg(long)]
        all: bool,
        /// Commit only what is already staged (default)
        #[arg(long)]
        staged: bool,
        /// Stage tracked unstaged files before committing
        #[arg(long)]
        unstaged: bool,
        /// Push immediately after committing
        #[arg(long)]
        push: bool,
        /// Amend the previous commit
        #[arg(long)]
        amend: bool,
        /// Skip pre-commit hooks
        #[arg(long)]
        no_verify: bool,
    },
    /// Show recent commits
    Log {
        /// Use a compact log
        #[arg(long)]
        short: bool,
    },
    /// Show git diff
    Diff {
        /// Limit diff to a specific path
        path: Option<String>,
        /// Show staged diff
        #[arg(long)]
        staged: bool,
    },
    /// List branches
    Branch,
    /// Push current branch
    Push {
        /// Remote name (defaults to origin)
        remote: Option<String>,
        /// Branch name
        branch: Option<String>,
    },
    /// Fetch and merge from remote
    Pull {
        /// Remote name
        remote: Option<String>,
        /// Branch name
        branch: Option<String>,
    },
}

fn stage_targets(targets: &[String]) -> Result<()> {
    let target_args: Vec<&str> = if targets.is_empty() {
        vec!["."]
    } else {
        targets.iter().map(String::as_str).collect()
    };

    let mut args = Vec::with_capacity(1 + target_args.len());
    args.push("add");
    args.extend(target_args);

    run_git(&args)
}

fn restore_stage(targets: &[String]) -> Result<()> {
    let target_args: Vec<&str> = if targets.is_empty() {
        vec!["."]
    } else {
        targets.iter().map(String::as_str).collect()
    };

    let mut args = Vec::with_capacity(2 + target_args.len());
    args.push("restore");
    args.push("--staged");
    args.extend(target_args);

    run_git(&args)
}

fn current_branch() -> Result<String> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .stderr(Stdio::inherit())
        .output()
        .context("failed to query current branch")?;

    if !output.status.success() {
        Err(anyhow!("unable to determine current branch"))
    } else {
        let branch = String::from_utf8(output.stdout)
            .context("branch name is not valid UTF-8")?
            .trim()
            .to_string();
        Ok(branch)
    }
}

fn run_git(args: &[&str]) -> Result<()> {
    let status = StdCommand::new("git")
        .args(args)
        .status()
        .with_context(|| format!("running git {}", args.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        bail!("git {} failed with {}", args.join(" "), status);
    }
}
