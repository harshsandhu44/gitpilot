use chrono::{DateTime, Utc, TimeZone};
use crate::error::GitPilotError;
use super::RepoContext;

pub struct LogFilter<'a> {
    pub author: Option<&'a str>,
    pub since: Option<DateTime<Utc>>,
    pub grep: Option<&'a str>,
}

pub fn filtered(ctx: &RepoContext, count: usize, scan_limit: usize, filter: &LogFilter) -> Result<Vec<CommitSummary>, GitPilotError> {
    let repo = &ctx.repo;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut results = Vec::new();
    let mut checked = 0usize;
    for oid in revwalk {
        if checked >= scan_limit || results.len() >= count {
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_helpers::{commit_file, make_repo, make_repo_context};
    use chrono::Duration;

    #[test]
    fn recent_returns_correct_count() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "first commit");
        commit_file(&repo, dir.path(), "b.txt", "b", "second commit");
        commit_file(&repo, dir.path(), "c.txt", "c", "third commit");
        let ctx = make_repo_context(repo, dir.path());
        let commits = recent(&ctx, 2).unwrap();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn recent_returns_all_when_count_exceeds_total() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "only commit");
        let ctx = make_repo_context(repo, dir.path());
        let commits = recent(&ctx, 100).unwrap();
        assert_eq!(commits.len(), 1);
    }

    #[test]
    fn recent_commits_are_newest_first() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "first");
        commit_file(&repo, dir.path(), "b.txt", "b", "second");
        let ctx = make_repo_context(repo, dir.path());
        let commits = recent(&ctx, 2).unwrap();
        assert_eq!(commits[0].message, "second");
        assert_eq!(commits[1].message, "first");
    }

    #[test]
    fn recent_commit_summary_fields() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "hello", "test message");
        let ctx = make_repo_context(repo, dir.path());
        let commits = recent(&ctx, 1).unwrap();
        assert_eq!(commits[0].message, "test message");
        assert_eq!(commits[0].author, "Test User");
        assert_eq!(commits[0].short_id.len(), 7);
    }

    #[test]
    fn filtered_by_author() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "from alice");
        commit_file(&repo, dir.path(), "b.txt", "b", "from bob");
        let ctx = make_repo_context(repo, dir.path());
        let filter = LogFilter { author: Some("Test User"), since: None, grep: None };
        let commits = filtered(&ctx, 10, 10_000, &filter).unwrap();
        assert_eq!(commits.len(), 2);

        // Filter by non-existent author
        let filter2 = LogFilter { author: Some("nobody"), since: None, grep: None };
        let commits2 = filtered(&ctx, 10, 10_000, &filter2).unwrap();
        assert_eq!(commits2.len(), 0);
    }

    #[test]
    fn filtered_by_grep() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "feat: add widget");
        commit_file(&repo, dir.path(), "b.txt", "b", "fix: repair widget");
        commit_file(&repo, dir.path(), "c.txt", "c", "chore: cleanup");
        let ctx = make_repo_context(repo, dir.path());

        let filter = LogFilter { author: None, since: None, grep: Some("widget") };
        let commits = filtered(&ctx, 10, 10_000, &filter).unwrap();
        assert_eq!(commits.len(), 2);
    }

    #[test]
    fn filtered_by_since_excludes_old_commits() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "recent commit");
        let ctx = make_repo_context(repo, dir.path());

        // Since one year in the future — should return nothing
        let future = Utc::now() + Duration::days(365);
        let filter = LogFilter { author: None, since: Some(future), grep: None };
        let commits = filtered(&ctx, 10, 10_000, &filter).unwrap();
        assert_eq!(commits.len(), 0);

        // Since one year ago — should return the commit
        let past = Utc::now() - Duration::days(365);
        let filter2 = LogFilter { author: None, since: Some(past), grep: None };
        let commits2 = filtered(&ctx, 10, 10_000, &filter2).unwrap();
        assert_eq!(commits2.len(), 1);
    }

    #[test]
    fn range_returns_commits_between_oids() {
        let (dir, repo) = make_repo();
        let base_oid = commit_file(&repo, dir.path(), "a.txt", "a", "base");
        commit_file(&repo, dir.path(), "b.txt", "b", "on branch");
        let head_oid = commit_file(&repo, dir.path(), "c.txt", "c", "also on branch");
        let ctx = make_repo_context(repo, dir.path());

        let commits = range(&ctx, base_oid, head_oid).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].message, "also on branch");
        assert_eq!(commits[1].message, "on branch");
    }

    #[test]
    fn commit_to_summary_short_id_is_7_chars() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "test");
        let head = repo.head().unwrap().peel_to_commit().unwrap();
        let summary = commit_to_summary(&head);
        assert_eq!(summary.short_id.len(), 7);
        assert_eq!(summary.message, "test");
        assert_eq!(summary.author, "Test User");
    }
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
