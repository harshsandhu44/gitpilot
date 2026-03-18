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

## Roadmap

Items are roughly ordered by priority. Each one moves the tool closer to a polished, production-grade CLI.

### Configuration file

Load settings from `~/.config/gitpilot/config.toml` and a per-repo `.gitpilot.toml`, falling back to hardcoded defaults. Lets users persist base branch, protected branches, stale threshold, and custom review patterns without passing flags every time.

### Shell completions

Generate and install completion scripts for bash, zsh, and fish via a `gitpilot completions <shell>` subcommand (clap's `generate` feature). Required for any CLI that people actually enjoy using.

### `--json` output flag

Add a global `--json` flag that emits structured JSON for every command. Enables scripting, CI pipelines, and editor integrations without screen-scraping.

### `switch` command

Fuzzy, interactive branch switcher. Lists local (and optionally remote) branches through dialoguer's fuzzy-select, then checks out the selection. Replaces the `git branch | fzf | xargs git checkout` muscle memory.

### `undo` command

Interactive "undo last N commits" with a choice of soft, mixed, or hard reset. Safer than remembering reset flags; shows the commits that will be affected before confirming.

### `stash` command

Interactive stash manager: list stashes with their messages and age, then pick one to apply, pop, or drop. Fills the gap left by the terse `git stash list` output.

### `sync` command

One-shot "get current branch up to date": fetch origin, rebase (or merge) from the upstream base branch, and report the result. Useful at the start of every work session.

### `init` command

Scaffold a `.gitpilot.toml` in the repo with commented-out defaults, and optionally install `gitpilot review` as a pre-commit hook. Reduces the setup friction for new users and new repos.

### `--dry-run` for destructive commands

Add a `--dry-run` flag to `cleanup` (and future destructive commands) that prints what would be deleted without doing it. Useful for CI checks and cautious users.

### CI-mode output

Auto-detect CI environments (`CI`, `GITHUB_ACTIONS`, `BUILDKITE`, etc.) and switch to plain, non-interactive, non-colored output with machine-friendly exit codes. `review` already exits `1` on findings; make the behavior consistent across all commands.

### `log` command

A compact, opinionated `git log` view: one-line commits with relative timestamps, author, and a branch/tag graph column. Filters by `--author`, `--since`, and `--grep` without remembering log format strings.

### `NO_COLOR` and `--no-color` support

Respect the `NO_COLOR` env var (per no-color.org) and a global `--no-color` flag. owo-colors supports this; it just needs to be wired to the env check on startup.

### Man page generation

Produce a `gitpilot.1` man page via `clap_mangen` and distribute it alongside the binary so users can run `man git-pilot`. Expected by package maintainers (Homebrew, AUR, etc.).

### Update checker

On startup (async, with a timeout), check the latest version on crates.io or GitHub releases and print a one-line notice if a newer version is available. Opt-out via config or `GITPILOT_NO_UPDATE_CHECK`.

## License

MIT
