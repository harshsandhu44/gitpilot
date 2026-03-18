use thiserror::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_repository_message() {
        assert_eq!(GitPilotError::NoRepository.to_string(), "Not inside a git repository");
    }

    #[test]
    fn no_branch_message() {
        assert_eq!(
            GitPilotError::NoBranch("feat/foo".to_string()).to_string(),
            "Branch not found: feat/foo"
        );
    }

    #[test]
    fn no_current_branch_message() {
        assert_eq!(
            GitPilotError::NoCurrentBranch.to_string(),
            "Could not determine current branch"
        );
    }

    #[test]
    fn git_error_wraps_git2() {
        let git_err = git2::Error::from_str("some git error");
        let e = GitPilotError::Git(git_err);
        assert!(e.to_string().contains("Git error"));
    }

    #[test]
    fn io_error_wraps_std_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let e = GitPilotError::Io(io_err);
        assert!(e.to_string().contains("IO error"));
    }
}

#[derive(Error, Debug)]
pub enum GitPilotError {
    #[error("Not inside a git repository")]
    NoRepository,
    #[allow(dead_code)]
    #[error("Branch not found: {0}")]
    NoBranch(String),
    #[allow(dead_code)]
    #[error("Could not determine current branch")]
    NoCurrentBranch,
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
