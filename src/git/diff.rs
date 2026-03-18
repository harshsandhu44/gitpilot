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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_helpers::{commit_file, make_repo, make_repo_context};

    #[test]
    fn staged_diff_empty_when_nothing_staged() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "hello", "initial");
        let ctx = make_repo_context(repo, dir.path());
        let diff = staged_diff(&ctx).unwrap();
        assert!(diff.files.is_empty());
        assert_eq!(diff.total_additions, 0);
        assert_eq!(diff.total_deletions, 0);
    }

    #[test]
    fn staged_diff_shows_added_lines() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "line1\n", "initial");
        // Stage a new file
        std::fs::write(dir.path().join("b.txt"), "alpha\nbeta\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("b.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let diff = staged_diff(&ctx).unwrap();
        assert!(diff.files.iter().any(|f| f.path == "b.txt"));
        assert!(diff.total_additions >= 2);
    }

    #[test]
    fn staged_diff_shows_modified_lines() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "original\n", "initial");
        std::fs::write(dir.path().join("a.txt"), "changed\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("a.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let diff = staged_diff(&ctx).unwrap();
        assert_eq!(diff.total_additions, 1);
        assert_eq!(diff.total_deletions, 1);
    }

    #[test]
    fn staged_diff_patch_contains_added_lines() {
        let (dir, repo) = make_repo();
        commit_file(&repo, dir.path(), "a.txt", "old\n", "initial");
        std::fs::write(dir.path().join("a.txt"), "new\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("a.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let diff = staged_diff(&ctx).unwrap();
        let file = diff.files.iter().find(|f| f.path == "a.txt").unwrap();
        assert!(file.patch.contains("+new"));
        assert!(file.patch.contains("-old"));
    }

    #[test]
    fn staged_diff_on_empty_repo() {
        let (dir, repo) = make_repo();
        // No commits yet — repo is empty
        std::fs::write(dir.path().join("first.txt"), "hello\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("first.txt")).unwrap();
        index.write().unwrap();
        let ctx = make_repo_context(repo, dir.path());
        let diff = staged_diff(&ctx).unwrap();
        assert!(diff.files.iter().any(|f| f.path == "first.txt"));
        assert!(diff.total_additions >= 1);
    }
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
