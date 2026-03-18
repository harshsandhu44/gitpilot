use chrono::{DateTime, Utc, TimeZone};
use crate::error::GitPilotError;
use super::RepoContext;

#[derive(Debug)]
pub struct CommitSummary {
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

pub fn recent(ctx: &RepoContext, count: usize) -> Result<Vec<CommitSummary>, GitPilotError> {
    let repo = &ctx.repo;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for (i, oid) in revwalk.enumerate() {
        if i >= count {
            break;
        }
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_summary(&commit));
    }
    Ok(commits)
}

pub fn range(ctx: &RepoContext, base_oid: git2::Oid, head_oid: git2::Oid) -> Result<Vec<CommitSummary>, GitPilotError> {
    let repo = &ctx.repo;
    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_oid)?;
    revwalk.hide(base_oid)?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        commits.push(commit_to_summary(&commit));
    }
    Ok(commits)
}

fn commit_to_summary(commit: &git2::Commit) -> CommitSummary {
    let short_id = commit
        .id()
        .to_string()
        .chars()
        .take(7)
        .collect();
    let message = commit
        .summary()
        .unwrap_or("")
        .to_string();
    let author = commit.author().name().unwrap_or("?").to_string();
    let timestamp = commit.time().seconds();
    let date = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(Utc::now);
    CommitSummary { short_id, message, author, date }
}
