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
    fn action(&self) -> Option<&str>;
    fn branch_name(&self) -> Result<String, Error>;
    fn branch_status(&self) -> Result<BranchStatus, Error>;
    fn rebase_i_head_name(&self) -> Result<String, Error>;
    fn to_short_oid(&self, oid: Oid) -> Result<Option<String>, Error>;
}

impl RepositoryExt for Repository {
    fn action(&self) -> Option<&str> {
        match self.state() {
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

        let detached = match self.head_detached() {
            Ok(detached) => detached,
            Err(e) => return Err(e),
        };

        let branch = if self.state() == RepositoryState::RebaseInteractive {
            match self.rebase_i_head_name() {
                Ok(name) => name,
                Err(e) => return Err(e),
            }
        } else if detached {
            let oid = head.as_ref().and_then(|h| h.target());
            let short = match oid.and_then(|oid| Some(self.to_short_oid(oid))) {
                Some(Ok(id)) => id,
                Some(Err(e)) => return Err(e),
                None => None,
            };

            short.unwrap_or("HEAD (detached)".to_string())
        } else {
            head.as_ref()
                .and_then(|h| h.shorthand())
                .unwrap_or("HEAD (no branch)")
                .to_string()
        };

        match self.action() {
            Some(action) => Ok(branch.to_string() + ":" + action),
            None => Ok(branch.to_string()),
        }
    }

    fn branch_status(&self) -> Result<BranchStatus, Error> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(false)
            .include_ignored(false)
            .include_unmodified(false)
            .exclude_submodules(true);

        let stats = match self.statuses(Some(&mut opts)) {
            Ok(stats) => stats,
            Err(e) => return Err(e),
        };

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

    fn rebase_i_head_name(&self) -> Result<String, Error> {
        let path = self.path().join("rebase-merge").join("head-name");

        let refname = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => return Err(Error::from_str(&e.to_string())),
        };

        match self.find_reference(&refname.trim()) {
            Ok(ref reference) => Ok(reference.shorthand().unwrap_or(&refname).to_string()),
            Err(e) => return Err(e),
        }
    }

    fn to_short_oid(&self, oid: Oid) -> Result<Option<String>, Error> {
        let object = match self.find_object(oid, None) {
            Ok(object) => object,
            Err(e) => return Err(e),
        };
        match object.short_id() {
            Ok(id) => Ok(id.as_str().and_then(|i| Some(i.to_string()))),
            Err(e) => Err(e),
        }
    }
}
