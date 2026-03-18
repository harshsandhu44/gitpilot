use anyhow::Result;
use dialoguer::{MultiSelect, Confirm, theme::ColorfulTheme};
use crate::commands::CommandContext;
use crate::display::{tables, theme};
use crate::git::branches::{list_branches, BranchState};

pub fn run(ctx: &CommandContext, base: Option<&str>) -> Result<()> {
    let base_branch = base
        .unwrap_or(&ctx.config.base_branch)
        .to_string();

    let repo = &ctx.repo.repo;
    let current_branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    let all_branches = list_branches(&ctx.repo, &base_branch, ctx.config.stale_days)?;

    let candidates: Vec<_> = all_branches
        .iter()
        .filter(|b| {
            b.name != current_branch
                && !ctx.config.protected_branches.contains(&b.name)
                && b.state != BranchState::Active
        })
        .collect();

    if candidates.is_empty() {
        println!("{}", theme::success("No branches to clean up."));
        return Ok(());
    }

    println!("{}", theme::heading("Branches eligible for cleanup:"));
    println!("{}", tables::branch_table(&candidates));
    println!();

    let labels: Vec<String> = candidates
        .iter()
        .map(|b| format!("{} ({:?})", b.name, b.state))
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select branches to delete")
        .items(&labels)
        .interact()?;

    if selections.is_empty() {
        println!("{}", theme::dim("No branches selected."));
        return Ok(());
    }

    let selected_names: Vec<&str> = selections.iter().map(|&i| candidates[i].name.as_str()).collect();

    println!();
    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Delete {} branch{}? This cannot be undone.",
            selections.len(),
            if selections.len() == 1 { "" } else { "es" }
        ))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("{}", theme::dim("Aborted."));
        return Ok(());
    }

    for name in selected_names {
        match repo.find_branch(name, git2::BranchType::Local) {
            Ok(mut branch) => match branch.delete() {
                Ok(_) => println!("{} {}", theme::success("Deleted:"), name),
                Err(e) => println!("{} {}: {}", theme::error("Failed:"), name, e),
            },
            Err(e) => println!("{} {}: {}", theme::error("Failed:"), name, e),
        }
    }

    Ok(())
}
