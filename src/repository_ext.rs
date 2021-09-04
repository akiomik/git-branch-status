use git2::{Error, ErrorCode, Repository, StatusOptions};

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

        Ok(branch.to_string())
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
