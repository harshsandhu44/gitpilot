use chrono::{DateTime, Utc, TimeZone};
use crate::error::GitPilotError;
use super::RepoContext;

pub struct LogFilter<'a> {
    pub author: Option<&'a str>,
    pub since: Option<DateTime<Utc>>,
    pub grep: Option<&'a str>,
}

pub fn filtered(ctx: &RepoContext, count: usize, filter: &LogFilter) -> Result<Vec<CommitSummary>, GitPilotError> {
    let repo = &ctx.repo;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut results = Vec::new();
    let mut checked = 0usize;
    for oid in revwalk {
        if checked >= 10_000 || results.len() >= count {
            break;
        }
        checked += 1;
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let summary = commit_to_summary(&commit);

        if let Some(author) = filter.author {
            if !summary.author.to_lowercase().contains(&author.to_lowercase()) {
                continue;
            }
        }
        if let Some(since) = filter.since {
            if summary.date < since {
                continue;
            }
        }
        if let Some(grep) = filter.grep {
            if !summary.message.to_lowercase().contains(&grep.to_lowercase()) {
                continue;
            }
        }
        results.push(summary);
    }
    Ok(results)
}

#[derive(Debug)]
pub struct CommitSummary {
    pub short_id: String,
    pub full_id: git2::Oid,
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

pub fn commit_to_summary(commit: &git2::Commit) -> CommitSummary {
    let full_id = commit.id();
    let short_id = full_id.to_string().chars().take(7).collect();
    let message = commit.summary().unwrap_or("").to_string();
    let author = commit.author().name().unwrap_or("?").to_string();
    let timestamp = commit.time().seconds();
    let date = Utc.timestamp_opt(timestamp, 0).single().unwrap_or_else(Utc::now);
    CommitSummary { short_id, full_id, message, author, date }
}
