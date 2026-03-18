use anyhow::Result;
use regex::Regex;
use tabled::{Table, Tabled};
use crate::commands::CommandContext;
use crate::display::theme;
use crate::git::diff;

#[derive(Tabled)]
struct FindingRow {
    #[tabled(rename = "Severity")]
    severity: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "File")]
    file: String,
    #[tabled(rename = "Line")]
    line: String,
    #[tabled(rename = "Snippet")]
    snippet: String,
}

pub fn run(ctx: &CommandContext) -> Result<i32> {
    let diff_summary = diff::staged_diff(&ctx.repo)?;

    if diff_summary.files.is_empty() {
        println!("{}", theme::dim("No staged changes to review."));
        return Ok(0);
    }

    let total_lines: usize = diff_summary.files.iter().map(|f| f.additions + f.deletions).sum();
    let mut findings: Vec<FindingRow> = Vec::new();
    let mut has_error = false;

    // Large diff warning
    if total_lines > ctx.config.review_diff_threshold {
        findings.push(FindingRow {
            severity: theme::warning("warning"),
            category: "Large diff".to_string(),
            file: String::new(),
            line: String::new(),
            snippet: format!("{} lines changed", total_lines),
        });
    }

    let secret_patterns: Vec<Regex> = ctx
        .config
        .review_secrets_patterns
        .iter()
        .filter_map(|p| {
            Regex::new(p)
                .map_err(|e| eprintln!("{}", theme::warning(&format!("Invalid review pattern '{}': {}", p, e))))
                .ok()
        })
        .collect();

    let debug_patterns: Vec<(&str, Regex)> = vec![
        ("println!", Regex::new(r"println!\s*\(").unwrap()),
        ("dbg!", Regex::new(r"dbg!\s*\(").unwrap()),
        ("console.log", Regex::new(r"console\.log\s*\(").unwrap()),
        ("print!", Regex::new(r"\bprint!\s*\(").unwrap()),
        ("var_dump", Regex::new(r"\bvar_dump\s*\(").unwrap()),
    ];

    let todo_patterns: Vec<(&str, Regex)> = vec![
        ("TODO", Regex::new(r"\bTODO\b").unwrap()),
        ("FIXME", Regex::new(r"\bFIXME\b").unwrap()),
        ("HACK", Regex::new(r"\bHACK\b").unwrap()),
        ("XXX", Regex::new(r"\bXXX\b").unwrap()),
    ];

    for file in &diff_summary.files {
        for (line_num, line) in file.patch.lines().enumerate() {
            if !line.starts_with('+') {
                continue;
            }
            let content = &line[1..]; // strip leading '+'

            // Secrets
            for pat in &secret_patterns {
                if pat.is_match(content) {
                    has_error = true;
                    findings.push(FindingRow {
                        severity: theme::error("error"),
                        category: "Potential secret".to_string(),
                        file: file.path.clone(),
                        line: (line_num + 1).to_string(),
                        snippet: truncate(content.trim(), 60),
                    });
                }
            }

            // Debug artifacts
            for (label, pat) in &debug_patterns {
                if pat.is_match(content) {
                    findings.push(FindingRow {
                        severity: theme::warning("warning"),
                        category: format!("Debug: {}", label),
                        file: file.path.clone(),
                        line: (line_num + 1).to_string(),
                        snippet: truncate(content.trim(), 60),
                    });
                }
            }

            // TODO markers
            for (label, pat) in &todo_patterns {
                if pat.is_match(content) {
                    findings.push(FindingRow {
                        severity: theme::dim("info"),
                        category: format!("Marker: {}", label),
                        file: file.path.clone(),
                        line: (line_num + 1).to_string(),
                        snippet: truncate(content.trim(), 60),
                    });
                }
            }
        }
    }

    if findings.is_empty() {
        println!("{}", theme::success("No issues found in staged changes."));
        Ok(0)
    } else {
        println!(
            "{} {} finding{} in staged changes:\n",
            theme::heading("Review:"),
            findings.len(),
            if findings.len() == 1 { "" } else { "s" }
        );
        println!("{}", Table::new(findings));
        if has_error {
            Ok(1)
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn truncate_short_unchanged() {
        assert_eq!(truncate("short", 20), "short");
    }

    #[test]
    fn truncate_at_limit_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_over_limit_adds_ellipsis() {
        let s = "a".repeat(70);
        let result = truncate(&s, 60);
        assert!(result.ends_with('…'));
        assert!(result.chars().count() <= 60);
    }

    #[test]
    fn truncate_zero_returns_ellipsis() {
        assert_eq!(truncate("anything", 0), "…");
    }

    #[test]
    fn secret_pattern_aws_matches() {
        let pat = Regex::new(r"AWS_SECRET").unwrap();
        assert!(pat.is_match("AWS_SECRET_KEY=abc123"));
        assert!(!pat.is_match("aws_key=foo"));
    }

    #[test]
    fn secret_pattern_github_token_matches() {
        let pat = Regex::new(r"ghp_[A-Za-z0-9]+").unwrap();
        assert!(pat.is_match("token = ghp_abcABC123"));
        assert!(!pat.is_match("token = ghs_abc"));
    }

    #[test]
    fn secret_pattern_password_matches() {
        let pat = Regex::new(r"password\s*=").unwrap();
        assert!(pat.is_match("password = secret"));
        assert!(pat.is_match("password=secret"));
        assert!(!pat.is_match("user_password_hash"));
    }

    #[test]
    fn debug_pattern_println_matches() {
        let pat = Regex::new(r"println!\s*\(").unwrap();
        assert!(pat.is_match("println!(\"debug\")"));
        assert!(pat.is_match("println! (\"debug\")"));
        // eprintln! contains println! as a substring — the real command strips leading '+' from
        // diff lines and then scans the content, so this is expected to flag eprintln! too.
        assert!(pat.is_match("eprintln!(\"ok\")"));
        assert!(!pat.is_match("log::info!(\"ok\")"));
    }

    #[test]
    fn debug_pattern_console_log_matches() {
        let pat = Regex::new(r"console\.log\s*\(").unwrap();
        assert!(pat.is_match("console.log(\"debug\")"));
        assert!(!pat.is_match("console.warn(\"ok\")"));
    }

    #[test]
    fn todo_pattern_matches_word_boundary() {
        let pat = Regex::new(r"\bTODO\b").unwrap();
        assert!(pat.is_match("// TODO: fix this"));
        assert!(!pat.is_match("TODOLIST"));
    }

    #[test]
    fn fixme_pattern_matches() {
        let pat = Regex::new(r"\bFIXME\b").unwrap();
        assert!(pat.is_match("// FIXME: broken"));
    }

    #[test]
    fn hack_pattern_matches() {
        let pat = Regex::new(r"\bHACK\b").unwrap();
        assert!(pat.is_match("// HACK: workaround"));
        assert!(!pat.is_match("shackle"));
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let end = s
            .char_indices()
            .nth(max.saturating_sub(1))
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        format!("{}…", &s[..end])
    }
}
