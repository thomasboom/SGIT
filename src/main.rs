mod cli;
mod commands;
mod git;
mod status;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{Cli, SgitCommand};
use commands::{
    create_branch, restore_stage, run_branch_interactive, run_commit, run_pull, run_push,
    run_reset, run_sync, stage_targets,
};
use git::{check_in_repo, run_git, run_git_silent};

fn main() {
    if let Err(err) = run() {
        for cause in err.chain() {
            eprintln!("error: {}", cause);
        }
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

    if !matches!(command, SgitCommand::Init) {
        check_in_repo()?;
    }

    match command {
        SgitCommand::Init => {
            run_git_silent(&["init"])?;
            println!("✓ Initialized Git repository");
        }
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
        SgitCommand::Reset {
            all,
            staged,
            unstaged,
            tracked,
            untracked,
        } => run_reset(all, staged, unstaged, tracked, untracked)?,
        SgitCommand::Branch { create } => {
            if let Some(branch_name) = create {
                create_branch(&branch_name)?;
            } else {
                run_branch_interactive()?;
            }
        }
        SgitCommand::Push { remote, branch } => {
            run_push(remote, branch)?;
        }
        SgitCommand::Pull { remote, branch } => {
            run_pull(remote, branch)?;
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
            run_commit(message, all, staged, unstaged, push, amend, no_verify)?;
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
    println!("  reset   – discard changes (interactive, or use --all/--staged/--unstaged/--tracked/--untracked).");
    println!(
        "  push    – send commits to your remote (uses Git's defaults unless you pass `--remote`/`--branch`)."
    );
    println!("  pull    – fetch + merge from your remote repository.");
    println!(
        "  commit  – make commits; `--all` stages everything, `--unstaged` stages only modified tracked files, `--push` runs `git push`, `--amend` rewrites the last commit, and `--no-verify` skips hooks."
    );
    println!("  sync    – fetch, pull, and push in one command with graceful error handling.");
}
