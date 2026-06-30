// Copyright 2021 Akiomi Kamakura
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs;
use std::path::Path;

use gix::bstr::BString;
use gix::commit::describe::SelectRef;
use gix::head::Kind::{Detached, Symbolic, Unborn};
use gix::progress::Discard;
use gix::refs::FullNameRef;
use gix::state::InProgress;
use gix::status::index_worktree::Item as IndexWorktreeItem;
use gix::status::plumbing::index_as_worktree::EntryStatus;
use gix::status::tree_index::TrackRenames;
use gix::status::{Item as StatusItem, UntrackedFiles};

use crate::branch::Status;
use crate::error::Error;

/// A thin wrapper over [`gix::Repository`] exposing only the operations this tool
/// needs, keeping all `gix` types contained to this module.
pub struct Repository(gix::Repository);

impl Repository {
    /// Discover a repository starting from `path` and walking up to the root.
    ///
    /// # Errors
    ///
    /// Returns an error if no git repository is found at or above `path`.
    pub fn discover(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self(gix::discover(path)?))
    }

    /// The branch name to display, optionally suffixed with the in-progress
    /// action (e.g. `main:rebase-i`).
    ///
    /// # Errors
    ///
    /// Returns an error if the HEAD reference cannot be resolved.
    pub fn branch_name(&self) -> Result<String, Error> {
        let head = self.0.head()?;
        let state = self.0.state();

        let branch = if let Some(name) = self.rebase_head_name() {
            // A rebase records the original branch name on disk; prefer it over
            // the detached HEAD that a rebase leaves behind.
            name
        } else {
            match &head.kind {
                Symbolic(reference) => reference.name.shorten().to_string(),
                Unborn(name) => Self::shorthand(name.as_ref()),
                Detached { target, .. } => {
                    // Prefer a tag pointing at HEAD, falling back to the short hash.
                    self.tag_name()
                        .or_else(|| self.short_id(*target))
                        .unwrap_or_else(|| "HEAD (detached)".to_string())
                }
            }
        };

        match state {
            Some(state) => Ok(branch + ":" + state.label()),
            None => Ok(branch),
        }
    }

    /// The worst status across the working tree, ignoring untracked files.
    ///
    /// # Errors
    ///
    /// Returns an error if the status iterator cannot be created or yields an
    /// error while iterating.
    pub fn branch_status(&self) -> Result<Status, Error> {
        // A conflict is the worst status, and it is recorded in the index as
        // unmerged entries (stage != 0). Detecting it from the in-memory index
        // avoids the expensive worktree scan entirely when one exists.
        if self.has_conflicts()? {
            return Ok(Status::Conflicted);
        }

        let iter = self
            .0
            .status(Discard)?
            .untracked_files(UntrackedFiles::None)
            // Rename detection (on by default) reads blob contents to compute
            // similarity, which is pure overhead here: a rename maps to the same
            // staged/unstaged status as a separate delete and add would.
            .index_worktree_rewrites(None)
            .tree_index_track_renames(TrackRenames::Disabled)
            .into_iter(Vec::<BString>::new())?;

        // With conflicts ruled out, an unstaged change is the worst remaining
        // status, so the worktree scan can stop at the first one it finds
        // instead of walking the whole tree.
        let mut status = Status::NotChanged;
        for item in iter {
            match item? {
                // HEAD <-> index: a staged change. Unstaged changes short-circuit
                // above, so staged is the highest status this loop can settle on.
                StatusItem::TreeIndex(_) => status = Status::Staged,
                // index <-> worktree: an unstaged change.
                StatusItem::IndexWorktree(IndexWorktreeItem::Modification {
                    status: entry,
                    ..
                }) => match entry {
                    EntryStatus::Change(_) => return Ok(Status::Unstaged),
                    // A conflict would have been caught by `has_conflicts`, but
                    // handle it defensively rather than misreporting it.
                    EntryStatus::Conflict { .. } => return Ok(Status::Conflicted),
                    // Stat-only refresh or `--intent-to-add`: nothing changed.
                    EntryStatus::NeedsUpdate(_) | EntryStatus::IntentToAdd => {}
                },
                // Rewrites are disabled above, but were one to slip through it
                // would still be an unstaged worktree change.
                StatusItem::IndexWorktree(IndexWorktreeItem::Rewrite { .. }) => {
                    return Ok(Status::Unstaged);
                }
                // Untracked entries are excluded by `UntrackedFiles::None`.
                StatusItem::IndexWorktree(IndexWorktreeItem::DirectoryContents { .. }) => {}
            }
        }

        Ok(status)
    }

    /// Whether the index has any unmerged entries, i.e. a conflict is in
    /// progress. Unmerged entries carry a non-zero stage (base/ours/theirs).
    fn has_conflicts(&self) -> Result<bool, Error> {
        let index = self.0.index_or_empty()?;
        Ok(index.entries().iter().any(|entry| entry.stage_raw() != 0))
    }

    /// The original branch name recorded by an in-progress rebase, if any.
    ///
    /// The merge backend (and interactive rebases) use `rebase-merge/`, while the
    /// apply backend uses `rebase-apply/`. Both record the original ref in a
    /// `head-name` file, so we read whichever is present instead of inferring the
    /// directory from the repository state.
    fn rebase_head_name(&self) -> Option<String> {
        let git_dir = self.0.path();
        for dir in ["rebase-merge", "rebase-apply"] {
            let path = git_dir.join(dir).join("head-name");
            if let Ok(content) = fs::read_to_string(&path) {
                let refname = content.trim();
                return Some(self.shorthand_of_ref(refname));
            }
        }
        None
    }

    /// A tag pointing exactly at HEAD, behaving like `git describe --exact-match`.
    fn tag_name(&self) -> Option<String> {
        let commit = self.0.head_commit().ok()?;
        let format = commit
            .describe()
            .names(SelectRef::AllTags)
            .max_candidates(0)
            .try_format()
            .ok()??;
        Some(format.to_string())
    }

    /// The abbreviated hex of an object id, or `None` if it cannot be resolved.
    fn short_id(&self, id: gix::ObjectId) -> Option<String> {
        let object = self.0.find_object(id).ok()?;
        let short = object.id().shorten().ok()?;
        Some(short.to_string())
    }

    /// Resolve a full ref name to its shorthand, prettifying via the ref store
    /// when possible and otherwise stripping the `refs/heads/` prefix.
    fn shorthand_of_ref(&self, refname: &str) -> String {
        if let Ok(reference) = self.0.find_reference(refname) {
            return reference.name().shorten().to_string();
        }
        refname
            .strip_prefix("refs/heads/")
            .unwrap_or(refname)
            .to_string()
    }

    fn shorthand(name: &FullNameRef) -> String {
        name.shorten().to_string()
    }
}

