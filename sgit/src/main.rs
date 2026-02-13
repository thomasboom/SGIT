use std::process::Command as StdCommand;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{Input, Select};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.explain {
        print_explanations();
        return Ok(());
    }

    let command = match cli.command {
        Some(command) => command,
        None => bail!("'sgit' requires a subcommand; use --help to see the available list"),
    };

    match command {
        SgitCommand::Init => run_git(&["init"])?,
        SgitCommand::Stage { targets } => stage_targets(&targets)?,
        SgitCommand::Unstage { targets } => restore_stage(&targets)?,
        SgitCommand::Status { short } => {
            if short {
                run_git(&["status", "-sb"])?;
            } else {
                run_git(&["status"])?;
            }
        }
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
            if remote.is_none() && branch.is_some() {
                bail!("cannot specify --branch without --remote");
            }

            let mut args_owned = vec!["push".to_string()];
            if let Some(remote) = remote {
                args_owned.push(remote);
                if let Some(branch) = branch {
                    args_owned.push(branch);
                }
            }

            let args_refs: Vec<&str> = args_owned.iter().map(String::as_str).collect();
            run_git(&args_refs)?;
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
            let (all, staged, unstaged, commit_msg) =
                if message.is_none() && !all && !staged && !unstaged {
                    let scope = Select::new()
                        .with_prompt("What would you like to commit?")
                        .items(&[
                            "Staged changes",
                            "Unstaged changes",
                            "All changes",
                            "Custom",
                        ])
                        .default(0)
                        .interact()?;

                    let (all, staged, unstaged) = match scope {
                        0 => (false, true, false),
                        1 => (false, false, true),
                        2 => (true, false, false),
                        _ => (false, false, false),
                    };

                    let msg: String = Input::new().with_prompt("Commit message").interact()?;
                    (all, staged, unstaged, msg)
                } else {
                    let msg = message.unwrap_or_default();
                    (all, staged, unstaged, msg)
                };

            if commit_msg.is_empty() {
                bail!("commit message cannot be empty");
            }

            let mut should_stage_untracked = false;
            if all {
                run_git(&["add", "-A"])?;
                should_stage_untracked = true;
            } else if unstaged {
                run_git(&["add", "-u"])?;
            } else if !staged && !unstaged && !all {
            }

            if staged && (all || unstaged) {
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
            commit_args.push(commit_msg.as_str());

            run_git(&commit_args)?;

            if push {
                run_git(&["push"])?;
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
    version,
    propagate_version = true
)]
struct Cli {
    /// Show a brief, beginner-friendly explanation of every available command
    #[arg(long, global = true)]
    explain: bool,

    #[command(subcommand)]
    command: Option<SgitCommand>,
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
    Status {
        /// Short status output like `git status -sb`
        #[arg(long)]
        short: bool,
    },
    /// Commit with a simple interface
    Commit {
        /// Commit message
        #[arg(short, long, value_name = "MSG")]
        message: Option<String>,
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

fn print_explanations() {
    println!("SGIT simplifies Git for beginners by wrapping each major workflow:");
    println!();
    println!("  init    – initialize a Git repository (runs `git init`).");
    println!("  stage   – add files to the staging area (defaults to the repo root).");
    println!("  unstage – remove staged files safely (runs `git restore --staged`).");
    println!("  status  – show what is staged vs unstaged (`--short` uses `git status -sb`).");
    println!("  log     – view history (`--short` shows compact entries).");
    println!("  diff    – compare working changes (`--staged` shows what will be committed).");
    println!("  branch  – list local branches.");
    println!(
        "  push    – send commits to your remote (uses Git’s defaults unless you pass `--remote`/`--branch`)."
    );
    println!("  pull    – fetch + merge from your remote repository.");
    println!(
        "  commit  – make commits; `--all` stages everything, `--unstaged` stages only modified tracked files, `--push` runs `git push`, `--amend` rewrites the last commit, and `--no-verify` skips hooks."
    );
}
