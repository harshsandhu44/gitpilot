# gitpilot

A Rust CLI that handles the tedious parts of daily Git workflow: quick repo inspection, PR summaries, pre-commit risk detection, and safe branch cleanup.

## Install

```bash
cargo install gitpilot
```

Or build from source:

```bash
cargo install --path .
```

Once installed, you can also invoke gitpilot as a Git subcommand:

```bash
git pilot status
git pilot summary
```

## Commands

### `status`

Shows the full state of your working tree at a glance.

```
gitpilot status
```

- Current branch and upstream ahead/behind count
- Staged, unstaged, and untracked files
- Stash count
- Last 5 commits

### `summary`

Summarizes what changed on the current branch relative to a base branch — useful when writing PR descriptions.

```
gitpilot summary
gitpilot summary --base develop
```

- Total additions and deletions per file
- List of commits on the branch

### `review`

Scans staged changes for common issues before you commit. Exits with code `1` if any errors are found, making it suitable as a pre-commit hook.

```
gitpilot review
```

Detects:

| Category | Examples |
|----------|----------|
| Potential secrets | `AWS_SECRET`, `ghp_…`, `password =`, `-----BEGIN` |
| Debug artifacts | `println!`, `dbg!`, `console.log` |
| Markers | `TODO`, `FIXME`, `HACK`, `XXX` |

**Use as a pre-commit hook:**

```bash
echo 'gitpilot review' >> .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### `cleanup`

Lists branches that are merged, gone (remote deleted), or stale (no commits in 30+ days), then lets you interactively pick which ones to delete.

```
gitpilot cleanup
gitpilot cleanup --base develop
```

Protected branches (`main`, `master`, `develop`) and the current branch are always skipped.

## Default config

| Setting | Default |
|---------|---------|
| Base branch | `main` |
| Protected branches | `main`, `master`, `develop` |
| Stale threshold | 30 days |

## License

MIT