/// Extension methods on [`gix::state::InProgress`] for display purposes.
trait InProgressExt {
    /// A short, human-readable label for the in-progress action (e.g. `"rebase-i"`).
    fn label(&self) -> &'static str;
}

impl InProgressExt for InProgress {
    fn label(&self) -> &'static str {
        match self {
            Self::ApplyMailbox => "am",
            Self::ApplyMailboxRebase => "am/rebase",
            Self::Bisect => "bisect",
            Self::CherryPick => "cherry",
            Self::CherryPickSequence => "cherry-seq",
            Self::Merge => "merge",
            Self::Rebase => "rebase",
            Self::RebaseInteractive => "rebase-i",
            Self::Revert => "revert",
            Self::RevertSequence => "revert-seq",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use assert_cmd::Command;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    /// Run `git` in `dir`, neutralizing the user's global signing config and
    /// pinning identity so fixtures are hermetic.
    fn git(dir: &Path, args: &[&str]) {
        let mut full = vec!["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"];
        full.extend_from_slice(args);
        Command::new("git")
            .current_dir(dir)
            .args(&full)
            .env("GIT_AUTHOR_NAME", "tester")
            .env("GIT_AUTHOR_EMAIL", "tester@example.com")
            .env("GIT_COMMITTER_NAME", "tester")
            .env("GIT_COMMITTER_EMAIL", "tester@example.com")
            .assert()
            .success();
    }

    fn git_stdout(dir: &Path, args: &[&str]) -> Result<String> {
        let cmd = Command::new("git")
            .current_dir(dir)
            .args(args)
            .assert()
            .success();
        let out = cmd.get_output().to_owned();
        Ok(String::from_utf8(out.stdout)?.trim().to_string())
    }

    /// A repository with a single committed file on `main`.
    fn init_repo() -> Result<TempDir> {
        let dir = TempDir::new()?;
        dir.child("f").write_str("a\n")?;
        git(dir.path(), &["init", "-q", "-b", "main"]);
        git(dir.path(), &["add", "f"]);
        git(dir.path(), &["commit", "-qm", "init"]);
        Ok(dir)
    }

    fn open(dir: &TempDir) -> Result<Repository> {
        Repository::discover(dir.path()).map_err(Into::into)
    }

    #[test]
    fn branch_name_returns_branch_on_unborn_branch() -> Result<()> {
        let dir = TempDir::new()?;
        git(dir.path(), &["init", "-q", "-b", "main"]);
        assert_eq!(open(&dir)?.branch_name()?, "main");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_returns_branch_on_normal_branch() -> Result<()> {
        let dir = init_repo()?;
        assert_eq!(open(&dir)?.branch_name()?, "main");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_returns_short_hash_on_detached_head() -> Result<()> {
        let dir = init_repo()?;
        git(dir.path(), &["checkout", "-q", "--detach", "HEAD"]);
        let short = git_stdout(dir.path(), &["rev-parse", "--short", "HEAD"])?;
        assert_eq!(open(&dir)?.branch_name()?, short);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_returns_lightweight_tag_on_detached_head_at_tag() -> Result<()> {
        let dir = init_repo()?;
        git(dir.path(), &["tag", "v1.0.0"]);
        git(dir.path(), &["checkout", "-q", "--detach", "v1.0.0"]);
        assert_eq!(open(&dir)?.branch_name()?, "v1.0.0");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_returns_annotated_tag_on_detached_head_at_tag() -> Result<()> {
        let dir = init_repo()?;
        git(dir.path(), &["tag", "-a", "v2.0.0", "-m", "rel"]);
        git(dir.path(), &["checkout", "-q", "--detach", "v2.0.0"]);
        assert_eq!(open(&dir)?.branch_name()?, "v2.0.0");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_appends_merge_action_during_merge() -> Result<()> {
        let dir = init_repo()?;
        // gix reports the `Merge` state when MERGE_HEAD exists. Fabricate it
        // directly rather than via `git merge`, whose conflict behavior is
        // environment-sensitive (e.g. it can no-op on some CI runners).
        let head = git_stdout(dir.path(), &["rev-parse", "HEAD"])?;
        dir.child(".git/MERGE_HEAD")
            .write_str(&format!("{head}\n"))?;
        assert_eq!(open(&dir)?.branch_name()?, "main:merge");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_uses_rebase_merge_head_name_during_interactive_rebase() -> Result<()> {
        let dir = init_repo()?;
        dir.child(".git/rebase-merge/head-name")
            .write_str("refs/heads/feature\n")?;
        dir.child(".git/rebase-merge/interactive").touch()?;
        assert_eq!(open(&dir)?.branch_name()?, "feature:rebase-i");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_name_uses_rebase_apply_head_name_during_apply_rebase() -> Result<()> {
        let dir = init_repo()?;
        dir.child(".git/rebase-apply/head-name")
            .write_str("refs/heads/feature\n")?;
        dir.child(".git/rebase-apply/rebasing").touch()?;
        assert_eq!(open(&dir)?.branch_name()?, "feature:rebase");
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_is_not_changed_on_clean_tree() -> Result<()> {
        let dir = init_repo()?;
        assert_eq!(open(&dir)?.branch_status()?, Status::NotChanged);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_is_staged_on_staged_change() -> Result<()> {
        let dir = init_repo()?;
        dir.child("g").write_str("b\n")?;
        git(dir.path(), &["add", "g"]);
        assert_eq!(open(&dir)?.branch_status()?, Status::Staged);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_is_unstaged_on_worktree_change() -> Result<()> {
        let dir = init_repo()?;
        dir.child("f").write_str("changed\n")?;
        assert_eq!(open(&dir)?.branch_status()?, Status::Unstaged);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_prefers_unstaged_over_staged_when_both_present() -> Result<()> {
        let dir = init_repo()?;
        // A staged addition and an unstaged worktree change at the same time:
        // the worse (unstaged) status must win regardless of iteration order.
        dir.child("g").write_str("b\n")?;
        git(dir.path(), &["add", "g"]);
        dir.child("f").write_str("changed\n")?;
        assert_eq!(open(&dir)?.branch_status()?, Status::Unstaged);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_prefers_conflict_over_unstaged_when_both_present() -> Result<()> {
        let dir = init_repo()?;
        git(dir.path(), &["checkout", "-q", "-b", "other"]);
        dir.child("f").write_str("theirs\n")?;
        git(dir.path(), &["commit", "-qam", "theirs"]);
        git(dir.path(), &["checkout", "-q", "main"]);
        dir.child("f").write_str("ours\n")?;
        git(dir.path(), &["commit", "-qam", "ours"]);
        let base = git_stdout(dir.path(), &["merge-base", "main", "other"])?;
        let base_tree = format!("{base}^{{tree}}");
        git(
            dir.path(),
            &["read-tree", "-m", &base_tree, "main^{tree}", "other^{tree}"],
        );
        // An additional plain worktree change on top of the conflict must not
        // downgrade the reported status away from conflicted.
        dir.child("extra").write_str("x\n")?;
        git(dir.path(), &["add", "extra"]);
        dir.child("extra").write_str("y\n")?;
        assert_eq!(open(&dir)?.branch_status()?, Status::Conflicted);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_ignores_untracked_files() -> Result<()> {
        let dir = init_repo()?;
        dir.child("untracked").write_str("x\n")?;
        assert_eq!(open(&dir)?.branch_status()?, Status::NotChanged);
        dir.close().map_err(Into::into)
    }

    #[test]
    fn branch_status_is_conflicted_on_merge_conflict() -> Result<()> {
        let dir = init_repo()?;
        git(dir.path(), &["checkout", "-q", "-b", "other"]);
        dir.child("f").write_str("theirs\n")?;
        git(dir.path(), &["commit", "-qam", "theirs"]);
        git(dir.path(), &["checkout", "-q", "main"]);
        dir.child("f").write_str("ours\n")?;
        git(dir.path(), &["commit", "-qam", "ours"]);
        // Produce an unmerged index (stages 1-3 for `f`) with a 3-way read-tree.
        // This reproduces a merge conflict deterministically, without depending
        // on `git merge`, whose conflict behavior is environment-sensitive.
        let base = git_stdout(dir.path(), &["merge-base", "main", "other"])?;
        let base_tree = format!("{base}^{{tree}}");
        git(
            dir.path(),
            &["read-tree", "-m", &base_tree, "main^{tree}", "other^{tree}"],
        );
        assert_eq!(open(&dir)?.branch_status()?, Status::Conflicted);
        dir.close().map_err(Into::into)
    }
}
