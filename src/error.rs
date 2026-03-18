use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitPilotError {
    #[error("Not inside a git repository")]
    NoRepository,
    #[error("Branch not found: {0}")]
    NoBranch(String),
    #[error("Could not determine current branch")]
    NoCurrentBranch,
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
