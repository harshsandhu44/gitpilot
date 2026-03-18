use anyhow::{anyhow, Result};
use serde::Serialize;
use std::process::Stdio;
use crate::commands::CommandContext;
use crate::config::SyncStrategy;
use crate::display::theme;

#[derive(Serialize)]
struct SyncJson {
    success: bool,
    strategy: String,
    dry_run: bool,
}

pub fn run(ctx: &CommandContext, base: Option<&str>, dry_run: bool) -> Result<()> {
    let base_branch = base
        .unwrap_or(&ctx.config.base_branch)
        .to_string();

    let strategy_str = match ctx.config.sync_strategy {
        SyncStrategy::Rebase => "rebase",
        SyncStrategy::Merge => "merge",
    };

    if dry_run {
        if ctx.json {
            println!("{}", serde_json::to_string(&SyncJson {
                success: true,
                strategy: strategy_str.to_string(),
                dry_run: true,
            })?);
        } else {
            println!("{}", theme::heading(&format!("Would sync with origin/{} (dry run)", base_branch)));
            println!("  Strategy: {}", strategy_str);
            println!("{}", theme::dim("No changes made."));
        }
        return Ok(());
    }

    if !ctx.json {
        println!("{}", theme::heading(&format!("Syncing with origin/{}", base_branch)));
    }

    // Fetch
    let fetch_status = std::process::Command::new("git")
        .args(["fetch", "origin"])
        .stdout(if ctx.json { Stdio::null() } else { Stdio::inherit() })
        .stderr(if ctx.json { Stdio::null() } else { Stdio::inherit() })
        .status()?;

    if !fetch_status.success() {
        return Err(anyhow!("git fetch origin failed. Check your network connection and remote configuration."));
    }

    let remote_ref = format!("origin/{}", base_branch);
    let args: Vec<&str> = match ctx.config.sync_strategy {
        SyncStrategy::Rebase => vec!["rebase", &remote_ref],
        SyncStrategy::Merge => vec!["merge", &remote_ref],
    };

    let sync_status = std::process::Command::new("git")
        .args(&args)
        .stdout(if ctx.json { Stdio::null() } else { Stdio::inherit() })
        .stderr(if ctx.json { Stdio::null() } else { Stdio::inherit() })
        .status()?;

    if !sync_status.success() {
        let hint = if matches!(ctx.config.sync_strategy, SyncStrategy::Rebase) {
            "Resolve conflicts, then run `git rebase --continue` or `git rebase --abort`."
        } else {
            "Resolve conflicts, then commit the merge."
        };
        return Err(anyhow!("git {} failed. {}", strategy_str, hint));
    }

    if ctx.json {
        println!("{}", serde_json::to_string(&SyncJson {
            success: true,
            strategy: strategy_str.to_string(),
            dry_run: false,
        })?);
    } else {
        println!("{}", theme::success(&format!("Synced via {} with origin/{}", strategy_str, base_branch)));
    }

    Ok(())
}
