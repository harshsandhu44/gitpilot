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

fn get_upstream_status(repo: &git2::Repository, branch: &str) -> Option<UpstreamStatus> {
    let local_branch = repo.find_branch(branch, git2::BranchType::Local).ok()?;
    let upstream = local_branch.upstream().ok()?;
    let local_oid = local_branch.get().target()?;
    let upstream_oid = upstream.get().target()?;
    let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid).ok()?;
    Some(UpstreamStatus { ahead, behind })
}
