use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sgit",
    about = "Blazing fast wrapper for Git with simplified workflows",
    version,
    propagate_version = true
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub explain: bool,

    #[command(subcommand)]
    pub command: Option<SgitCommand>,
}

#[derive(Subcommand)]
pub enum SgitCommand {
    Init,
    Stage {
        #[arg(value_name = "PATH")]
        targets: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        tracked: bool,
    },
    Unstage {
        #[arg(value_name = "PATH")]
        targets: Vec<String>,
        #[arg(long)]
        all: bool,
    },
    Status {
        #[arg(long)]
        short: bool,
    },
    Commit {
        #[arg(short, long, value_name = "MSG")]
        message: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        unstaged: bool,
        #[arg(long)]
        push: bool,
        #[arg(long)]
        amend: bool,
        #[arg(long)]
        no_verify: bool,
    },
    Log {
        #[arg(long)]
        short: bool,
    },
    Diff {
        path: Option<String>,
        #[arg(long)]
        staged: bool,
    },
    Reset {
        #[arg(long)]
        all: bool,
        #[arg(long)]
        staged: bool,
        #[arg(long)]
        unstaged: bool,
        #[arg(long)]
        tracked: bool,
        #[arg(long)]
        untracked: bool,
    },
    Branch {
        #[arg(short, long)]
        create: Option<String>,
    },
    Push {
        remote: Option<String>,
        branch: Option<String>,
    },
    Pull {
        remote: Option<String>,
        branch: Option<String>,
    },
    Sync {
        remote: Option<String>,
        branch: Option<String>,
    },
}
