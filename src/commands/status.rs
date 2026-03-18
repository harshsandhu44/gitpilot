use anyhow::Result;
use serde::Serialize;
use crate::commands::CommandContext;
use crate::display::{tables, theme};
use crate::git::{commits, status};

#[derive(Serialize)]
struct FileJson {
    path: String,
    status: String,
}

#[derive(Serialize)]
struct UpstreamJson {
    ahead: usize,
    behind: usize,
}

#[derive(Serialize)]
struct CommitJson {
    hash: String,
    author: String,
    date: String,
    message: String,
}

#[derive(Serialize)]
struct StatusJson {
    branch: String,
    upstream: Option<UpstreamJson>,
    staged: Vec<FileJson>,
    unstaged: Vec<FileJson>,
    untracked: Vec<FileJson>,
    stash_count: usize,
    recent_commits: Vec<CommitJson>,
}

pub fn run(ctx: &CommandContext) -> Result<()> {
    let repo_status = status::get_repo_status(&ctx.repo)?;
    let commits = commits::recent(&ctx.repo, 5)?;

    if ctx.json {
        let json = StatusJson {
            branch: repo_status.branch.clone(),
            upstream: repo_status.upstream.as_ref().map(|u| UpstreamJson {
                ahead: u.ahead,
                behind: u.behind,
            }),
            staged: repo_status.staged.iter().map(|f| FileJson { path: f.path.clone(), status: f.status.clone() }).collect(),
            unstaged: repo_status.unstaged.iter().map(|f| FileJson { path: f.path.clone(), status: f.status.clone() }).collect(),
            untracked: repo_status.untracked.iter().map(|f| FileJson { path: f.path.clone(), status: f.status.clone() }).collect(),
            stash_count: repo_status.stash_count,
            recent_commits: commits.iter().map(|c| CommitJson {
                hash: c.short_id.clone(),
                author: c.author.clone(),
                date: c.date.to_rfc3339(),
                message: c.message.clone(),
            }).collect(),
        };
        println!("{}", serde_json::to_string(&json)?);
        return Ok(());
    }

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
    if !commits.is_empty() {
        println!("{}", theme::heading("Recent commits:"));
        println!("{}", tables::commit_table(&commits));
    }

    Ok(())
}
