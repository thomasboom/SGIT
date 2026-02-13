# AGENTS.md - SGIT Project Guidelines

## Project Overview

SGIT (Simple Git) is a Rust CLI wrapper around Git that provides simplified workflows for common Git operations. It is a single-binary project with no external tests currently.

## Build/Lint/Test Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the binary directly
cargo run -- <command>

# Check for compilation errors (faster than build)
cargo check

# Run all tests (if any are added)
cargo test

# Run a specific test by name
cargo test <test_name>

# Run a specific test file
cargo test --test <test_file>

# Run tests with output shown
cargo test -- --nocapture

# Lint with clippy
cargo clippy

# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check
```

## Code Style Guidelines

### Imports

- Group imports logically: standard library first, then external crates, then local modules
- Use `use` statements at the top of the file
- Import specific items rather than glob imports (`use anyhow::{bail, Context, Result}` not `use anyhow::*`)
- Rename imports to avoid conflicts (e.g., `use std::process::Command as StdCommand`)

```rust
use std::process::Command as StdCommand;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{Confirm, Input, Select};
```

### Formatting

- Use 4 spaces for indentation (Rust standard)
- Maximum line length: 100 characters
- Place opening braces on the same line
- Use `cargo fmt` to auto-format before committing

### Types

- Use `Result<T>` as the return type for fallible functions (aliased from `anyhow::Result`)
- Use `Option<T>` for optional values
- Prefer `String` for owned strings, `&str` for borrowed
- Use `Vec<T>` for collections

### Naming Conventions

- **Functions**: snake_case (`run_git`, `get_staged_files`, `reset_all`)
- **Structs/Enums**: PascalCase (`Cli`, `SgitCommand`)
- **Constants**: SCREAMING_SNAKE_CASE
- **Variables**: snake_case
- **Enum variants**: PascalCase (`SgitCommand::Init`, `SgitCommand::Stage`)

### Error Handling

- Use `anyhow` crate for error handling
- Use `bail!` macro for early returns with an error message
- Use `.context()` to add context to operations that may fail

```rust
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
```

### CLI Structure

- Use `clap` derive macros (`#[derive(Parser, Subcommand)]`)
- Document commands and arguments with doc comments (`///`)
- Use `#[arg(long)]` for long flags, `#[arg(short, long)]` for both
- Use `#[command(subcommand)]` for subcommands

```rust
#[derive(Parser)]
#[command(name = "sgit", about = "Description", version)]
struct Cli {
    #[arg(long, global = true)]
    explain: bool,

    #[command(subcommand)]
    command: Option<SgitCommand>,
}
```

### Functions

- Keep functions focused on a single responsibility
- Use `-> Result<()>` for functions that may fail
- Pattern match on `is_interactive` to handle CLI vs interactive modes
- Extract helper functions for repeated logic

### Pattern Matching

- Use `match` for enum dispatch
- Handle all variants explicitly
- Use `_ => {}` for no-op default cases when appropriate

### Command Execution

- Use `std::process::Command` for running Git commands
- Check `status.success()` before returning `Ok(())`
- Use `.output()` when you need to capture stdout/stderr

```rust
let output = StdCommand::new("git")
    .args(["status", "--porcelain"])
    .output()
    .context("running git status --porcelain")?;
```

### String Handling

- Use `String::from_utf8_lossy()` for converting command output
- Use `.trim()` to clean up output strings
- Use `.lines()` for iterating over multi-line output
- Use `.as_str()` to convert `String` to `&str` when needed

## Architecture Notes

- Single-file architecture (`src/main.rs`) - the project is small enough to not need modules
- Entry point is `fn main()` which calls `run()` and handles errors
- All Git operations go through `run_git()` or `run_git_in_dir()` helper functions
- Interactive prompts use the `dialoguer` crate
- User-facing errors are printed via `eprintln!`

## Adding New Commands

1. Add a new variant to `SgitCommand` enum with doc comment
2. Add any required arguments as fields with `#[arg]` attributes
3. Add a match arm in the main `match command { }` block
4. Implement the command logic in a dedicated function
5. Update `print_explanations()` to document the new command

## Dependencies

- `anyhow`: Error handling
- `clap`: CLI argument parsing (derive feature enabled)
- `dialoguer`: Interactive prompts (Select, Input, Confirm, MultiSelect)