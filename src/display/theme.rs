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
