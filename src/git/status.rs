use git2::Status;
use crate::error::GitPilotError;
use super::RepoContext;

#[derive(Debug)]
pub struct FileChange {
    pub path: String,
    pub status: String,
}

#[derive(Debug)]
pub struct UpstreamStatus {
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug)]
pub struct RepoStatus {
    pub branch: String,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<FileChange>,
    pub stash_count: usize,
    pub upstream: Option<UpstreamStatus>,
}

pub fn get_repo_status(ctx: &RepoContext) -> Result<RepoStatus, GitPilotError> {
    let repo = &ctx.repo;

    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "HEAD (detached)".to_string());

    let statuses = repo.statuses(None)?;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for entry in statuses.iter() {
        let path = entry.path().unwrap_or("?").to_string();
        let flags = entry.status();

        if flags.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            let status = if flags.contains(Status::INDEX_NEW) {
                "added"
            } else if flags.contains(Status::INDEX_DELETED) {
                "deleted"
            } else if flags.contains(Status::INDEX_RENAMED) {
                "renamed"
            } else {
                "modified"
            };
            staged.push(FileChange { path: path.clone(), status: status.to_string() });
        }

        if flags.intersects(
            Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_TYPECHANGE
                | Status::WT_RENAMED,
        ) {
            let status = if flags.contains(Status::WT_DELETED) {
                "deleted"
            } else if flags.contains(Status::WT_RENAMED) {
                "renamed"
            } else {
                "modified"
            };
            unstaged.push(FileChange { path: path.clone(), status: status.to_string() });
        }

        if flags.contains(Status::WT_NEW) {
            untracked.push(FileChange { path, status: "untracked".to_string() });
        }
    }

    // Count stash entries via reflog on refs/stash
    let stash_count = repo
        .reflog("refs/stash")
        .map(|log| log.len())
        .unwrap_or(0);

    let upstream = get_upstream_status(repo, &branch);

    Ok(RepoStatus {
        branch,
        staged,
        unstaged,
        untracked,
        stash_count,
        upstream,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_helpers::{commit_file, make_repo, make_repo_context};

    #[test]
    fn get_repo_status_returns_branch_name() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "initial");
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        // After first commit on init, branch is typically "master" or "main"
        assert!(!status.branch.is_empty());
        assert!(status.branch == "master" || status.branch == "main");
    }

    #[test]
    fn get_repo_status_clean_repo_has_no_changes() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "initial");
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert!(status.staged.is_empty());
        assert!(status.unstaged.is_empty());
        assert!(status.untracked.is_empty());
    }

    #[test]
    fn get_repo_status_detects_untracked_file() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "initial");
        // Write a file without staging it
        std::fs::write(dir.path().join("new.txt"), "hello").unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert!(status.untracked.iter().any(|f| f.path == "new.txt"));
    }

    #[test]
    fn get_repo_status_detects_staged_new_file() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "initial");
        // Stage a new file without committing
        let new_path = dir.path().join("staged.txt");
        std::fs::write(&new_path, "staged content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert!(status.staged.iter().any(|f| f.path == "staged.txt"));
        assert!(status.staged.iter().any(|f| f.status == "added"));
    }

    #[test]
    fn get_repo_status_detects_staged_modified_file() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "original", "initial");
        // Modify and stage
        std::fs::write(dir.path().join("a.txt"), "modified").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("a.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert!(status.staged.iter().any(|f| f.path == "a.txt" && f.status == "modified"));
    }

    #[test]
    fn get_repo_status_detects_unstaged_modification() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "original", "initial");
        // Modify without staging
        std::fs::write(dir.path().join("a.txt"), "changed but not staged").unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert!(status.unstaged.iter().any(|f| f.path == "a.txt"));
    }

    #[test]
    fn get_repo_status_no_stash_by_default() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "a", "initial");
        let ctx = make_repo_context(repo, dir.path());
        let status = get_repo_status(&ctx).unwrap();
        assert_eq!(status.stash_count, 0);
    }
}

fn get_upstream_status(repo: &git2::Repository, branch: &str) -> Option<UpstreamStatus> {
    let local_branch = repo.find_branch(branch, git2::BranchType::Local).ok()?;
    let upstream = local_branch.upstream().ok()?;
    let local_oid = local_branch.get().target()?;
    let upstream_oid = upstream.get().target()?;
    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;
    Some(UpstreamStatus { ahead, behind })
}
