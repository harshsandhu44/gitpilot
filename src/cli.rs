use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitpilot", about = "A Git assistant for daily developer workflow", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show repository status: staged, unstaged, stash, upstream, recent commits
    Status,
    /// Summarize commits and diff vs base branch
    Summary {
        /// Base branch to compare against
        #[arg(long, short)]
        base: Option<String>,
    },
    /// Review staged changes for potential issues
    Review,
    /// Clean up merged, stale, or gone branches
    Cleanup {
        /// Base branch to check merges against
        #[arg(long, short)]
        base: Option<String>,
    },
}
