use std::cell::RefCell;
use std::collections::HashMap;
use crate::error::GitPilotError;
use super::RepoContext;

#[derive(Debug)]
pub struct FileDiff {
    pub path: String,
    pub additions: usize,
    pub deletions: usize,
    pub patch: String,
}

#[derive(Debug)]
pub struct DiffSummary {
    pub files: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
}

struct FileAccum {
    order: usize,
    additions: usize,
    deletions: usize,
    patch: String,
}

fn process_diff(diff: git2::Diff) -> Result<DiffSummary, GitPilotError> {
    let counter = RefCell::new(0usize);
    let map: RefCell<HashMap<String, FileAccum>> = RefCell::new(HashMap::new());

    diff.foreach(
        &mut |delta, _| {
            let path = delta
                .new_file()
                .path()
                .and_then(|p| p.to_str())
                .unwrap_or("?")
                .to_string();
            let mut m = map.borrow_mut();
            if !m.contains_key(&path) {
                let idx = {
                    let mut c = counter.borrow_mut();
                    let v = *c;
                    *c += 1;
                    v
                };
                m.insert(path, FileAccum {
                    order: idx,
                    additions: 0,
                    deletions: 0,
                    patch: String::new(),
                });
            }
            true
        },
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            let path = delta
                .new_file()
                .path()
                .and_then(|p| p.to_str())
                .unwrap_or("?")
                .to_string();
            let mut m = map.borrow_mut();
            if let Some(f) = m.get_mut(&path) {
                match line.origin() {
                    '+' => {
                        f.additions += 1;
                        let content = std::str::from_utf8(line.content()).unwrap_or("").to_string();
                        f.patch.push('+');
                        f.patch.push_str(&content);
                    }
                    '-' => {
                        f.deletions += 1;
                        let content = std::str::from_utf8(line.content()).unwrap_or("").to_string();
                        f.patch.push('-');
                        f.patch.push_str(&content);
                    }
                    _ => {}
                }
            }
            true
        }),
    )?;

    let mut total_additions = 0usize;
    let mut total_deletions = 0usize;

    let mut entries: Vec<(String, FileAccum)> = map.into_inner().into_iter().collect();
    entries.sort_by_key(|(_, acc)| acc.order);

    let files: Vec<FileDiff> = entries
        .into_iter()
        .map(|(path, acc)| {
            total_additions += acc.additions;
            total_deletions += acc.deletions;
            FileDiff {
                path,
                additions: acc.additions,
                deletions: acc.deletions,
                patch: acc.patch,
            }
        })
        .collect();

    Ok(DiffSummary {
        files,
        total_additions,
        total_deletions,
    })
}

pub fn diff_vs_base(ctx: &RepoContext, base_branch: &str) -> Result<DiffSummary, GitPilotError> {
    let repo = &ctx.repo;

    let base_ref = format!("refs/remotes/origin/{}", base_branch);
    let base_obj = repo
        .revparse_single(&base_ref)
        .or_else(|_| repo.revparse_single(base_branch))?;

    let head = repo.head()?.peel_to_commit()?;
    let base_commit = base_obj.peel_to_commit()?;

    let merge_base_oid = repo.merge_base(head.id(), base_commit.id())?;
    let merge_base = repo.find_commit(merge_base_oid)?;

    let base_tree = merge_base.tree()?;
    let head_tree = head.tree()?;

    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;
    process_diff(diff)
}

pub fn staged_diff(ctx: &RepoContext) -> Result<DiffSummary, GitPilotError> {
    let repo = &ctx.repo;

    let head_tree = if repo.is_empty()? {
        None
    } else {
        Some(repo.head()?.peel_to_tree()?)
    };

    let diff = repo.diff_tree_to_index(head_tree.as_ref(), None, None)?;
    process_diff(diff)
}
