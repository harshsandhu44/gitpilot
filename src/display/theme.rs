use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;

pub fn heading(s: &str) -> String {
    format!("{}", s.bold())
}

pub fn success(s: &str) -> String {
    format!("{}", s.green())
}

pub fn warning(s: &str) -> String {
    format!("{}", s.yellow())
}

pub fn error(s: &str) -> String {
    format!("{}", s.red())
}

pub fn dim(s: &str) -> String {
    format!("{}", s.dimmed())
}

pub fn info(s: &str) -> String {
    format!("{}", s.cyan())
}

pub fn relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let secs = (now - dt).num_seconds();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let m = secs / 60;
        format!("{} minute{} ago", m, if m == 1 { "" } else { "s" })
    } else if secs < 86400 {
        let h = secs / 3600;
        format!("{} hour{} ago", h, if h == 1 { "" } else { "s" })
    } else if secs < 7 * 86400 {
        let d = secs / 86400;
        format!("{} day{} ago", d, if d == 1 { "" } else { "s" })
    } else if secs < 30 * 86400 {
        let w = secs / (7 * 86400);
        format!("{} week{} ago", w, if w == 1 { "" } else { "s" })
    } else if secs < 365 * 86400 {
        let mo = secs / (30 * 86400);
        format!("{} month{} ago", mo, if mo == 1 { "" } else { "s" })
    } else {
        let y = secs / (365 * 86400);
        format!("{} year{} ago", y, if y == 1 { "" } else { "s" })
    }
}
