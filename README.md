# git-pilot

A Rust CLI that handles the tedious parts of daily Git workflow: quick repo inspection, PR summaries, pre-commit risk detection, branch cleanup, and more.

## Install

```bash
cargo install git-pilot
```

Or build from source:

```bash
cargo install --path .
```

Once installed, use it as a Git subcommand:

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
git pilot status
```

- Current branch and upstream ahead/behind count
- Staged, unstaged, and untracked files
- Stash count
- Last 5 commits

### `summary`

Summarizes what changed on the current branch relative to a base branch — useful when writing PR descriptions.

```
git pilot summary
git pilot summary --base develop
```

- Total additions and deletions per file
- List of commits on the branch

### `review`

Scans staged changes for common issues before you commit. Exits with code `1` if any errors are found, making it suitable as a pre-commit hook.

```
git pilot review
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
git pilot cleanup
git pilot cleanup --base develop
git pilot cleanup --dry-run
```

- `--dry-run` — preview what would be deleted without deleting anything
- Protected branches (`main`, `master`, `develop`) and the current branch are always skipped

### `switch`

Fuzzy, interactive branch switcher. Checks out the selected branch; creates a local tracking branch if `--remote` is used and the branch only exists on origin.

```
git pilot switch
git pilot switch --remote
```

### `sync`

Fetches origin and rebases (or merges) the current branch onto the base branch.

```
git pilot sync
git pilot sync --base develop
```

The strategy is controlled by `sync_strategy` in your config (default: `rebase`).

### `log`

Compact commit history with relative timestamps, author, and ref decorations.

```
git pilot log
git pilot log --count 50
git pilot log --author alice
git pilot log --since 7d
git pilot log --grep feat
```

`--since` accepts `YYYY-MM-DD`, `Nd` (days), `Nw` (weeks), or `Nm` (months).

### `undo`

Interactively undo the last N commits with a choice of soft, mixed, or hard reset. Shows the affected commits before confirming a hard reset.

```
git pilot undo
git pilot undo --count 10
```

### `stash`

Interactive stash manager: lists stashes, then lets you apply, pop, or drop the selected entry.

```
git pilot stash
```

### `init`

Scaffolds a `.gitpilot.toml` in the current repo with commented-out defaults. Pass `--hook` to also install `git pilot review` as a pre-commit hook.

```
git pilot init
git pilot init --hook
```

### `generate`

Generate shell completion scripts or a man page.

```
git pilot generate completions bash
git pilot generate completions zsh
git pilot generate completions fish
git pilot generate man
git pilot generate man --output git-pilot.1
```

Shorthand for completions:

```
git pilot completions zsh
```

**Use as a pre-commit hook (manual):**

```bash
echo 'git pilot review' >> .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Or use `git pilot init --hook` to do this automatically.

## Configuration

Config is loaded from `~/.config/gitpilot/config.toml` (global) and `.gitpilot.toml` in the repo root (local), with local values taking precedence. Run `git pilot init` to scaffold a local config.

| Setting | Default | Description |
|---------|---------|-------------|
| `base_branch` | `"main"` | Branch used for comparisons in `summary`, `cleanup`, `sync` |
| `protected_branches` | `["main", "master", "develop"]` | Never deleted by `cleanup` |
| `stale_days` | `30` | Branches with no commits newer than this are flagged as stale |
| `review_secrets_patterns` | See defaults | Regex patterns checked by `review` |
| `sync_strategy` | `"rebase"` | `"rebase"` or `"merge"` |

## Update checker

On startup, git-pilot checks crates.io for a newer version and prints a notice if one is available. Set `GITPILOT_NO_UPDATE_CHECK=1` to disable.

## License

MIT
