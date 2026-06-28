# Performance on large repositories

`git-branch-status` can be slow in very large repositories. This document records
the investigation into where the time goes, what was tried, and the options for
improving it. It is meant as the basis for deciding how (and whether) to change
the implementation.

## Summary

- The branch **name** is cheap to compute (~6 ms). Effectively all of the time is
  spent in `branch_status()`.
- `branch_status()` is slow because libgit2 walks the **entire working tree**
  when diffing the index against the working directory, even with untracked
  files disabled.
- libgit2's status/diff cannot reach the speed of `git status -uno` (which only
  `lstat`s the paths recorded in the index), so option tweaks alone do not help.
- A meaningful speedup requires an architectural change, not a flag change.

## Methodology

- Tool: [hyperfine](https://github.com/sharkdp/hyperfine) (`--warmup 5 --shell=none`).
- Repository: `translated-content` (a clone of the MDN translated content), a
  realistic large repository.
  - Tracked files: **28,718**
  - Working tree: **clean** (worst case for status — every tracked file must be
    checked to prove nothing changed).
- Each candidate change was built into a separate release binary and benchmarked
  against the current release binary on the same repository.

The same investigation can be reproduced on a synthetic repository (e.g. a few
tens of thousands of files created and committed) when `translated-content` is
not available.

## Measurements

| Variant | Time | Notes |
| --- | --- | --- |
| Current (`statuses()`, untracked disabled) | **~2.0 s** | baseline |
| `StatusOptions::no_refresh(true)` | ~2.0 s | 1.01× — within noise |
| Diff API directly (conflicts via index, unstaged via index↔workdir, staged via HEAD↔index) | ~2.1 s | no improvement |
| Branch name only (status skipped) | **~6 ms** | isolates the cost to `branch_status()` |
| `git status -uno --porcelain` (git binary) | **~64 ms** | for reference |
| `git diff --quiet` (git binary) | **~61 ms** | for reference |
| `git status --porcelain` (git binary, with untracked) | ~1.66 s | for reference |

## Root cause

Two facts pin down the cause:

1. With untracked files **disabled**, the tool's system time (~1.4 s) matches
   `git status` **with** untracked files (~1.5 s), not `git status -uno`
   (~0.42 s of system time). In other words, libgit2 is still doing the full
   working-directory traversal — it `readdir`s every directory — and merely
   filters untracked entries out of the result.
2. Computing the status via the diff API directly is no faster, because libgit2's
   working-directory iterator does the same full traversal underneath.

By contrast, `git status -uno` / `git diff --quiet` only `lstat` the paths
already recorded in the index; they never enumerate directories. git also
parallelizes those `lstat`s (`core.preloadIndex`, on by default), while libgit2
runs single-threaded. Both factors compound:

- Avoiding the directory walk: ~5× (2.0 s → ~0.42 s single-threaded equivalent).
- Parallelizing the `lstat`s: another ~7× (~0.42 s → ~64 ms wall).

libgit2 exposes no "index-only" working-directory mode, so neither factor is
reachable through `StatusOptions`.

## What was ruled out

- **`no_refresh(true)`** — skips the soft index reload only; the working-tree
  traversal still dominates. No measurable change.
- **Diff API instead of `statuses()`** — same iterator, same cost. (It would also
  let conflicts be detected cheaply via `Index::has_conflicts()` and allow an
  early exit, but that only helps a *dirty* tree; the slow case is a *clean*
  tree, which must be fully scanned regardless.)
- **`exclude_submodules` / `include_ignored` / `include_unmodified`** — already
  set favorably in the current code.

## Options for a real speedup

### A. Shell out to `git` for the status

Use `git` (which is guaranteed to be present, since this tool runs as
`git branch-status`) to decide the status, e.g. `git diff --quiet` (unstaged),
`git diff --cached --quiet` (staged), plus a conflict check — or a single
`git status --porcelain=v2 -uno` parse. Keep libgit2 for the cheap branch name.

- Pros: ~30× faster on large repos (~2 s → ~60 ms); matches git's own behavior
  exactly.
- Cons: adds a process spawn (a few ms on small repos); depends on the `git`
  binary; departs from the current "pure libgit2" design.

### B. Reimplement the index-vs-worktree check on top of libgit2

Iterate the index entries and compare each against the file's `lstat` data using
the index's stat cache, replicating git's `-uno` fast path.

- Pros: fast without spawning a process.
- Cons: must reproduce git's racy-clean handling and content fallbacks
  correctly; significant, error-prone code for a prompt indicator.

### C. Make the status optional / bounded

Keep libgit2 but add an escape hatch: a `--no-status` mode (branch name only,
~6 ms) and/or automatically skip the status above a file-count threshold.

- Pros: low risk; small, self-contained change.
- Cons: does not actually make status fast — it trades the feature away in the
  cases where it is slow.

## Recommendation

Option **A** offers the largest, lowest-risk win and is consistent with the tool
being a git subcommand. Option **C** can complement any choice as a user-facing
escape hatch. Option **B** is fast but carries the most correctness risk.

## Upstream and prior art

This is a long-standing, known limitation in libgit2 rather than something
fixable through `StatusOptions` — and it is treated upstream as a performance
enhancement, not a correctness bug.

- [libgit2#4230 — "git_status_list is slower than `git status`"](https://github.com/libgit2/libgit2/issues/4230):
  open since 2017 and still open (last bumped in 2025). A maintainer confirmed it
  is something they care about, while framing it as a mix of "easy wins" and
  hard, structural work (e.g. threading). The thread pins down extra root causes
  beyond the working-tree walk noted above:
  - libgit2 status is single-threaded, whereas `git` parallelizes its `lstat`s.
  - For every file it searches for `.gitattributes` / `.gitignore` in each parent
    directory up to the repository root, with weak caching of negative lookups.
    Computing the OID for an entry pulls in the filter/attribute machinery
    (visible in the reported call stacks: `git_diff_index_to_workdir` →
    `maybe_modified` → `git_diff__oid_for_entry` → attribute lookups → `lstat`).
  - Even when only unstaged changes are wanted, working-tree files are still
    checked against ignore rules.
- Comments in that thread independently corroborate this investigation:
  - `git_diff_index_to_workdir` is as slow as `git_status_list`, while
    `git_diff_tree_to_index` is fast — matching the "diff API directly" result
    above.
  - A `git2-rs` user reported the same and resorted to shelling out to
    `git status`, citing a ~20× difference (in line with Option A here).
- Related upstream PRs (e.g. libgit2#5018 and follow-ups) have stalled in review,
  so an upstream fix should not be assumed to be coming soon.
- [gitstatusd](https://github.com/romkatv/gitstatus) is prior art for a dedicated
  fast implementation: a patched/forked libgit2 (skip parsing `.gitignore` on a
  clean tree, parallelize the index-to-workdir scan, disable index validation)
  reported as ~10× faster than `lg2 status` and ~2.5× faster than `git status`.
  It powers Powerlevel10k's prompt.
