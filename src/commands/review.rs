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
    if total_lines > 500 {
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
        .filter_map(|p| Regex::new(p).ok())
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
