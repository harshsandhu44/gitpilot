use anyhow::{anyhow, Result};
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use git2::BranchType;
use serde::Serialize;
use crate::commands::CommandContext;
use crate::display::theme;

#[derive(Serialize)]
struct SwitchJson {
    branch: String,
    action: String,
}

pub fn run(ctx: &CommandContext, include_remote: bool) -> Result<()> {
    let repo = &ctx.repo.repo;

    let current = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut branch_names: Vec<String> = Vec::new();

    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        if let Some(name) = branch.name()? {
            let label = if name == current {
                format!("* {}", name)
            } else {
                name.to_string()
            };
            branch_names.push(label);
        }
    }

    if include_remote {
        for branch in repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                let local_name = name.strip_prefix("origin/").unwrap_or(name);
                if local_name != "HEAD" && local_name != current && !branch_names.contains(&local_name.to_string()) {
                    branch_names.push(local_name.to_string());
                }
            }
        }
    }

    if branch_names.is_empty() {
        println!("{}", theme::dim("No branches available."));
        return Ok(());
    }

    if !dialoguer::console::Term::stdout().is_term() {
        return Err(anyhow!("switch requires an interactive terminal"));
    }

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Switch to branch")
        .items(&branch_names)
        .interact()?;

    let raw = &branch_names[selection];
    let name = raw.strip_prefix("* ").unwrap_or(raw);

    if name == current {
        if !ctx.json {
            println!("{}", theme::dim(&format!("Already on '{}'", name)));
        }
        return Ok(());
    }

    // Try to find existing local branch
    let checkout_result = if repo.find_branch(name, BranchType::Local).is_ok() {
        checkout_local(repo, name)
    } else if include_remote {
        // Create local tracking branch from remote
        create_and_checkout(repo, name)
    } else {
        checkout_local(repo, name)
    };

    match checkout_result {
        Ok(()) => {
            if ctx.json {
                println!("{}", serde_json::to_string(&SwitchJson {
                    branch: name.to_string(),
                    action: "switched".to_string(),
                })?);
            } else {
                println!("{} {}", theme::success("Switched to"), name);
            }
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("unstaged") || msg.contains("overwritten") || msg.contains("local changes") {
                return Err(anyhow!("Cannot switch: uncommitted changes. Commit or stash first."));
            }
            return Err(e);
        }
    }

    Ok(())
}

fn checkout_local(repo: &git2::Repository, name: &str) -> Result<()> {
    let obj = repo.revparse_single(&format!("refs/heads/{}", name))?;
    repo.checkout_tree(&obj, None)?;
    repo.set_head(&format!("refs/heads/{}", name))?;
    Ok(())
}

fn create_and_checkout(repo: &git2::Repository, name: &str) -> Result<()> {
    let remote_ref = format!("refs/remotes/origin/{}", name);
    let obj = repo.revparse_single(&remote_ref)?;
    let commit = obj.peel_to_commit()?;
    let mut branch = repo.branch(name, &commit, false)?;
    branch.set_upstream(Some(&format!("origin/{}", name)))?;
    checkout_local(repo, name)
}
