use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn relative_time_just_now() {
        let t = Utc::now() - Duration::seconds(30);
        assert_eq!(relative_time(t), "just now");
    }

    #[test]
    fn relative_time_one_minute() {
        let t = Utc::now() - Duration::seconds(90);
        assert_eq!(relative_time(t), "1 minute ago");
    }

    #[test]
    fn relative_time_plural_minutes() {
        let t = Utc::now() - Duration::minutes(5);
        assert_eq!(relative_time(t), "5 minutes ago");
    }

    #[test]
    fn relative_time_one_hour() {
        let t = Utc::now() - Duration::minutes(65);
        assert_eq!(relative_time(t), "1 hour ago");
    }

    #[test]
    fn relative_time_plural_hours() {
        let t = Utc::now() - Duration::hours(3);
        assert_eq!(relative_time(t), "3 hours ago");
    }

    #[test]
    fn relative_time_one_day() {
        let t = Utc::now() - Duration::hours(25);
        assert_eq!(relative_time(t), "1 day ago");
    }

    #[test]
    fn relative_time_plural_days() {
        let t = Utc::now() - Duration::days(3);
        assert_eq!(relative_time(t), "3 days ago");
    }

    #[test]
    fn relative_time_one_week() {
        let t = Utc::now() - Duration::days(10);
        assert_eq!(relative_time(t), "1 week ago");
    }

    #[test]
    fn relative_time_plural_weeks() {
        let t = Utc::now() - Duration::days(21);
        assert_eq!(relative_time(t), "3 weeks ago");
    }

    #[test]
    fn relative_time_months() {
        let t = Utc::now() - Duration::days(60);
        assert_eq!(relative_time(t), "2 months ago");
    }

    #[test]
    fn relative_time_one_year() {
        let t = Utc::now() - Duration::days(400);
        assert_eq!(relative_time(t), "1 year ago");
    }

    #[test]
    fn relative_time_plural_years() {
        let t = Utc::now() - Duration::days(800);
        assert_eq!(relative_time(t), "2 years ago");
    }

    #[test]
    fn color_fns_contain_input_text() {
        owo_colors::set_override(false);
        assert!(success("ok").contains("ok"));
        assert!(error("bad").contains("bad"));
        assert!(warning("warn").contains("warn"));
        assert!(heading("head").contains("head"));
        assert!(dim("quiet").contains("quiet"));
        assert!(info("note").contains("note"));
    }
}

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
