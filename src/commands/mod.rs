pub mod cleanup;
pub mod clone;
pub mod completions;
pub mod generate;
pub mod init;
pub mod log;
pub mod review;
pub mod stash;
pub mod status;
pub mod summary;
pub mod switch;
pub mod sync;
pub mod undo;

use crate::config::Config;
use crate::git::RepoContext;

pub struct CommandContext {
    pub repo: RepoContext,
    pub config: Config,
    pub json: bool,
    #[allow(dead_code)]
    pub no_color: bool,
}
