use anyhow::{anyhow, Result};
use dialoguer::{Confirm, theme::ColorfulTheme};
use crate::config::Config;
use crate::display::theme;

const TOML_TEMPLATE: &str = r#"# gitpilot configuration
# See https://github.com/harshsandhu44/gitpilot for documentation

# Base branch for comparisons (summary, cleanup, sync)
# base_branch = "main"

# Branches that will never be deleted by cleanup
# protected_branches = ["main", "master", "develop"]

# Branches older than this (in days) are considered stale
# stale_days = 30

# Regex patterns to flag as potential secrets in review
# review_secrets_patterns = [
#   "AWS_SECRET",
#   "api_key\\s*=",
#   "-----BEGIN",
#   "ghp_[A-Za-z0-9]+",
#   "password\\s*=",
# ]

# Sync strategy: "rebase" or "merge"
# sync_strategy = "rebase"
"#;

pub fn run(_config: &Config, install_hook: bool) -> Result<()> {
    let path = std::env::current_dir()?.join(".gitpilot.toml");

    if path.exists() {
        let overwrite = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(".gitpilot.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            println!("{}", theme::dim("Aborted."));
            return Ok(());
        }
    }

    std::fs::write(&path, TOML_TEMPLATE)?;
    println!("{} .gitpilot.toml", theme::success("Created"));

    if install_hook {
        install_pre_commit_hook()?;
    }

    Ok(())
}

fn install_pre_commit_hook() -> Result<()> {
    let hook_dir = std::env::current_dir()?.join(".git").join("hooks");
    if !hook_dir.exists() {
        return Err(anyhow!("No .git/hooks directory found. Are you in a git repository?"));
    }

    let hook_path = hook_dir.join("pre-commit");
    let hook_line = "gitpilot review\n";

    if hook_path.exists() {
        let existing = std::fs::read_to_string(&hook_path)?;
        if existing.contains("gitpilot review") {
            println!("{}", theme::dim("pre-commit hook already contains gitpilot review — skipping."));
            return Ok(());
        }
        let mut content = existing;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(hook_line);
        std::fs::write(&hook_path, content)?;
    } else {
        std::fs::write(&hook_path, format!("#!/bin/sh\n{}", hook_line))?;
    }

    // Set executable permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("{} .git/hooks/pre-commit", theme::success("Installed hook"));
    Ok(())
}
