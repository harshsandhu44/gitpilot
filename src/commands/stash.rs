use anyhow::Result;
use dialoguer::{Select, theme::ColorfulTheme};
use crate::commands::CommandContext;
use crate::display::theme;

struct StashEntry {
    index: usize,
    message: String,
}

pub fn run(ctx: &mut CommandContext) -> Result<()> {
    let mut entries: Vec<StashEntry> = Vec::new();

    ctx.repo.repo.stash_foreach(|index, message, _oid| {
        entries.push(StashEntry {
            index,
            message: message.to_string(),
        });
        true
    })?;

    if entries.is_empty() {
        if !ctx.json {
            println!("{}", theme::dim("No stashes."));
        }
        return Ok(());
    }

    let labels: Vec<String> = entries
        .iter()
        .map(|e| format!("stash@{{{}}}: {}", e.index, e.message))
        .collect();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select stash")
        .items(&labels)
        .default(0)
        .interact()?;

    let stash_index = entries[selected].index;

    let actions = vec!["Apply", "Pop", "Drop", "Cancel"];
    let action = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Action")
        .items(&actions)
        .default(0)
        .interact()?;

    match action {
        0 => {
            // Apply
            match ctx.repo.repo.stash_apply(stash_index, None) {
                Ok(()) => {
                    if !ctx.json {
                        println!("{}", theme::success("Stash applied."));
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("conflict") || msg.contains("merge") {
                        println!("{}", theme::warning(&format!("Conflicts when applying stash: {}", msg)));
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        1 => {
            // Pop
            match ctx.repo.repo.stash_pop(stash_index, None) {
                Ok(()) => {
                    if !ctx.json {
                        println!("{}", theme::success("Stash popped."));
                    }
                }
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("conflict") || msg.contains("merge") {
                        println!("{}", theme::warning(&format!("Conflicts when popping stash: {}", msg)));
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        2 => {
            // Drop
            ctx.repo.repo.stash_drop(stash_index)?;
            if !ctx.json {
                println!("{}", theme::success("Stash dropped."));
            }
        }
        _ => {
            if !ctx.json {
                println!("{}", theme::dim("Cancelled."));
            }
        }
    }

    Ok(())
}
