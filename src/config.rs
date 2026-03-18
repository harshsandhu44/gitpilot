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
