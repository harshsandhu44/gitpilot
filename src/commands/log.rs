use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use std::collections::HashMap;
use crate::commands::CommandContext;
use crate::display::theme;
use crate::git::commits::{filtered, LogFilter};

#[derive(Serialize)]
struct CommitJson {
    hash: String,
    author: String,
    date: String,
    message: String,
    refs: Vec<String>,
}

pub fn run(
    ctx: &CommandContext,
    author: Option<&str>,
    since: Option<&str>,
    grep: Option<&str>,
    count: usize,
) -> Result<()> {
    let since_dt = since.map(parse_since).transpose()?;

    let filter = LogFilter {
        author,
        since: since_dt,
        grep,
    };

    let commits = filtered(&ctx.repo, count, &filter)?;

    if commits.is_empty() {
        if !ctx.json {
            println!("{}", theme::dim("No commits found."));
        } else {
            println!("[]");
        }
        return Ok(());
    }

    // Build ref decoration map
    let mut ref_map: HashMap<git2::Oid, Vec<String>> = HashMap::new();
    if let Ok(refs) = ctx.repo.repo.references() {
        for r in refs.flatten() {
            if let (Some(name), Some(oid)) = (r.shorthand(), r.target()) {
                ref_map.entry(oid).or_default().push(name.to_string());
            }
        }
    }

    if ctx.json {
        let items: Vec<CommitJson> = commits
            .iter()
            .map(|c| CommitJson {
                hash: c.short_id.clone(),
                author: c.author.clone(),
                date: c.date.to_rfc3339(),
                message: c.message.clone(),
                refs: ref_map.get(&c.full_id).cloned().unwrap_or_default(),
            })
            .collect();
        println!("{}", serde_json::to_string(&items)?);
    } else {
        for c in &commits {
            let refs = ref_map.get(&c.full_id).cloned().unwrap_or_default();
            let ref_label = if refs.is_empty() {
                String::new()
            } else {
                format!("  {}", theme::info(&format!("[{}]", refs.join(", "))))
            };
            println!(
                "{}  {}  {}  {}{}",
                theme::info(&c.short_id),
                theme::dim(&theme::relative_time(c.date)),
                c.author,
                c.message,
                ref_label,
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_days() {
        let before = Utc::now();
        let result = parse_since("7d").unwrap();
        let expected = before - Duration::days(7);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_one_day() {
        let before = Utc::now();
        let result = parse_since("1d").unwrap();
        let expected = before - Duration::days(1);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_weeks() {
        let before = Utc::now();
        let result = parse_since("2w").unwrap();
        let expected = before - Duration::weeks(2);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_months() {
        let before = Utc::now();
        let result = parse_since("3m").unwrap();
        let expected = before - Duration::days(90);
        assert!((result - expected).num_seconds().abs() < 2);
    }

    #[test]
    fn parse_since_date() {
        let result = parse_since("2026-01-15").unwrap();
        assert_eq!(result.format("%Y-%m-%d").to_string(), "2026-01-15");
    }

    #[test]
    fn parse_since_date_at_midnight() {
        let result = parse_since("2025-06-01").unwrap();
        assert_eq!(result.format("%H:%M:%S").to_string(), "00:00:00");
    }

    #[test]
    fn parse_since_invalid_string() {
        assert!(parse_since("abc").is_err());
    }

    #[test]
    fn parse_since_unknown_unit() {
        assert!(parse_since("7x").is_err());
    }

    #[test]
    fn parse_since_empty_string() {
        assert!(parse_since("").is_err());
    }

    #[test]
    fn parse_since_bad_date_format() {
        assert!(parse_since("15-01-2026").is_err());
    }
}

fn parse_since(s: &str) -> Result<DateTime<Utc>> {
    // Shorthand: Nd (days), Nw (weeks), Nm (months)
    if let Some(rest) = s.strip_suffix('d') {
        if let Ok(n) = rest.parse::<i64>() {
            return Ok(Utc::now() - Duration::days(n));
        }
    }
    if let Some(rest) = s.strip_suffix('w') {
        if let Ok(n) = rest.parse::<i64>() {
            return Ok(Utc::now() - Duration::weeks(n));
        }
    }
    if let Some(rest) = s.strip_suffix('m') {
        if let Ok(n) = rest.parse::<i64>() {
            return Ok(Utc::now() - Duration::days(n * 30));
        }
    }
    // YYYY-MM-DD
    let naive = NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date format: '{}'. Use YYYY-MM-DD, Nd, Nw, or Nm", s))?;
    Ok(naive.and_hms_opt(0, 0, 0).unwrap().and_utc())
}
