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

#[cfg(test)]
mod tests {
    use super::*;

    static DIR_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn toml_template_contains_base_branch() {
        assert!(TOML_TEMPLATE.contains("base_branch"));
    }

    #[test]
    fn toml_template_contains_protected_branches() {
        assert!(TOML_TEMPLATE.contains("protected_branches"));
    }

    #[test]
    fn toml_template_contains_stale_days() {
        assert!(TOML_TEMPLATE.contains("stale_days"));
    }

    #[test]
    fn toml_template_contains_sync_strategy() {
        assert!(TOML_TEMPLATE.contains("sync_strategy"));
    }

    #[test]
    fn toml_template_contains_review_secrets_patterns() {
        assert!(TOML_TEMPLATE.contains("review_secrets_patterns"));
    }

    #[test]
    fn install_hook_creates_new_file_with_git_pilot_review() {
        let _guard = DIR_MUTEX.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".git").join("hooks")).unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = install_pre_commit_hook();
        std::env::set_current_dir(&original).unwrap();

        result.unwrap();
        let hook = std::fs::read_to_string(dir.path().join(".git/hooks/pre-commit")).unwrap();
        assert!(hook.contains("git pilot review"));
        assert!(hook.starts_with("#!/bin/sh"));
    }

    #[test]
    fn install_hook_appends_to_existing_file() {
        let _guard = DIR_MUTEX.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let hooks_dir = dir.path().join(".git").join("hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        let hook_path = hooks_dir.join("pre-commit");
        std::fs::write(&hook_path, "#!/bin/sh\necho 'existing hook'\n").unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = install_pre_commit_hook();
        std::env::set_current_dir(&original).unwrap();

        result.unwrap();
        let hook = std::fs::read_to_string(&hook_path).unwrap();
        assert!(hook.contains("existing hook"));
        assert!(hook.contains("git pilot review"));
    }

    #[test]
    fn install_hook_skips_if_already_present() {
        let _guard = DIR_MUTEX.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let hooks_dir = dir.path().join(".git").join("hooks");
        std::fs::create_dir_all(&hooks_dir).unwrap();
        let hook_path = hooks_dir.join("pre-commit");
        std::fs::write(&hook_path, "#!/bin/sh\ngit pilot review\n").unwrap();

        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = install_pre_commit_hook();
        std::env::set_current_dir(&original).unwrap();

        result.unwrap();
        // Content should not have been duplicated
        let hook = std::fs::read_to_string(&hook_path).unwrap();
        assert_eq!(hook.matches("git pilot review").count(), 1);
    }

    #[test]
    fn install_hook_fails_without_git_dir() {
        let _guard = DIR_MUTEX.lock().unwrap();
        let dir = tempfile::TempDir::new().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = install_pre_commit_hook();
        std::env::set_current_dir(&original).unwrap();

        assert!(result.is_err());
    }
}

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
    let hook_line = "git pilot review\n";

    if hook_path.exists() {
        let existing = std::fs::read_to_string(&hook_path)?;
        if existing.contains("git pilot review") {
            println!("{}", theme::dim("pre-commit hook already contains git pilot review — skipping."));
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
