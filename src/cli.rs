use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "git-pilot", about = "A Git assistant for daily developer workflow", version)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_status() {
        let cli = Cli::try_parse_from(["git-pilot", "status"]).unwrap();
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn parse_review() {
        let cli = Cli::try_parse_from(["git-pilot", "review"]).unwrap();
        assert!(matches!(cli.command, Commands::Review));
    }

    #[test]
    fn parse_summary_default() {
        let cli = Cli::try_parse_from(["git-pilot", "summary"]).unwrap();
        assert!(matches!(cli.command, Commands::Summary { base: None }));
    }

    #[test]
    fn parse_summary_with_base() {
        let cli = Cli::try_parse_from(["git-pilot", "summary", "--base", "develop"]).unwrap();
        assert!(matches!(cli.command, Commands::Summary { base: Some(ref b) } if b == "develop"));
    }

    #[test]
    fn parse_cleanup_default() {
        let cli = Cli::try_parse_from(["git-pilot", "cleanup"]).unwrap();
        assert!(matches!(cli.command, Commands::Cleanup { base: None, dry_run: false }));
    }

    #[test]
    fn parse_cleanup_dry_run() {
        let cli = Cli::try_parse_from(["git-pilot", "cleanup", "--dry-run"]).unwrap();
        assert!(matches!(cli.command, Commands::Cleanup { dry_run: true, .. }));
    }

    #[test]
    fn parse_cleanup_with_base_and_dry_run() {
        let cli =
            Cli::try_parse_from(["git-pilot", "cleanup", "--base", "main", "--dry-run"]).unwrap();
        assert!(matches!(cli.command, Commands::Cleanup { dry_run: true, base: Some(ref b) } if b == "main"));
    }

    #[test]
    fn parse_switch_default() {
        let cli = Cli::try_parse_from(["git-pilot", "switch"]).unwrap();
        assert!(matches!(cli.command, Commands::Switch { remote: false }));
    }

    #[test]
    fn parse_switch_remote() {
        let cli = Cli::try_parse_from(["git-pilot", "switch", "--remote"]).unwrap();
        assert!(matches!(cli.command, Commands::Switch { remote: true }));
    }

    #[test]
    fn parse_sync_default() {
        let cli = Cli::try_parse_from(["git-pilot", "sync"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync { base: None }));
    }

    #[test]
    fn parse_sync_with_base() {
        let cli = Cli::try_parse_from(["git-pilot", "sync", "--base", "main"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync { base: Some(ref b) } if b == "main"));
    }

    #[test]
    fn parse_log_defaults() {
        let cli = Cli::try_parse_from(["git-pilot", "log"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Log { author: None, since: None, grep: None, count: 30 }
        ));
    }

    #[test]
    fn parse_log_with_options() {
        let cli = Cli::try_parse_from([
            "git-pilot", "log", "--author", "alice", "--grep", "feat", "--count", "50",
        ])
        .unwrap();
        assert!(matches!(
            cli.command,
            Commands::Log { count: 50, .. }
        ));
        if let Commands::Log { author, grep, .. } = cli.command {
            assert_eq!(author.as_deref(), Some("alice"));
            assert_eq!(grep.as_deref(), Some("feat"));
        }
    }

    #[test]
    fn parse_undo_default_count() {
        let cli = Cli::try_parse_from(["git-pilot", "undo"]).unwrap();
        assert!(matches!(cli.command, Commands::Undo { count: 5 }));
    }

    #[test]
    fn parse_undo_custom_count() {
        let cli = Cli::try_parse_from(["git-pilot", "undo", "--count", "10"]).unwrap();
        assert!(matches!(cli.command, Commands::Undo { count: 10 }));
    }

    #[test]
    fn parse_stash() {
        let cli = Cli::try_parse_from(["git-pilot", "stash"]).unwrap();
        assert!(matches!(cli.command, Commands::Stash));
    }

    #[test]
    fn parse_init_no_hook() {
        let cli = Cli::try_parse_from(["git-pilot", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init { hook: false }));
    }

    #[test]
    fn parse_init_with_hook() {
        let cli = Cli::try_parse_from(["git-pilot", "init", "--hook"]).unwrap();
        assert!(matches!(cli.command, Commands::Init { hook: true }));
    }

    #[test]
    fn parse_completions_zsh() {
        let cli = Cli::try_parse_from(["git-pilot", "completions", "zsh"]).unwrap();
        assert!(matches!(cli.command, Commands::Completions { shell: Shell::Zsh }));
    }

    #[test]
    fn parse_completions_bash() {
        let cli = Cli::try_parse_from(["git-pilot", "completions", "bash"]).unwrap();
        assert!(matches!(cli.command, Commands::Completions { shell: Shell::Bash }));
    }

    #[test]
    fn parse_json_flag() {
        let cli = Cli::try_parse_from(["git-pilot", "--json", "status"]).unwrap();
        assert!(cli.json);
    }

    #[test]
    fn parse_no_color_flag() {
        let cli = Cli::try_parse_from(["git-pilot", "--no-color", "status"]).unwrap();
        assert!(cli.no_color);
    }

    #[test]
    fn parse_unknown_command_fails() {
        assert!(Cli::try_parse_from(["git-pilot", "notacommand"]).is_err());
    }

    #[test]
    fn parse_generate_completions() {
        let cli = Cli::try_parse_from(["git-pilot", "generate", "completions", "fish"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Generate { target: GenerateTarget::Completions { shell: Shell::Fish } }
        ));
    }

    #[test]
    fn parse_generate_man() {
        let cli = Cli::try_parse_from(["git-pilot", "generate", "man"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Generate { target: GenerateTarget::Man { output: None } }
        ));
    }
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
