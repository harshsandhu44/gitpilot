pub mod cleanup;
pub mod review;
pub mod status;
pub mod summary;

use crate::config::Config;
use crate::git::RepoContext;

pub struct CommandContext {
    pub repo: RepoContext,
    pub config: Config,
}
