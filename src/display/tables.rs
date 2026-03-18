use tabled::{Table, Tabled};
use crate::git::status::FileChange;
use crate::git::commits::CommitSummary;
use crate::git::branches::BranchInfo;

#[derive(Tabled)]
pub struct FileRow {
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "File")]
    pub path: String,
}

#[derive(Tabled)]
pub struct CommitRow {
    #[tabled(rename = "Hash")]
    pub hash: String,
    #[tabled(rename = "Author")]
    pub author: String,
    #[tabled(rename = "Date")]
    pub date: String,
    #[tabled(rename = "Message")]
    pub message: String,
}

#[derive(Tabled)]
pub struct BranchRow {
    #[tabled(rename = "Branch")]
    pub name: String,
    #[tabled(rename = "Status")]
    pub status: String,
    #[tabled(rename = "Age (days)")]
    pub age: String,
    #[tabled(rename = "Last Commit")]
    pub last_commit: String,
}

pub fn file_table(changes: &[FileChange]) -> String {
    let rows: Vec<FileRow> = changes
        .iter()
        .map(|c| FileRow {
            status: c.status.clone(),
            path: c.path.clone(),
        })
        .collect();
    Table::new(rows).to_string()
}

pub fn commit_table(commits: &[CommitSummary]) -> String {
    let rows: Vec<CommitRow> = commits
        .iter()
        .map(|c| CommitRow {
            hash: c.short_id.clone(),
            author: c.author.clone(),
            date: c.date.format("%Y-%m-%d").to_string(),
            message: truncate(&c.message, 60),
        })
        .collect();
    Table::new(rows).to_string()
}

pub fn branch_table<B: std::ops::Deref<Target = BranchInfo>>(branches: &[B]) -> String {
    let rows: Vec<BranchRow> = branches
        .iter()
        .map(|b| BranchRow {
            name: b.name.clone(),
            status: format!("{:?}", b.state),
            age: b.age_days.to_string(),
            last_commit: truncate(&b.last_commit_msg, 50),
        })
        .collect();
    Table::new(rows).to_string()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
