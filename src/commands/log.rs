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
