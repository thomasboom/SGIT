use std::process::Command as StdCommand;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Input, Select};

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
        SgitCommand::Stage {
            targets,
            all,
            tracked,
        } => stage_targets(&targets, all, tracked)?,
        SgitCommand::Unstage { targets, all } => restore_stage(&targets, all)?,
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
        SgitCommand::Branch { create } => {
            if let Some(branch_name) = create {
                run_git(&["branch", &branch_name])?;
                run_git(&["checkout", &branch_name])?;
            } else {
                run_branch_interactive()?;
            }
        }
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
        SgitCommand::Sync { remote, branch } => {
            run_sync(remote.as_deref(), branch.as_deref())?;
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
            let is_interactive = message.is_none() && !all && !staged && !unstaged;
            let (all, staged, unstaged, commit_msg, push, custom_files) = if is_interactive {
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

                let mut custom_files: Vec<String> = Vec::new();
                if scope == 3 {
                    let files = get_all_uncommitted_files()?;
                    if files.is_empty() {
                        println!("No files to commit.");
                        return Ok(());
                    }
                    let selected = dialoguer::MultiSelect::new()
                        .with_prompt("Select files to stage")
                        .items(&files)
                        .interact()?;

                    if selected.is_empty() {
                        println!("No files selected.");
                        return Ok(());
                    }

                    for idx in selected {
                        custom_files.push(files[idx].clone());
                    }
                }

                let msg: String = Input::new().with_prompt("Commit message").interact()?;
                let should_push = Confirm::new()
                    .with_prompt("Push after committing?")
                    .default(false)
                    .interact()?;
                (all, staged, unstaged, msg, should_push, custom_files)
            } else {
                let msg = message.unwrap_or_default();
                (all, staged, unstaged, msg, push, Vec::new())
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
            } else if !custom_files.is_empty() {
                let repo_root = get_repo_root()?;
                let mut args = vec!["add".to_string()];
                args.extend(custom_files.iter().map(|s| s.clone()));
                let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                run_git_in_dir(&args_refs, &repo_root)?;
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
    /// Stage files (interactive if no targets/flags provided)
    Stage {
        #[arg(value_name = "PATH")]
        targets: Vec<String>,
        /// Stage all files (tracked + untracked)
        #[arg(long)]
        all: bool,
        /// Stage only tracked files
        #[arg(long)]
        tracked: bool,
    },
    /// Unstage files or reset staged changes (interactive if no targets/flags provided)
    Unstage {
        #[arg(value_name = "PATH")]
        targets: Vec<String>,
        /// Unstage all staged files
        #[arg(long)]
        all: bool,
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
    /// List and checkout branches (interactive)
    Branch {
        /// Create a new branch
        #[arg(short, long)]
        create: Option<String>,
    },
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
    /// Sync: fetch, pull, and push in one command
    Sync {
        /// Remote name
        remote: Option<String>,
        /// Branch name
        branch: Option<String>,
    },
}

fn stage_targets(targets: &[String], all: bool, tracked: bool) -> Result<()> {
    let is_interactive = targets.is_empty() && !all && !tracked;

    if is_interactive {
        let selection = Select::new()
            .with_prompt("What would you like to stage?")
            .items(&["All files", "Tracked files only", "Specific files"])
            .default(0)
            .interact()?;

        match selection {
            0 => run_git(&["add", "-A"]),
            1 => run_git(&["add", "-u"]),
            2 => {
                let files = get_unstaged_files()?;
                if files.is_empty() {
                    println!("No unstaged files to stage.");
                    return Ok(());
                }
                let selected = dialoguer::MultiSelect::new()
                    .with_prompt("Select files to stage")
                    .items(&files)
                    .interact()?;

                if selected.is_empty() {
                    println!("No files selected.");
                    return Ok(());
                }

                let repo_root = get_repo_root()?;
                let mut args = vec!["add".to_string()];
                for idx in selected {
                    args.push(files[idx].clone());
                }
                let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                run_git_in_dir(&args_refs, &repo_root)
            }
            _ => Ok(()),
        }
    } else if all {
        run_git(&["add", "-A"])
    } else if tracked {
        run_git(&["add", "-u"])
    } else {
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
}

fn get_repo_root() -> Result<String> {
    let output = StdCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("getting repo root")?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout);
        Ok(path.trim().to_string())
    } else {
        bail!("failed to get repo root");
    }
}

fn get_porcelain_lines() -> Result<Vec<(String, String)>> {
    let output = StdCommand::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("running git status --porcelain")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries: Vec<(String, String)> = stdout
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }
            let status = line[..2].to_string();
            let path = line[3..].to_string();
            Some((status, path))
        })
        .collect();

    Ok(entries)
}

