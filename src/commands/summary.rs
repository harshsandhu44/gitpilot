use anyhow::Result;
use crate::commands::CommandContext;
use crate::display::{tables, theme};
use crate::git::{commits, diff};

pub fn run(ctx: &CommandContext, base: Option<&str>) -> Result<()> {
    let base_branch = base
        .unwrap_or(&ctx.config.base_branch)
        .to_string();

    println!(
        "{} {} vs {}",
        theme::heading("Summary:"),
        theme::info(
            ctx.repo
                .repo
                .head()
                .ok()
                .and_then(|h| h.shorthand().map(|s| s.to_string()))
                .unwrap_or_else(|| "HEAD".to_string())
                .as_str()
        ),
        theme::dim(&base_branch)
    );
    println!();

    let diff_summary = diff::diff_vs_base(&ctx.repo, &base_branch)?;

    println!(
        "{} +{} -{} across {} file{}",
        theme::heading("Changes:"),
        theme::success(&diff_summary.total_additions.to_string()),
        theme::error(&diff_summary.total_deletions.to_string()),
        diff_summary.files.len(),
        if diff_summary.files.len() == 1 { "" } else { "s" }
    );
    println!();

    if !diff_summary.files.is_empty() {
        println!("{}", theme::heading("Files changed:"));
        for f in &diff_summary.files {
            println!(
                "  {} +{} -{}",
                f.path,
                theme::success(&f.additions.to_string()),
                theme::error(&f.deletions.to_string()),
            );
        }
        println!();
    }

    // Get commits on this branch
    let repo = &ctx.repo.repo;
    let base_ref = format!("refs/remotes/origin/{}", base_branch);
    let base_obj = repo
        .revparse_single(&base_ref)
        .or_else(|_| repo.revparse_single(&base_branch));

    if let Ok(base_obj) = base_obj {
        if let (Ok(head_commit), Ok(base_commit)) = (
            repo.head().and_then(|h| h.peel_to_commit()),
            base_obj.peel_to_commit(),
        ) {
            if let Ok(merge_base) = repo.merge_base(head_commit.id(), base_commit.id()) {
                let branch_commits = commits::range(
                    &ctx.repo,
                    merge_base,
                    head_commit.id(),
                )?;
                if !branch_commits.is_empty() {
                    println!(
                        "{} ({} commit{})",
                        theme::heading("Commits on this branch:"),
                        branch_commits.len(),
                        if branch_commits.len() == 1 { "" } else { "s" }
                    );
                    println!("{}", tables::commit_table(&branch_commits));
                }
            }
        }
    }

    Ok(())
}
