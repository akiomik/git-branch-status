# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Speed up status detection by short-circuiting the worktree scan. Conflicts are
  now detected from the index up front, and the scan stops at the first unstaged
  change instead of always walking the entire worktree.

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

[unreleased]: https://github.com/akiomik/git-branch-status/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/akiomik/git-branch-status/releases/tag/v0.2.0
[0.1.1]: https://github.com/akiomik/git-branch-status/releases/tag/v0.1.1
[0.1.0]: https://github.com/akiomik/git-branch-status/releases/tag/v0.1.0
