# gitpilot

A Rust CLI that handles the tedious parts of daily Git workflow: quick repo inspection, PR summaries, pre-commit risk detection, branch cleanup, and more.

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

## Global flags

| Flag | Env var | Description |
|------|---------|-------------|
| `--json` | — | Emit structured JSON output |
| `--no-color` | `NO_COLOR` | Disable color output |

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

### `cleanup`

Lists branches that are merged, gone (remote deleted), or stale (no commits in 30+ days), then lets you interactively pick which ones to delete.

```
gitpilot cleanup
gitpilot cleanup --base develop
gitpilot cleanup --dry-run
```

- `--dry-run` — preview what would be deleted without deleting anything
- Protected branches (`main`, `master`, `develop`) and the current branch are always skipped

### `switch`

Fuzzy, interactive branch switcher. Checks out the selected branch; creates a local tracking branch if `--remote` is used and the branch only exists on origin.

```
gitpilot switch
gitpilot switch --remote
```

### `sync`

Fetches origin and rebases (or merges) the current branch onto the base branch.

```
gitpilot sync
gitpilot sync --base develop
```

The strategy is controlled by `sync_strategy` in your config (default: `rebase`).

### `log`

Compact commit history with relative timestamps, author, and ref decorations.

```
gitpilot log
gitpilot log --count 50
gitpilot log --author alice
gitpilot log --since 7d
gitpilot log --grep feat
```

`--since` accepts `YYYY-MM-DD`, `Nd` (days), `Nw` (weeks), or `Nm` (months).

### `undo`

Interactively undo the last N commits with a choice of soft, mixed, or hard reset. Shows the affected commits before confirming a hard reset.

```
gitpilot undo
gitpilot undo --count 10
```

### `stash`

Interactive stash manager: lists stashes, then lets you apply, pop, or drop the selected entry.

```
gitpilot stash
```

### `init`

Scaffolds a `.gitpilot.toml` in the current repo with commented-out defaults. Pass `--hook` to also install `gitpilot review` as a pre-commit hook.

```
gitpilot init
gitpilot init --hook
```

### `generate`

Generate shell completion scripts or a man page.

```
gitpilot generate completions bash
gitpilot generate completions zsh
gitpilot generate completions fish
gitpilot generate man
gitpilot generate man --output gitpilot.1
```

Shorthand for completions:

```
gitpilot completions zsh
```

**Use as a pre-commit hook (manual):**

```bash
echo 'gitpilot review' >> .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or use `gitpilot init --hook` to do this automatically.

## Configuration

Config is loaded from `~/.config/gitpilot/config.toml` (global) and `.gitpilot.toml` in the repo root (local), with local values taking precedence. Run `gitpilot init` to scaffold a local config.

| Setting | Default | Description |
|---------|---------|-------------|
| `base_branch` | `"main"` | Branch used for comparisons in `summary`, `cleanup`, `sync` |
| `protected_branches` | `["main", "master", "develop"]` | Never deleted by `cleanup` |
| `stale_days` | `30` | Branches with no commits newer than this are flagged as stale |
| `review_secrets_patterns` | See defaults | Regex patterns checked by `review` |
| `sync_strategy` | `"rebase"` | `"rebase"` or `"merge"` |

## Update checker

On startup, gitpilot checks crates.io for a newer version and prints a notice if one is available. Set `GITPILOT_NO_UPDATE_CHECK=1` to disable.

## License

MIT
