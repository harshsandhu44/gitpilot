mod cli;
mod commands;
mod config;
mod display;
mod error;
mod git;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use commands::CommandContext;
use config::Config;
use git::RepoContext;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Commands::Completions { shell } = &cli.command {
        return commands::completions::run(*shell);
    }

    let config = Config::load()?;

    match &cli.command {
        Commands::Status => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config };
            commands::status::run(&ctx)?;
        }
        Commands::Summary { base } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config };
            commands::summary::run(&ctx, base.as_deref())?;
        }
        Commands::Review => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config };
            let exit_code = commands::review::run(&ctx)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Commands::Cleanup { base } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config };
            commands::cleanup::run(&ctx, base.as_deref())?;
        }
        Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
}
