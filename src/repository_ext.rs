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

use git2::{Error, ErrorCode, Repository, RepositoryState, StatusOptions};

use crate::branch_status::BranchStatus;
use crate::status_entry_ext::StatusEntryExt;

pub trait RepositoryExt {
    fn branch_name(&self) -> Result<String, Error>;
    fn branch_status(&self) -> Result<BranchStatus, Error>;
}

impl RepositoryExt for Repository {
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

        let branch = head
            .as_ref()
            .and_then(|h| h.shorthand())
            .unwrap_or("HEAD (no branch)");

        let action = match self.state() {
            RepositoryState::RebaseInteractive => ":rebase",
            RepositoryState::Merge => ":merge",
            _ => "",
        };

        Ok(branch.to_string() + action)
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
}
