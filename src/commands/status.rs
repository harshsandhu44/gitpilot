use anyhow::Result;
use crate::commands::CommandContext;
use crate::display::{tables, theme};
use crate::git::{commits, status};

pub fn run(ctx: &CommandContext) -> Result<()> {
    let repo_status = status::get_repo_status(&ctx.repo)?;

    // Branch header
    println!(
        "{} {}",
        theme::heading("Branch:"),
        theme::info(&repo_status.branch)
    );

    // Upstream
    if let Some(ref upstream) = repo_status.upstream {
        let msg = if upstream.ahead == 0 && upstream.behind == 0 {
            theme::success("up to date with upstream")
        } else {
            theme::warning(&format!(
                "↑{} ahead, ↓{} behind upstream",
                upstream.ahead, upstream.behind
            ))
        };
        println!("  {}", msg);
    } else {
        println!("  {}", theme::dim("no upstream tracking branch"));
    }

    println!();

    // Staged
    if !repo_status.staged.is_empty() {
        println!("{}", theme::heading("Staged changes:"));
        println!("{}", tables::file_table(&repo_status.staged));
        println!();
    }

    // Unstaged
    if !repo_status.unstaged.is_empty() {
        println!("{}", theme::heading("Unstaged changes:"));
        println!("{}", tables::file_table(&repo_status.unstaged));
        println!();
    }

    // Untracked
    if !repo_status.untracked.is_empty() {
        println!("{}", theme::heading("Untracked files:"));
        println!("{}", tables::file_table(&repo_status.untracked));
        println!();
    }

    if repo_status.staged.is_empty() && repo_status.unstaged.is_empty() && repo_status.untracked.is_empty() {
        println!("{}", theme::success("Working tree clean"));
        println!();
    }

    // Stash
    if repo_status.stash_count > 0 {
        println!(
            "{}",
            theme::warning(&format!("Stash: {} entr{}", repo_status.stash_count,
                if repo_status.stash_count == 1 { "y" } else { "ies" }))
        );
        println!();
    }

    // Recent commits
    let commits = commits::recent(&ctx.repo, 5)?;
    if !commits.is_empty() {
        println!("{}", theme::heading("Recent commits:"));
        println!("{}", tables::commit_table(&commits));
    }

    Ok(())
}
