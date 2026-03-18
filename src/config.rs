use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct Config {
    pub base_branch: String,
    pub protected_branches: Vec<String>,
    pub stale_days: u64,
    pub review_secrets_patterns: Vec<String>,
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
        }
    }
}

#[derive(Deserialize)]
struct FileConfig {
    base_branch: Option<String>,
    protected_branches: Option<Vec<String>>,
    stale_days: Option<u64>,
    review_secrets_patterns: Option<Vec<String>>,
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
