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
use gix::state::InProgress;
use gix::status::Item as StatusItem;
use gix::status::UntrackedFiles;
use gix::status::index_worktree::Item as IndexWorktreeItem;
use gix::status::plumbing::index_as_worktree::EntryStatus;

use crate::branch_status::BranchStatus;

/// The single domain error for the `gix` backend.
///
/// The many per-operation `gix` error types (some of which are large) are boxed
/// behind this one type, so the rest of the crate never names a `gix` type and
/// the `Result` stays small.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(Box<dyn std::error::Error + Send + Sync + 'static>);

macro_rules! impl_from_gix_error {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for Error {
                fn from(err: $ty) -> Self {
                    Error(Box::new(err))
                }
            }
        )+
    };
}

impl_from_gix_error!(
    gix::discover::Error,
    gix::reference::find::existing::Error,
    gix::status::Error,
    gix::status::into_iter::Error,
    gix::status::iter::Error,
);

/// A thin wrapper over [`gix::Repository`] exposing only the operations this tool
/// needs, keeping all `gix` types contained to this module.
pub struct Repository(gix::Repository);

impl Repository {
    /// Discover a repository starting from `path` and walking up to the root.
    pub fn discover(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self(gix::discover(path)?))
    }

    /// The branch name to display, optionally suffixed with the in-progress
    /// action (e.g. `main:rebase-i`).
    pub fn branch_name(&self) -> Result<String, Error> {
        let head = self.0.head()?;
        let state = self.0.state();

        let branch = if let Some(name) = self.rebase_head_name() {
            // A rebase records the original branch name on disk; prefer it over
            // the detached HEAD that a rebase leaves behind.
            name
        } else {
            match &head.kind {
                gix::head::Kind::Symbolic(reference) => reference.name.shorten().to_string(),
                gix::head::Kind::Unborn(name) => Self::shorthand(name.as_ref()),
                gix::head::Kind::Detached { target, .. } => {
                    // Prefer a tag pointing at HEAD, falling back to the short hash.
                    self.tag_name()
                        .or_else(|| self.short_id(*target))
                        .unwrap_or_else(|| "HEAD (detached)".to_string())
                }
            }
        };

        match Self::action(state) {
            Some(action) => Ok(branch + ":" + action),
            None => Ok(branch),
        }
    }

    /// The worst status across the working tree, ignoring untracked files.
    pub fn branch_status(&self) -> Result<BranchStatus, Error> {
        let iter = self
            .0
            .status(gix::progress::Discard)?
            .untracked_files(UntrackedFiles::None)
            .into_iter(Vec::<BString>::new())?;

        let mut status = BranchStatus::NotChanged;
        for item in iter {
            let next = match item? {
                // HEAD <-> index: a staged change.
                StatusItem::TreeIndex(_) => BranchStatus::Staged,
                // index <-> worktree: an unstaged change or a conflict.
                StatusItem::IndexWorktree(IndexWorktreeItem::Modification {
                    status: entry,
                    ..
                }) => match entry {
                    EntryStatus::Conflict { .. } => BranchStatus::Conflicted,
                    EntryStatus::Change(_) => BranchStatus::Unstaged,
                    // Stat-only refresh or `--intent-to-add`: nothing changed.
                    EntryStatus::NeedsUpdate(_) | EntryStatus::IntentToAdd => continue,
                },
                // A rename detected against the index counts as unstaged.
                StatusItem::IndexWorktree(IndexWorktreeItem::Rewrite { .. }) => {
                    BranchStatus::Unstaged
                }
                // Untracked entries are excluded by `UntrackedFiles::None`.
                StatusItem::IndexWorktree(IndexWorktreeItem::DirectoryContents { .. }) => continue,
            };

            if next > status {
                status = next;
            }
            if status == BranchStatus::Conflicted {
                break;
            }
        }

        Ok(status)
    }

    /// Map an in-progress operation to the suffix shown after the branch name.
    fn action(state: Option<InProgress>) -> Option<&'static str> {
        match state? {
            InProgress::ApplyMailbox => Some("am"),
            InProgress::ApplyMailboxRebase => Some("am/rebase"),
            InProgress::Bisect => Some("bisect"),
            InProgress::CherryPick => Some("cherry"),
            InProgress::CherryPickSequence => Some("cherry-seq"),
            InProgress::Merge => Some("merge"),
            InProgress::Rebase => Some("rebase"),
            InProgress::RebaseInteractive => Some("rebase-i"),
            InProgress::Revert => Some("revert"),
            InProgress::RevertSequence => Some("revert-seq"),
        }
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

    fn shorthand(name: &gix::refs::FullNameRef) -> String {
        name.shorten().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Repository;
    use crate::branch_status::BranchStatus;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    /// Run `git` in `dir`, neutralizing the user's global signing config and
    /// pinning identity so fixtures are hermetic.
    fn git(dir: &Path, args: &[&str]) {
        let mut full = vec!["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"];
        full.extend_from_slice(args);
        let out = Command::new("git")
            .current_dir(dir)
            .args(&full)
            .env("GIT_AUTHOR_NAME", "tester")
            .env("GIT_AUTHOR_EMAIL", "tester@example.com")
            .env("GIT_COMMITTER_NAME", "tester")
            .env("GIT_COMMITTER_EMAIL", "tester@example.com")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn git_stdout(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// A repository with a single committed file on `main`.
    fn init_repo() -> TempDir {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        fs::write(dir.path().join("f"), "a\n").unwrap();
        git(dir.path(), &["add", "f"]);
        git(dir.path(), &["commit", "-qm", "init"]);
        dir
    }

    fn open(dir: &TempDir) -> Repository {
        Repository::discover(dir.path()).unwrap()
    }

    #[test]
    fn branch_name_returns_branch_on_unborn_branch() {
        let dir = TempDir::new().unwrap();
        git(dir.path(), &["init", "-q", "-b", "main"]);
        assert_eq!(open(&dir).branch_name().unwrap(), "main");
    }

    #[test]
    fn branch_name_returns_branch_on_normal_branch() {
        let dir = init_repo();
        assert_eq!(open(&dir).branch_name().unwrap(), "main");
    }

    #[test]
    fn branch_name_returns_short_hash_on_detached_head() {
        let dir = init_repo();
        git(dir.path(), &["checkout", "-q", "--detach", "HEAD"]);
        let short = git_stdout(dir.path(), &["rev-parse", "--short", "HEAD"]);
        assert_eq!(open(&dir).branch_name().unwrap(), short);
    }

    #[test]
    fn branch_name_returns_lightweight_tag_on_detached_head_at_tag() {
        let dir = init_repo();
        git(dir.path(), &["tag", "v1.0.0"]);
        git(dir.path(), &["checkout", "-q", "--detach", "v1.0.0"]);
        assert_eq!(open(&dir).branch_name().unwrap(), "v1.0.0");
    }

    #[test]
    fn branch_name_returns_annotated_tag_on_detached_head_at_tag() {
        let dir = init_repo();
        git(dir.path(), &["tag", "-a", "v2.0.0", "-m", "rel"]);
        git(dir.path(), &["checkout", "-q", "--detach", "v2.0.0"]);
        assert_eq!(open(&dir).branch_name().unwrap(), "v2.0.0");
    }

    #[test]
    fn branch_name_appends_merge_action_during_merge() {
        let dir = init_repo();
        // gix reports the `Merge` state when MERGE_HEAD exists. Fabricate it
        // directly rather than via `git merge`, whose conflict behavior is
        // environment-sensitive (e.g. it can no-op on some CI runners).
        let head = git_stdout(dir.path(), &["rev-parse", "HEAD"]);
        fs::write(
            dir.path().join(".git").join("MERGE_HEAD"),
            format!("{head}\n"),
        )
        .unwrap();
        assert_eq!(open(&dir).branch_name().unwrap(), "main:merge");
    }

    #[test]
    fn branch_name_uses_rebase_merge_head_name_during_interactive_rebase() {
        let dir = init_repo();
        let rebase_dir = dir.path().join(".git").join("rebase-merge");
        fs::create_dir_all(&rebase_dir).unwrap();
        fs::write(rebase_dir.join("head-name"), "refs/heads/feature\n").unwrap();
        fs::write(rebase_dir.join("interactive"), "").unwrap();
        assert_eq!(open(&dir).branch_name().unwrap(), "feature:rebase-i");
    }

    #[test]
    fn branch_name_uses_rebase_apply_head_name_during_apply_rebase() {
        let dir = init_repo();
        let rebase_dir = dir.path().join(".git").join("rebase-apply");
        fs::create_dir_all(&rebase_dir).unwrap();
        fs::write(rebase_dir.join("head-name"), "refs/heads/feature\n").unwrap();
        fs::write(rebase_dir.join("rebasing"), "").unwrap();
        assert_eq!(open(&dir).branch_name().unwrap(), "feature:rebase");
    }

    #[test]
    fn branch_status_is_not_changed_on_clean_tree() {
        let dir = init_repo();
        assert_eq!(
            open(&dir).branch_status().unwrap(),
            BranchStatus::NotChanged
        );
    }

    #[test]
    fn branch_status_is_staged_on_staged_change() {
        let dir = init_repo();
        fs::write(dir.path().join("g"), "b\n").unwrap();
        git(dir.path(), &["add", "g"]);
        assert_eq!(open(&dir).branch_status().unwrap(), BranchStatus::Staged);
    }

    #[test]
    fn branch_status_is_unstaged_on_worktree_change() {
        let dir = init_repo();
        fs::write(dir.path().join("f"), "changed\n").unwrap();
        assert_eq!(open(&dir).branch_status().unwrap(), BranchStatus::Unstaged);
    }

    #[test]
    fn branch_status_ignores_untracked_files() {
        let dir = init_repo();
        fs::write(dir.path().join("untracked"), "x\n").unwrap();
        assert_eq!(
            open(&dir).branch_status().unwrap(),
            BranchStatus::NotChanged
        );
    }

    #[test]
    fn branch_status_is_conflicted_on_merge_conflict() {
        let dir = init_repo();
        git(dir.path(), &["checkout", "-q", "-b", "other"]);
        fs::write(dir.path().join("f"), "theirs\n").unwrap();
        git(dir.path(), &["commit", "-qam", "theirs"]);
        git(dir.path(), &["checkout", "-q", "main"]);
        fs::write(dir.path().join("f"), "ours\n").unwrap();
        git(dir.path(), &["commit", "-qam", "ours"]);
        // Produce an unmerged index (stages 1-3 for `f`) with a 3-way read-tree.
        // This reproduces a merge conflict deterministically, without depending
        // on `git merge`, whose conflict behavior is environment-sensitive.
        let base = git_stdout(dir.path(), &["merge-base", "main", "other"]);
        let base_tree = format!("{base}^{{tree}}");
        git(
            dir.path(),
            &["read-tree", "-m", &base_tree, "main^{tree}", "other^{tree}"],
        );
        assert_eq!(
            open(&dir).branch_status().unwrap(),
            BranchStatus::Conflicted
        );
    }
}
