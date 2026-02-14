use anyhow::{bail, Result};
use dialoguer::{Input, Select};

use crate::git::run_git_silent;
use crate::status::{get_branches, get_current_branch};

pub fn create_branch(branch_name: &str) -> Result<()> {
    let branch_name = branch_name.trim();
    if branch_name.is_empty() {
        bail!("branch name cannot be empty");
    }
    if branch_name.contains(|c: char| c.is_whitespace()) {
        bail!("branch name cannot contain whitespace");
    }
    run_git_silent(&["branch", branch_name])?;
    run_git_silent(&["checkout", branch_name])?;
    println!("✓ Created and switched to branch '{}'", branch_name);
    Ok(())
}

pub fn run_branch_interactive() -> Result<()> {
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
        run_git_silent(&["branch", &normalized_name])?;
        run_git_silent(&["checkout", &normalized_name])?;
        println!("✓ Created and switched to branch '{}'", normalized_name);
    } else {
        let selected_branch = &branches[selection];
        if selected_branch == &current {
            println!("Already on branch '{}'.", selected_branch);
        } else {
            run_git_silent(&["checkout", selected_branch])?;
            println!("✓ Switched to branch '{}'", selected_branch);
        }
    }

    Ok(())
}
