use git2::{Error, ErrorCode, Repository};
use ansi_term::Colour::{Green, Red, Yellow};

enum BranchStatus {
    NotChanged,
    Staged,
    Unstaged,
    Conflicted,
}

fn get_branch_of(repo: &Repository) -> Result<String, Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => return Err(e),
    };

    let branch = head.as_ref().and_then(|h| h.shorthand()).unwrap_or("HEAD (no branch)");

    return Ok(branch.to_string());
}

fn get_branch_status_of(repo: &Repository) -> Result<BranchStatus, Error> {
    let stats = match repo.statuses(None) {
        Ok(stats) => stats,
        Err(e)    => return Err(e),
    };

    let status = stats.iter().fold(BranchStatus::NotChanged, |acc, s|
        if s.status().is_conflicted() {
            BranchStatus::Conflicted
        } else if s.status().is_wt_new() || s.status().is_wt_modified() || s.status().is_wt_deleted() || s.status().is_wt_typechange() || s.status().is_wt_renamed() {
            BranchStatus::Unstaged
        } else if s.status().is_index_new() || s.status().is_index_modified() || s.status().is_index_deleted() || s.status().is_index_typechange() || s.status().is_index_renamed() {
            BranchStatus::Staged
        } else {
            acc
        }
    );

    return Ok(status);
}

fn main() {
    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(_)   => std::process::exit(1),
    };

    let branch = match get_branch_of(&repo) {
        Ok(branch) => branch,
        Err(e)     => panic!("failed to get branch: {}", e),
    };

    let status = match get_branch_status_of(&repo) {
        Ok(status) => status,
        Err(e)     => panic!("failed to get branch status: {}", e),
    };

    match status {
        BranchStatus::NotChanged => print!("{}", Green.paint(branch)),
        BranchStatus::Staged     => print!("{}", Yellow.paint(branch)),
        BranchStatus::Unstaged   => print!("{}", Red.paint(branch)),
        BranchStatus::Conflicted => print!("{}", Red.paint(branch)),
    };
}
