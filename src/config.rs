use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SyncStrategy {
    #[default]
    Rebase,
    Merge,
}

#[derive(Serialize, Clone)]
pub struct Config {
    pub base_branch: String,
    pub protected_branches: Vec<String>,
    pub stale_days: u64,
    pub review_secrets_patterns: Vec<String>,
    pub sync_strategy: SyncStrategy,
    pub log_limit: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_branch: "main".to_string(),
            protected_branches: vec![
                "main".to_string(),
                "master".to_string(),
                "develop".to_string(),
            ],
            stale_days: 30,
            review_secrets_patterns: vec![
                r"AWS_SECRET".to_string(),
                r"api_key\s*=".to_string(),
                r"-----BEGIN".to_string(),
                r"ghp_[A-Za-z0-9]+".to_string(),
                r"password\s*=".to_string(),
            ],
            sync_strategy: SyncStrategy::default(),
            log_limit: 10_000,
        }
    }
}

#[derive(Deserialize)]
struct FileConfig {
    base_branch: Option<String>,
    protected_branches: Option<Vec<String>>,
    stale_days: Option<u64>,
    review_secrets_patterns: Option<Vec<String>>,
    sync_strategy: Option<SyncStrategy>,
    log_limit: Option<usize>,
}

fn global_config_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("gitpilot")
            .join("config.toml")
    })
}

fn apply(base: &mut Config, file: FileConfig) {
    if let Some(v) = file.base_branch {
        base.base_branch = v;
    }
    if let Some(v) = file.protected_branches {
        base.protected_branches = v;
    }
    if let Some(v) = file.stale_days {
        base.stale_days = v;
    }
    if let Some(v) = file.review_secrets_patterns {
        base.review_secrets_patterns = v;
    }
    if let Some(v) = file.sync_strategy {
        base.sync_strategy = v;
    }
    if let Some(v) = file.log_limit {
        base.log_limit = v;
    }
}

fn load_file(path: &PathBuf) -> Result<Option<FileConfig>> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file_config: FileConfig = toml::from_str(&contents)
        .with_context(|| format!("malformed TOML in {}", path.display()))?;
    Ok(Some(file_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_base_branch_is_main() {
        assert_eq!(Config::default().base_branch, "main");
    }

    #[test]
    fn default_protected_branches() {
        let c = Config::default();
        assert!(c.protected_branches.contains(&"main".to_string()));
        assert!(c.protected_branches.contains(&"master".to_string()));
        assert!(c.protected_branches.contains(&"develop".to_string()));
    }

    #[test]
    fn default_stale_days_is_30() {
        assert_eq!(Config::default().stale_days, 30);
    }

    #[test]
    fn default_review_patterns_include_known_secrets() {
        let c = Config::default();
        assert!(c.review_secrets_patterns.iter().any(|p| p.contains("AWS_SECRET")));
        assert!(c.review_secrets_patterns.iter().any(|p| p.contains("password")));
        assert!(c.review_secrets_patterns.iter().any(|p| p.contains("ghp_")));
    }

    #[test]
    fn apply_overrides_base_branch() {
        let mut c = Config::default();
        apply(&mut c, FileConfig {
            base_branch: Some("develop".to_string()),
            protected_branches: None,
            stale_days: None,
            review_secrets_patterns: None,
            sync_strategy: None,
            log_limit: None,
        });
        assert_eq!(c.base_branch, "develop");
    }

    #[test]
    fn apply_overrides_stale_days() {
        let mut c = Config::default();
        apply(&mut c, FileConfig {
            base_branch: None,
            protected_branches: None,
            stale_days: Some(60),
            review_secrets_patterns: None,
            sync_strategy: None,
            log_limit: None,
        });
        assert_eq!(c.stale_days, 60);
    }

    #[test]
    fn apply_overrides_protected_branches() {
        let mut c = Config::default();
        apply(&mut c, FileConfig {
            base_branch: None,
            protected_branches: Some(vec!["trunk".to_string()]),
            stale_days: None,
            review_secrets_patterns: None,
            sync_strategy: None,
            log_limit: None,
        });
        assert_eq!(c.protected_branches, vec!["trunk"]);
    }

    #[test]
    fn apply_none_fields_leave_defaults_unchanged() {
        let mut c = Config::default();
        let original_stale = c.stale_days;
        let original_base = c.base_branch.clone();
        apply(&mut c, FileConfig {
            base_branch: None,
            protected_branches: None,
            stale_days: None,
            review_secrets_patterns: None,
            sync_strategy: None,
            log_limit: None,
        });
        assert_eq!(c.base_branch, original_base);
        assert_eq!(c.stale_days, original_stale);
    }

    #[test]
    fn load_file_returns_none_for_missing_path() {
        let path = std::path::PathBuf::from("/tmp/gitpilot_nonexistent_test_config_xyz.toml");
        let result = load_file(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn load_file_parses_base_branch() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"base_branch = "develop""#).unwrap();
        let result = load_file(&f.path().to_path_buf()).unwrap().unwrap();
        assert_eq!(result.base_branch, Some("develop".to_string()));
    }

    #[test]
    fn load_file_parses_stale_days() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "stale_days = 90").unwrap();
        let result = load_file(&f.path().to_path_buf()).unwrap().unwrap();
        assert_eq!(result.stale_days, Some(90));
    }

    #[test]
    fn load_file_errors_on_invalid_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "not = valid = toml!!!").unwrap();
        let result = load_file(&f.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn load_file_parses_sync_strategy_merge() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, r#"sync_strategy = "merge""#).unwrap();
        let result = load_file(&f.path().to_path_buf()).unwrap().unwrap();
        assert!(matches!(result.sync_strategy, Some(SyncStrategy::Merge)));
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        if let Some(global_path) = global_config_path() {
            if let Some(file_config) = load_file(&global_path)? {
                apply(&mut config, file_config);
            }
        }

        let local_path = std::env::current_dir()?.join(".gitpilot.toml");
        if let Some(file_config) = load_file(&local_path)? {
            apply(&mut config, file_config);
        }

        Ok(config)
    }
}
