# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Escape `%` characters in branch names when formatting for zsh (`--mode zsh`).
  Without this, a branch named e.g. `feature/%n` would cause zsh to expand `%n`
  to the login name inside `$PROMPT`, producing incorrect output.
- Ignore an empty or whitespace-only `rebase-merge/head-name` or
  `rebase-apply/head-name` file instead of returning an empty branch name.
  A partially-written or corrupted file now falls back to the real HEAD.
- Only read the `head-name` file to recover the original branch name when gix
  confirms a rebase is in progress (`Rebase`, `RebaseInteractive`, or
  `ApplyMailboxRebase`). Reading it unconditionally could silently suppress the
  action label or show a stale name from a previous rebase. When a rebase is
  active but the file is absent, the display now falls back to HEAD gracefully.
- When a ref recorded in `head-name` cannot be found in the repository (e.g.
  the branch was deleted mid-rebase), the shorthand fallback now strips
  `refs/remotes/` and `refs/tags/` in addition to `refs/heads/`, instead of
  returning the full ref name verbatim.

## [0.2.1] - 2026-06-30

### Changed

- Speed up status detection by short-circuiting the worktree scan. Conflicts are
  now detected from the index up front, and the scan stops at the first unstaged
  change instead of always walking the entire worktree.
- Disable rename detection during status detection. It read blob contents to
  compute similarity without affecting the reported status, so turning it off
  removes that overhead.

## [0.2.0] - 2026-06-30

### Added

- Accept an optional `[DIR]` positional argument to specify the git repository
  path (e.g. `git-branch-status --mode zsh /path/to/repo`). Defaults to `.`
  when omitted, preserving the previous behavior.

### Changed

- Replace the libgit2 (`git2`) backend with the pure-Rust `gix` (gitoxide). This
  makes `git-branch-status` about 22x faster in large repositories (e.g. roughly
  2.0 s to 92 ms on a 28k-file repository) and drops the libgit2/OpenSSL C
  dependency, so the binary is now pure Rust

## [0.1.1] - 2026-06-29

### Fixed

- Exit quietly instead of panicking when the branch name or status cannot be
  retrieved, so the prompt is not polluted with a panic message
- Show the branch name instead of `HEAD (no branch)` on an unborn branch (a
  repository without any commits yet)
- Show the original branch name during an apply-backend rebase, instead of the
  detached commit hash
- Show the tag name when HEAD is detached at a tag, instead of the commit hash

## [0.1.0] - 2026-06-29

### Changed

- Initial release

[unreleased]: https://github.com/akiomik/git-branch-status/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/akiomik/git-branch-status/releases/tag/v0.2.1
[0.2.0]: https://github.com/akiomik/git-branch-status/releases/tag/v0.2.0
[0.1.1]: https://github.com/akiomik/git-branch-status/releases/tag/v0.1.1
[0.1.0]: https://github.com/akiomik/git-branch-status/releases/tag/v0.1.0
