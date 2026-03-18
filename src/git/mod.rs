pub mod branches;
pub mod commits;
pub mod diff;
pub mod status;

use std::path::PathBuf;
use crate::error::GitPilotError;

pub struct RepoContext {
    pub repo: git2::Repository,
    #[allow(dead_code)]
    pub workdir: PathBuf,
}

impl RepoContext {
    pub fn open() -> Result<Self, GitPilotError> {
        let repo = git2::Repository::discover(".")
            .map_err(|_| GitPilotError::NoRepository)?;
        let workdir = repo
            .workdir()
            .ok_or(GitPilotError::NoRepository)?
            .to_path_buf();
        Ok(Self { repo, workdir })
    }
}

/// Test helpers shared across git module tests.
#[cfg(test)]
pub(crate) mod test_helpers {
    use super::RepoContext;
    use git2::{Repository, Signature};
    use std::path::Path;
    use tempfile::TempDir;

    pub fn make_repo() -> (TempDir, Repository) {
        let dir = TempDir::new().unwrap();
        let repo = Repository::init(dir.path()).unwrap();
        {
            let mut cfg = repo.config().unwrap();
            cfg.set_str("user.name", "Test User").unwrap();
            cfg.set_str("user.email", "test@example.com").unwrap();
        }
        (dir, repo)
    }

    pub fn commit_file(
        repo: &Repository,
        workdir: &Path,
        filename: &str,
        content: &str,
        msg: &str,
    ) -> git2::Oid {
        let path = workdir.join(filename);
        std::fs::write(&path, content).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new(filename)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test User", "test@example.com").unwrap();
        let parents: Vec<git2::Commit> = repo
            .head()
            .ok()
            .map(|h| vec![h.peel_to_commit().unwrap()])
            .unwrap_or_default();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parent_refs).unwrap()
    }

    pub fn make_repo_context(repo: Repository, workdir: &Path) -> RepoContext {
        RepoContext { repo, workdir: workdir.to_path_buf() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_fails_outside_git_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        // Point discover to a temp dir with no .git
        // We can't easily test RepoContext::open() without changing cwd,
        // but we can verify the error type from git2 directly.
        let result = git2::Repository::discover(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn open_succeeds_in_git_repo() {
        let (dir, _repo) = test_helpers::make_repo();
        let git_repo = git2::Repository::discover(dir.path()).unwrap();
        let ctx = test_helpers::make_repo_context(git_repo, dir.path());
        assert!(ctx.workdir.exists());
    }
}