fn get_unstaged_files() -> Result<Vec<String>> {
    let entries = get_porcelain_lines()?;
    let files: Vec<String> = entries
        .into_iter()
        .filter(|(status, _)| {
            let xy = status.chars().collect::<Vec<_>>();
            let x = xy.get(0).copied().unwrap_or(' ');
            let y = xy.get(1).copied().unwrap_or(' ');
            x == ' ' && y != ' ' && y != '?'
        })
        .map(|(_, path)| path)
        .collect();

    Ok(files)
}

fn get_staged_files() -> Result<Vec<String>> {
    let entries = get_porcelain_lines()?;
    let files: Vec<String> = entries
        .into_iter()
        .filter(|(status, _)| {
            let x = status.chars().next().unwrap_or(' ');
            matches!(x, 'M' | 'A' | 'D' | 'R' | 'C')
        })
        .map(|(_, path)| path)
        .collect();

    Ok(files)
}

fn get_all_uncommitted_files() -> Result<Vec<String>> {
    let entries = get_porcelain_lines()?;
    let files: Vec<String> = entries.into_iter().map(|(_, path)| path).collect();
    Ok(files)
}

fn restore_stage(targets: &[String], all: bool) -> Result<()> {
    let is_interactive = targets.is_empty() && !all;

    if is_interactive {
        let selection = Select::new()
            .with_prompt("What would you like to unstage?")
            .items(&["All staged files", "Specific files"])
            .default(0)
            .interact()?;

        match selection {
            0 => run_git(&["restore", "--staged", "."]),
            1 => {
                let files = get_staged_files()?;
                if files.is_empty() {
                    println!("No staged files to unstage.");
                    return Ok(());
                }
                let selected = dialoguer::MultiSelect::new()
                    .with_prompt("Select files to unstage")
                    .items(&files)
                    .interact()?;

                if selected.is_empty() {
                    println!("No files selected.");
                    return Ok(());
                }

                let repo_root = get_repo_root()?;
                let mut args = vec!["restore".to_string(), "--staged".to_string()];
                for idx in selected {
                    args.push(files[idx].clone());
                }
                let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
                run_git_in_dir(&args_refs, &repo_root)
            }
            _ => Ok(()),
        }
    } else if all {
        run_git(&["restore", "--staged", "."])
    } else {
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

fn run_git_in_dir(args: &[&str], dir: &str) -> Result<()> {
    let status = StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("running git {} in {}", args.join(" "), dir))?;

    if status.success() {
        Ok(())
    } else {
        bail!("git {} failed with {}", args.join(" "), status);
    }
}

fn run_sync(remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let remote_name = remote.unwrap_or("origin");

    println!("→ Fetching from {}...", remote_name);
    let fetch_result = run_git_quiet(&["fetch", remote_name]);
    if let Err(e) = fetch_result {
        eprintln!("⚠ Fetch failed: {}", e);
        eprintln!("  Continuing with local state...");
    } else {
        println!("✓ Fetch complete");
    }

    println!("→ Pulling changes...");
    let mut pull_args = vec!["pull"];
    let mut pull_owned: Vec<String> = Vec::new();
    if let Some(r) = remote {
        pull_owned.push(r.to_string());
        if let Some(b) = branch {
            pull_owned.push(b.to_string());
        }
    }
    let pull_refs: Vec<&str> = if pull_owned.is_empty() {
        pull_args
    } else {
        pull_args.extend(pull_owned.iter().map(String::as_str));
        pull_args
    };

    let pull_result = run_git(&pull_refs);
    let had_conflicts = pull_result.is_err();
    if let Err(e) = pull_result {
        if e.to_string().contains("CONFLICT") || e.to_string().contains("merge conflict") {
            eprintln!("✗ Pull failed due to merge conflicts");
            eprintln!("  Please resolve conflicts and run 'sgit push' manually.");
            return Err(e);
        } else {
            eprintln!("⚠ Pull failed: {}", e);
            eprintln!("  Attempting to push local changes anyway...");
        }
    } else {
        println!("✓ Pull complete");
    }

    println!("→ Pushing changes...");
    let mut push_args = vec!["push"];
    let mut push_owned: Vec<String> = Vec::new();
    if let Some(r) = remote {
        push_owned.push(r.to_string());
        if let Some(b) = branch {
            push_owned.push(b.to_string());
        }
    }
    let push_refs: Vec<&str> = if push_owned.is_empty() {
        push_args
    } else {
        push_args.extend(push_owned.iter().map(String::as_str));
        push_args
    };

    let push_result = run_git(&push_refs);
    if let Err(e) = push_result {
        eprintln!("✗ Push failed: {}", e);
        if had_conflicts {
            eprintln!("  Resolve conflicts first, then push manually.");
        } else {
            eprintln!("  Check your permissions or network connection.");
        }
        return Err(e);
    }

    println!("✓ Sync complete: fetched, pulled, and pushed successfully.");
    Ok(())
}

fn run_git_quiet(args: &[&str]) -> Result<()> {
    let output = StdCommand::new("git")
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
}

fn get_branches() -> Result<Vec<String>> {
    let output = StdCommand::new("git")
        .args(["branch", "--format=%(refname:short)"])
        .output()
        .context("running git branch")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(branches)
}

fn get_current_branch() -> Result<String> {
    let output = StdCommand::new("git")
        .args(["branch", "--show-current"])
        .output()
        .context("getting current branch")?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn run_branch_interactive() -> Result<()> {
    let branches = get_branches()?;
    let current = get_current_branch().unwrap_or_default();

    let mut display_branches: Vec<String> = branches
        .iter()
        .map(|b| {
            if b == &current {
                format!("{} (current)", b)
            } else {
                b.clone()
            }
        })
        .collect();
    display_branches.push("Create new branch...".to_string());

    let selection = Select::new()
        .with_prompt("Select a branch to checkout")
        .items(&display_branches)
        .default(0)
        .interact()?;

    if selection == branches.len() {
        let branch_name: String = Input::new().with_prompt("New branch name").interact()?;

        if branch_name.is_empty() {
            bail!("branch name cannot be empty");
        }

        let normalized_name = branch_name.trim().replace(' ', "-");
        run_git(&["branch", &normalized_name])?;
        run_git(&["checkout", &normalized_name])?;
    } else {
        let selected_branch = &branches[selection];
        if selected_branch == &current {
            println!("Already on branch '{}'.", selected_branch);
        } else {
            run_git(&["checkout", selected_branch])?;
        }
    }

    Ok(())
}

fn print_explanations() {
    println!("SGIT simplifies Git for beginners by wrapping each major workflow:");
    println!();
    println!("  init    – initialize a Git repository (runs `git init`).");
    println!("  stage   – add files to the staging area (interactive, or use --all/--tracked).");
    println!("  unstage – remove staged files safely (interactive, or use --all).");
    println!("  status  – show what is staged vs unstaged (`--short` uses `git status -sb`).");
    println!("  log     – view history (`--short` shows compact entries).");
    println!("  diff    – compare working changes (`--staged` shows what will be committed).");
    println!("  branch  – list and checkout branches (interactive); use -c <name> to create a new branch.");
    println!(
        "  push    – send commits to your remote (uses Git's defaults unless you pass `--remote`/`--branch`)."
    );
    println!("  pull    – fetch + merge from your remote repository.");
    println!(
        "  commit  – make commits; `--all` stages everything, `--unstaged` stages only modified tracked files, `--push` runs `git push`, `--amend` rewrites the last commit, and `--no-verify` skips hooks."
    );
    println!("  sync    – fetch, pull, and push in one command with graceful error handling.");
}
