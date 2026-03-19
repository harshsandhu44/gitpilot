use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;
use crate::config::Config;
use crate::display::theme;

/// Parse a repo spec into a (url, inferred_dest_name) pair.
/// Accepts:
///   - Full HTTPS/HTTP URLs: https://github.com/owner/repo[.git]
///   - SSH URLs:             git@github.com:owner/repo[.git]
///   - Shorthand:            owner/repo  → https://github.com/owner/repo
pub fn parse_repo_spec(spec: &str) -> (String, String) {
    if spec.starts_with("git@") || spec.starts_with("https://") || spec.starts_with("http://") {
        let name = infer_name_from_url(spec);
        (spec.to_string(), name)
    } else {
        // Treat as owner/repo shorthand
        let name = spec.split('/').last().unwrap_or(spec).trim_end_matches(".git").to_string();
        let url = format!("https://github.com/{}", spec);
        (url, name)
    }
}

fn infer_name_from_url(url: &str) -> String {
    // For SSH URLs like git@github.com:owner/repo.git
    // For HTTPS URLs like https://github.com/owner/repo.git
    let segment = if url.starts_with("git@") {
        url.split(':').last().unwrap_or(url)
    } else {
        url.split('/').last().unwrap_or(url)
    };
    // Strip the last path component from SSH colon-separated part
    let leaf = segment.split('/').last().unwrap_or(segment);
    leaf.trim_end_matches(".git").to_string()
}

pub fn detect_stack(dest: &Path) -> &'static str {
    if dest.join("Cargo.toml").exists() {
        "Rust"
    } else if dest.join("package.json").exists() {
        "Node.js"
    } else if dest.join("go.mod").exists() {
        "Go"
    } else if dest.join("pyproject.toml").exists() || dest.join("requirements.txt").exists() {
        "Python"
    } else if dest.join("pom.xml").exists() {
        "Java (Maven)"
    } else if dest.join("build.gradle").exists() {
        "Java (Gradle)"
    } else if dest.join("Gemfile").exists() {
        "Ruby"
    } else if dest.join("composer.json").exists() {
        "PHP"
    } else {
        "Unknown"
    }
}

pub fn detect_key_files(dest: &Path) -> Vec<String> {
    let candidates = [
        "README.md",
        "CONTRIBUTING.md",
        "Makefile",
        "Dockerfile",
        ".env.example",
        "CHANGELOG.md",
        ".github/workflows",
        ".github/CODEOWNERS",
    ];
    candidates
        .iter()
        .filter(|f| dest.join(f).exists())
        .map(|f| f.to_string())
        .collect()
}

#[derive(Serialize)]
struct CloneOutput {
    repo: String,
    destination: String,
    default_branch: String,
    remotes: Vec<String>,
    stack: String,
    key_files: Vec<String>,
    suggested_commands: Vec<String>,
}

pub fn run(
    _config: &Config,
    repo: &str,
    into: Option<&str>,
    branch: Option<&str>,
    depth: Option<u32>,
    json: bool,
    _no_color: bool,
) -> Result<()> {
    let (url, inferred_name) = parse_repo_spec(repo);
    let dest_name = into.unwrap_or(&inferred_name).to_string();

    // Build git clone args
    let mut args = vec!["clone".to_string(), url.clone()];
    if let Some(b) = branch {
        args.push("--branch".to_string());
        args.push(b.to_string());
    }
    if let Some(d) = depth {
        args.push("--depth".to_string());
        args.push(d.to_string());
    }
    args.push(dest_name.clone());

    let status = std::process::Command::new("git")
        .args(&args)
        .status()?;

    if !status.success() {
        bail!("git clone failed with exit code {}", status.code().unwrap_or(1));
    }

    let dest = Path::new(&dest_name);

    // Post-clone inspection
    let default_branch = detect_default_branch(dest);
    let remotes = detect_remotes(dest);
    let stack = detect_stack(dest).to_string();
    let key_files = detect_key_files(dest);

    let suggested_commands = vec![
        format!("cd {}", dest_name),
        "git pilot summary".to_string(),
        "git pilot log --count 15".to_string(),
        "git pilot init --hook".to_string(),
    ];

    if json {
        let out = CloneOutput {
            repo: repo.to_string(),
            destination: format!("./{}", dest_name),
            default_branch,
            remotes,
            stack,
            key_files,
            suggested_commands,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "{} {} into {}",
            theme::success("Cloned"),
            repo,
            theme::info(&format!("./{}", dest_name))
        );
        println!();
        println!("{}", theme::heading("Repo snapshot"));
        println!("- default branch: {}", default_branch);
        println!("- remotes: {}", if remotes.is_empty() { "none".to_string() } else { remotes.join(", ") });
        println!("- likely stack: {}", stack);
        if !key_files.is_empty() {
            println!("- key files: {}", key_files.join(", "));
        }
        println!("- suggested next commands:");
        for cmd in &suggested_commands {
            println!("  {}", theme::dim(cmd));
        }
    }

    Ok(())
}

