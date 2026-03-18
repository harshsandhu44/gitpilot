use anyhow::{anyhow, Result};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde::Serialize;
use crate::commands::CommandContext;
use crate::display::theme;
use crate::git::commits;

#[derive(Serialize)]
struct UndoJson {
    reset_mode: String,
    commits_undone: usize,
    new_head: String,
}

pub fn run(ctx: &CommandContext, count: usize) -> Result<()> {
    let recent = commits::recent(&ctx.repo, count)?;

    if recent.is_empty() {
        return Err(anyhow!("No commits found in history."));
    }

    let items: Vec<String> = recent
        .iter()
        .enumerate()
        .map(|(_i, c)| format!("{} {} — {}", c.short_id, c.message.chars().take(60).collect::<String>(), c.author))
        .collect();

    // Show the table and ask how many to undo
    println!("{}", theme::heading("Recent commits:"));
    for item in &items {
        println!("  {}", item);
    }
    println!();

    let undo_labels: Vec<String> = (1..=recent.len())
        .map(|n| format!("Undo {} commit{}", n, if n == 1 { "" } else { "s" }))
        .collect();

    let undo_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How many commits to undo?")
        .items(&undo_labels)
        .default(0)
        .interact()?;

    let n = undo_sel + 1;

    let mode_labels = vec![
        "Soft  — keep changes staged",
        "Mixed — keep changes unstaged (default)",
        "Hard  — discard all working tree changes",
    ];

    let mode_sel = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Reset mode")
        .items(&mode_labels)
        .default(1)
        .interact()?;

    let reset_mode = match mode_sel {
        0 => git2::ResetType::Soft,
        1 => git2::ResetType::Mixed,
        _ => git2::ResetType::Hard,
    };

    if mode_sel == 2 {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Hard reset will discard all working tree changes. Continue?")
            .default(false)
            .interact()?;
        if !confirmed {
            println!("{}", theme::dim("Aborted."));
            return Ok(());
        }
    }

    // Walk back N parents from HEAD
    let repo = &ctx.repo.repo;
    let head = repo.head()?.peel_to_commit()?;
    let mut target = head.clone();
    for i in 0..n {
        let parent_count = target.parent_count();
        if parent_count == 0 {
            return Err(anyhow!("Not enough history: only {} commit{} available.", i, if i == 1 { "" } else { "s" }));
        }
        target = target.parent(0)?;
    }

    let target_obj = target.as_object();
    repo.reset(target_obj, reset_mode, None)?;

    let mode_str = match mode_sel {
        0 => "soft",
        1 => "mixed",
        _ => "hard",
    };

    if ctx.json {
        println!("{}", serde_json::to_string(&UndoJson {
            reset_mode: mode_str.to_string(),
            commits_undone: n,
            new_head: target.id().to_string().chars().take(7).collect(),
        })?);
    } else {
        println!(
            "{} {} commit{} ({} reset). HEAD is now {}",
            theme::success("Undone:"),
            n,
            if n == 1 { "" } else { "s" },
            mode_str,
            target.id().to_string().chars().take(7).collect::<String>(),
        );
    }

    Ok(())
}
