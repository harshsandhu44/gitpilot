use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gitpilot", about = "A Git assistant for daily developer workflow", version)]
pub struct Cli {
    #[arg(long, global = true, help = "Output results as JSON")]
    pub json: bool,

    #[arg(long, global = true, env = "NO_COLOR", help = "Disable color output")]
    pub no_color: bool,

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
        /// Preview branches to delete without actually deleting
        #[arg(long)]
        dry_run: bool,
    },
    /// Interactive branch switcher
    Switch {
        /// Include remote branches
        #[arg(long)]
        remote: bool,
    },
    /// Fetch and sync current branch with base
    Sync {
        /// Base branch to sync against
        #[arg(long, short)]
        base: Option<String>,
    },
    /// Browse commit history with filters
    Log {
        /// Filter by author name
        #[arg(long)]
        author: Option<String>,
        /// Filter since date (YYYY-MM-DD, Nd, Nw, Nm)
        #[arg(long)]
        since: Option<String>,
        /// Filter by commit message pattern
        #[arg(long)]
        grep: Option<String>,
        /// Number of commits to show
        #[arg(long, short, default_value = "30")]
        count: usize,
    },
    /// Interactive commit undo (reset HEAD)
    Undo {
        /// Number of recent commits to show for selection
        #[arg(long, short, default_value = "5")]
        count: usize,
    },
    /// Interactive stash manager
    Stash,
    /// Initialize a .gitpilot.toml config file
    Init {
        /// Also install a pre-commit hook
        #[arg(long)]
        hook: bool,
    },
    /// Generate shell completions or man page
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
    },
    /// Generate shell completion scripts (shorthand)
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
pub enum GenerateTarget {
    /// Generate shell completion script
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate man page
    Man {
        /// Output file path (defaults to stdout)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}
