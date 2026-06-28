# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/akiomik/nowhear/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/akiomik/nowhear/releases/tag/v0.1.0