fn detect_default_branch(dest: &Path) -> String {
    if let Ok(repo) = git2::Repository::open(dest) {
        if let Ok(head) = repo.head() {
            if let Some(name) = head.shorthand() {
                return name.to_string();
            }
        }
    }
    "unknown".to_string()
}

fn detect_remotes(dest: &Path) -> Vec<String> {
    if let Ok(repo) = git2::Repository::open(dest) {
        if let Ok(remotes) = repo.remotes() {
            return remotes.iter().flatten().map(|r| r.to_string()).collect();
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parse_https_url() {
        let (url, name) = parse_repo_spec("https://github.com/owner/repo.git");
        assert_eq!(url, "https://github.com/owner/repo.git");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_https_url_no_git_suffix() {
        let (url, name) = parse_repo_spec("https://github.com/owner/myrepo");
        assert_eq!(url, "https://github.com/owner/myrepo");
        assert_eq!(name, "myrepo");
    }

    #[test]
    fn parse_ssh_url() {
        let (url, name) = parse_repo_spec("git@github.com:owner/repo.git");
        assert_eq!(url, "git@github.com:owner/repo.git");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_shorthand() {
        let (url, name) = parse_repo_spec("owner/repo");
        assert_eq!(url, "https://github.com/owner/repo");
        assert_eq!(name, "repo");
    }

    #[test]
    fn parse_shorthand_with_git_suffix() {
        let (url, name) = parse_repo_spec("owner/repo.git");
        assert_eq!(url, "https://github.com/owner/repo.git");
        assert_eq!(name, "repo");
    }

    #[test]
    fn stack_detection_rust() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Rust");
    }

    #[test]
    fn stack_detection_node() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_stack(dir.path()), "Node.js");
    }

    #[test]
    fn stack_detection_go() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("go.mod"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Go");
    }

    #[test]
    fn stack_detection_python_pyproject() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Python");
    }

    #[test]
    fn stack_detection_python_requirements() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Python");
    }

    #[test]
    fn stack_detection_java_maven() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("pom.xml"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Java (Maven)");
    }

    #[test]
    fn stack_detection_java_gradle() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("build.gradle"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Java (Gradle)");
    }

    #[test]
    fn stack_detection_ruby() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Gemfile"), "").unwrap();
        assert_eq!(detect_stack(dir.path()), "Ruby");
    }

    #[test]
    fn stack_detection_php() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("composer.json"), "{}").unwrap();
        assert_eq!(detect_stack(dir.path()), "PHP");
    }

    #[test]
    fn stack_detection_unknown() {
        let dir = TempDir::new().unwrap();
        assert_eq!(detect_stack(dir.path()), "Unknown");
    }

    #[test]
    fn stack_detection_rust_takes_priority_over_node() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_stack(dir.path()), "Rust");
    }

    #[test]
    fn key_files_none_present() {
        let dir = TempDir::new().unwrap();
        let files = detect_key_files(dir.path());
        assert!(files.is_empty());
    }

    #[test]
    fn key_files_readme_and_makefile() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();
        fs::write(dir.path().join("Makefile"), "").unwrap();
        let files = detect_key_files(dir.path());
        assert!(files.contains(&"README.md".to_string()));
        assert!(files.contains(&"Makefile".to_string()));
    }

    #[test]
    fn key_files_github_workflows_dir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        let files = detect_key_files(dir.path());
        assert!(files.contains(&".github/workflows".to_string()));
    }

    #[test]
    fn key_files_all_candidates() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("README.md"), "").unwrap();
        fs::write(dir.path().join("CONTRIBUTING.md"), "").unwrap();
        fs::write(dir.path().join("Makefile"), "").unwrap();
        fs::write(dir.path().join("Dockerfile"), "").unwrap();
        fs::write(dir.path().join(".env.example"), "").unwrap();
        fs::write(dir.path().join("CHANGELOG.md"), "").unwrap();
        fs::create_dir_all(dir.path().join(".github/workflows")).unwrap();
        fs::create_dir_all(dir.path().join(".github")).unwrap();
        fs::write(dir.path().join(".github/CODEOWNERS"), "").unwrap();
        let files = detect_key_files(dir.path());
        assert_eq!(files.len(), 8);
    }
}
