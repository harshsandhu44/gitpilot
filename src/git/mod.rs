pub mod branches;
pub mod commits;
pub mod diff;
pub mod status;

use std::path::PathBuf;
use crate::error::GitPilotError;

pub struct RepoContext {
    pub repo: git2::Repository,
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
