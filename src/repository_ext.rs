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

use git2::{Error, ErrorCode, Oid, Repository, RepositoryState, StatusOptions};

use crate::branch_status::BranchStatus;
use crate::status_entry_ext::StatusEntryExt;

pub trait RepositoryExt {
    fn action(&self, state: RepositoryState) -> Option<&'static str>;
    fn branch_name(&self) -> Result<String, Error>;
    fn branch_status(&self) -> Result<BranchStatus, Error>;
    fn rebase_head_name(&self, state: RepositoryState) -> Result<Option<String>, Error>;
    fn unborn_branch_name(&self) -> Result<Option<String>, Error>;
    fn to_short_oid(&self, oid: Oid) -> Result<Option<String>, Error>;
}

impl RepositoryExt for Repository {
    fn action(&self, state: RepositoryState) -> Option<&'static str> {
        match state {
            RepositoryState::ApplyMailbox => Some("am"),
            RepositoryState::ApplyMailboxOrRebase => Some("am/rebase"),
            RepositoryState::Bisect => Some("bisect"),
            RepositoryState::CherryPick => Some("cherry"),
            RepositoryState::CherryPickSequence => Some("cherry-seq"),
            RepositoryState::Merge => Some("merge"),
            RepositoryState::Rebase => Some("rebase"),
            RepositoryState::RebaseInteractive => Some("rebase-i"),
            RepositoryState::RebaseMerge => Some("rebase-m"),
            RepositoryState::Revert => Some("revert"),
            RepositoryState::RevertSequence => Some("revert-seq"),
            _ => None,
        }
    }

    fn branch_name(&self) -> Result<String, Error> {
        let head = match self.head() {
            Ok(head) => Some(head),
            Err(ref e)
                if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound =>
            {
                None
            }
            Err(e) => return Err(e),
        };

        let detached = self.head_detached()?;
        let state = self.state();

        let branch = if let Some(name) = self.rebase_head_name(state)? {
            name
        } else if detached {
            let oid = head.as_ref().and_then(|h| h.target());
            let short = match oid.map(|oid| self.to_short_oid(oid)) {
                Some(Ok(id)) => id,
                Some(Err(e)) => return Err(e),
                None => None,
            };

            short.unwrap_or_else(|| "HEAD (detached)".to_string())
        } else if let Some(name) = head.as_ref().and_then(|h| h.shorthand().ok()) {
            name.to_string()
        } else {
            // An unborn branch (e.g. a freshly initialized repository) has no
            // HEAD commit, so resolve the branch name from the symbolic HEAD.
            self.unborn_branch_name()?
                .unwrap_or_else(|| "HEAD (no branch)".to_string())
        };

        match self.action(state) {
            Some(action) => Ok(branch + ":" + action),
            None => Ok(branch),
        }
    }

    fn branch_status(&self) -> Result<BranchStatus, Error> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(false)
            .include_ignored(false)
            .include_unmodified(false)
            .exclude_submodules(true);

        let stats = self.statuses(Some(&mut opts))?;

        let status = stats.iter().fold(BranchStatus::NotChanged, |acc, s| {
            if acc < BranchStatus::Conflicted && s.is_conflicted() {
                BranchStatus::Conflicted
            } else if acc < BranchStatus::Unstaged && s.is_unstaged() {
                BranchStatus::Unstaged
            } else if acc < BranchStatus::Staged && s.is_staged() {
                BranchStatus::Staged
            } else {
                acc
            }
        });

        Ok(status)
    }

    fn rebase_head_name(&self, state: RepositoryState) -> Result<Option<String>, Error> {
        // The original branch name is recorded in a `head-name` file while a
        // rebase is in progress. The merge backend (and interactive rebases)
        // use `rebase-merge/`, while the apply backend uses `rebase-apply/`.
        let dir = match state {
            RepositoryState::RebaseInteractive | RepositoryState::RebaseMerge => "rebase-merge",
            RepositoryState::Rebase => "rebase-apply",
            _ => return Ok(None),
        };

        let path = self.path().join(dir).join("head-name");
        let refname = match fs::read_to_string(&path) {
            Ok(content) => content.trim().to_string(),
            Err(_) => return Ok(None),
        };

        let name = match self.find_reference(&refname) {
            Ok(reference) => reference.shorthand().unwrap_or(&refname).to_string(),
            Err(_) => refname
                .strip_prefix("refs/heads/")
                .unwrap_or(&refname)
                .to_string(),
        };

        Ok(Some(name))
    }

    fn unborn_branch_name(&self) -> Result<Option<String>, Error> {
        let reference = self.find_reference("HEAD")?;
        Ok(reference.symbolic_target()?.map(|target| {
            target
                .strip_prefix("refs/heads/")
                .unwrap_or(target)
                .to_string()
        }))
    }

    fn to_short_oid(&self, oid: Oid) -> Result<Option<String>, Error> {
        let object = self.find_object(oid, None)?;
        match object.short_id() {
            Ok(id) => Ok(id.as_str().map(|i| i.to_string()).ok()),
            Err(e) => Err(e),
        }
    }
}
