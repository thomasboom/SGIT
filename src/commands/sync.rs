use anyhow::Result;

use crate::git::run_git_quiet;
use crate::status::get_current_branch;

pub fn run_push(remote: Option<String>, branch: Option<String>) -> Result<()> {
    if remote.is_none() && branch.is_some() {
        anyhow::bail!("cannot specify --branch without --remote");
    }

    print!("→ Pushing");
    if let Some(ref r) = remote {
        print!(" to {}", r);
    }
    if let Some(ref b) = branch {
        print!("/{}", b);
    }
    println!("...");

    let mut args_owned = vec!["push".to_string()];
    if let Some(remote) = remote {
        args_owned.push(remote);
        if let Some(branch) = branch {
            args_owned.push(branch);
        }
    }

    let args_refs: Vec<&str> = args_owned.iter().map(String::as_str).collect();
    run_git_quiet(&args_refs)?;
    println!("✓ Pushed successfully");
    Ok(())
}

pub fn run_pull(remote: Option<String>, branch: Option<String>) -> Result<()> {
    print!("→ Pulling");
    if let Some(ref r) = remote {
        print!(" from {}", r);
    }
    if let Some(ref b) = branch {
        print!("/{}", b);
    }
    println!("...");

    let mut args_owned = vec!["pull".to_string()];
    if let Some(remote) = remote {
        args_owned.push(remote);
        if let Some(branch) = branch {
            args_owned.push(branch);
        }
    }

    let args_refs: Vec<&str> = args_owned.iter().map(String::as_str).collect();
    run_git_quiet(&args_refs)?;
    println!("✓ Pulled successfully");
    Ok(())
}

pub fn run_sync(remote: Option<&str>, branch: Option<&str>) -> Result<()> {
    let remote_name = remote.unwrap_or("origin");

    println!("→ Fetching from {}...", remote_name);
    let fetch_result = run_git_quiet(&["fetch", remote_name]);
    if let Err(e) = fetch_result {
        let err_str = e.to_string();
        if err_str.contains("could not resolve host") || err_str.contains("network") {
            eprintln!("✗ Network error: cannot reach '{}'", remote_name);
            return Err(e);
        }
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

    let pull_result = run_git_quiet(&pull_refs);
    if let Err(e) = pull_result {
        let err_str = e.to_string();
        if err_str.contains("CONFLICT") || err_str.contains("merge conflict") {
            eprintln!("✗ Pull failed due to merge conflicts");
            eprintln!("  Resolve conflicts manually:");
            eprintln!("    1. Edit conflicting files (marked with <<<<<<<)");
            eprintln!("    2. Run 'sgit stage .' to stage resolved files");
            eprintln!("    3. Run 'sgit commit' to complete the merge");
            return Err(e);
        }
        if err_str.contains("no tracking information") {
            eprintln!("✗ Branch has no upstream configured");
            eprintln!(
                "  Try: git branch --set-upstream-to={}/{}",
                remote_name,
                get_current_branch().unwrap_or_default()
            );
            return Err(e);
        }
        eprintln!("⚠ Pull failed: {}", e);
        eprintln!("  Attempting to push local changes anyway...");
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

    let push_result = run_git_quiet(&push_refs);
    if let Err(e) = push_result {
        let err_str = e.to_string();
        if err_str.contains("rejected") {
            eprintln!("✗ Push rejected: remote has new commits");
            eprintln!("  Run 'sgit pull' first to integrate remote changes.");
        } else if err_str.contains("no upstream branch") {
            eprintln!("✗ No upstream branch configured");
            eprintln!(
                "  Try: git push -u {} {}",
                remote_name,
                get_current_branch().unwrap_or_default()
            );
        } else {
            eprintln!("✗ Push failed: {}", e);
        }
        return Err(e);
    }

    println!("✓ Sync complete: fetched, pulled, and pushed successfully.");
    Ok(())
}
