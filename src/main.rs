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
use display::theme;
use git::RepoContext;
use std::thread::JoinHandle;

fn maybe_start_update_checker() -> Option<JoinHandle<Option<String>>> {
    if std::env::var("GITPILOT_NO_UPDATE_CHECK").is_ok() {
        return None;
    }
    Some(std::thread::spawn(|| {
        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(2))
            .build();
        let body: serde_json::Value = agent
            .get("https://crates.io/api/v1/crates/gitpilot")
            .set("User-Agent", concat!("gitpilot/", env!("CARGO_PKG_VERSION")))
            .call()
            .ok()?
            .into_json()
            .ok()?;
        let latest = body["crate"]["newest_version"].as_str()?.to_string();
        if latest != env!("CARGO_PKG_VERSION") {
            Some(latest)
        } else {
            None
        }
    }))
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.no_color {
        owo_colors::set_override(false);
    }

    // Early exits — no repo needed
    match &cli.command {
        Commands::Completions { shell } => {
            return commands::completions::run(*shell);
        }
        Commands::Generate { target } => {
            return commands::generate::run(target);
        }
        Commands::Init { hook } => {
            let config = Config::default();
            return commands::init::run(&config, *hook);
        }
        _ => {}
    }

    let update_handle = maybe_start_update_checker();

    let config = Config::load()?;

    let result = run_command(&cli, config);

    // Print update notice if available
    if let Some(handle) = update_handle {
        if let Ok(Some(v)) = handle.join() {
            eprintln!(
                "{}",
                theme::dim(&format!(
                    "Update available: {} → {}. Run `cargo install gitpilot`.",
                    env!("CARGO_PKG_VERSION"),
                    v
                ))
            );
        }
    }

    result
}

fn run_command(cli: &Cli, config: Config) -> Result<()> {
    match &cli.command {
        Commands::Status => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::status::run(&ctx)?;
        }
        Commands::Summary { base } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::summary::run(&ctx, base.as_deref())?;
        }
        Commands::Review => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            let exit_code = commands::review::run(&ctx)?;
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        Commands::Cleanup { base, dry_run } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::cleanup::run(&ctx, base.as_deref(), *dry_run)?;
        }
        Commands::Switch { remote } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::switch::run(&ctx, *remote)?;
        }
        Commands::Sync { base } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::sync::run(&ctx, base.as_deref())?;
        }
        Commands::Log { author, since, grep, count } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::log::run(&ctx, author.as_deref(), since.as_deref(), grep.as_deref(), *count)?;
        }
        Commands::Undo { count } => {
            let repo = RepoContext::open()?;
            let ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::undo::run(&ctx, *count)?;
        }
        Commands::Stash => {
            let repo = RepoContext::open()?;
            let mut ctx = CommandContext { repo, config, json: cli.json, no_color: cli.no_color };
            commands::stash::run(&mut ctx)?;
        }
        Commands::Completions { .. } | Commands::Generate { .. } | Commands::Init { .. } => {
            unreachable!()
        }
    }

    Ok(())
}
