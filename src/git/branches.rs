use chrono::{DateTime, Utc, TimeZone};
use git2::BranchType;
use crate::error::GitPilotError;
use super::RepoContext;

#[derive(Debug, Clone, PartialEq)]
pub enum BranchState {
    Merged,
    Gone,
    Stale,
    Active,
}

#[derive(Debug)]
pub struct BranchInfo {
    pub name: String,
    pub state: BranchState,
    #[allow(dead_code)]
    pub last_commit_date: DateTime<Utc>,
    pub last_commit_msg: String,
    pub age_days: i64,
}

pub fn list_branches(ctx: &RepoContext, base_branch: &str, stale_days: u64) -> Result<Vec<BranchInfo>, GitPilotError> {
    let repo = &ctx.repo;
    let now = Utc::now();

    let base_ref = format!("refs/remotes/origin/{}", base_branch);
    let base_oid = repo
        .revparse_single(&base_ref)
        .or_else(|_| repo.revparse_single(base_branch))
        .ok()
        .and_then(|o| o.peel_to_commit().ok())
        .map(|c| c.id());

    let branches = repo.branches(Some(BranchType::Local))?;
    let mut result = Vec::new();

    for branch in branches {
        let (branch, _) = branch?;
        let name = branch.name()?.unwrap_or("?").to_string();

        let commit = branch.get().peel_to_commit()?;
        let timestamp = commit.time().seconds();
        let last_commit_date = Utc.timestamp_opt(timestamp, 0).single().unwrap_or(now);
        let last_commit_msg = commit.summary().unwrap_or("").to_string();
        let age_days = (now - last_commit_date).num_days();

        // Check if gone (upstream tracking ref no longer exists)
        let state = if is_gone(&branch, repo) {
            BranchState::Gone
        } else if let Some(base) = base_oid {
            if is_merged(repo, commit.id(), base) {
                BranchState::Merged
            } else if age_days > stale_days as i64 {
                BranchState::Stale
            } else {
                BranchState::Active
            }
        } else if age_days > stale_days as i64 {
            BranchState::Stale
        } else {
            BranchState::Active
        };

        result.push(BranchInfo {
            name,
            state,
            last_commit_date,
            last_commit_msg,
            age_days,
        });
    }

    Ok(result)
}

fn is_gone(branch: &git2::Branch, repo: &git2::Repository) -> bool {
    if let Ok(upstream) = branch.upstream() {
        let upstream_ref = upstream.get().name().unwrap_or("").to_string();
        repo.find_reference(&upstream_ref).is_err()
    } else {
        false
    }
}

fn is_merged(repo: &git2::Repository, branch_tip: git2::Oid, base_tip: git2::Oid) -> bool {
    // A branch is merged if its tip is an ancestor of base
    if let Ok(merge_base) = repo.merge_base(branch_tip, base_tip) {
        merge_base == branch_tip
    } else {
        false
    }
}
