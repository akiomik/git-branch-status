#[derive(PartialEq, PartialOrd)]
pub enum BranchStatus {
    NotChanged,
    Staged,
    Unstaged,
    Conflicted,
}
