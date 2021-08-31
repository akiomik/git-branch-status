extern crate clap;

use clap::{Arg, App};
use git2::{Error, ErrorCode, Repository, StatusEntry};
use ansi_term::Colour::{Green, Red, Yellow};

#[derive(PartialEq, PartialOrd)]
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

    Ok(branch.to_string())
}

fn is_conflicted(s: &StatusEntry) -> bool {
    s.status().is_conflicted()
}

fn is_unstaged(s: &StatusEntry) -> bool {
    s.status().is_wt_new() || s.status().is_wt_modified() || s.status().is_wt_deleted() || s.status().is_wt_typechange() || s.status().is_wt_renamed()
}

fn is_staged(s: &StatusEntry) -> bool {
    s.status().is_index_new() || s.status().is_index_modified() || s.status().is_index_deleted() || s.status().is_index_typechange() || s.status().is_index_renamed()
}

fn get_branch_status_of(repo: &Repository) -> Result<BranchStatus, Error> {
    let stats = match repo.statuses(None) {
        Ok(stats) => stats,
        Err(e)    => return Err(e),
    };

    let status = stats.iter().fold(BranchStatus::NotChanged, |acc, s| {
        if acc < BranchStatus::Conflicted && is_conflicted(&s) {
            BranchStatus::Conflicted
        } else if acc < BranchStatus::Unstaged && is_unstaged(&s) {
            BranchStatus::Unstaged
        } else if acc < BranchStatus::Staged && is_staged(&s) {
            BranchStatus::Staged
        } else {
            acc
        }
    });

    Ok(status)
}

fn main() {
    let matches = App::new("git-branch-status")
                      .version("0.1.0")
                      .author("Akiomi Kamakura <akiomik@gmail.com>")
                      .about("Show git branch name colored by status")
                      .arg(Arg::with_name("mode")
                           .short("m")
                           .long("mode")
                           .value_name("MODE")
                           .help("Sets a mode. Currently, `stdout` and `zsh` are supported)")
                           .takes_value(true))
                      .get_matches();

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

    let mode = matches.value_of("mode").unwrap_or("stdout");
    match mode {
        "zsh" => {
            match status {
                BranchStatus::NotChanged => print!("%F{{green}}{}%f", branch),
                BranchStatus::Staged     => print!("%F{{yellow}}{}%f", branch),
                BranchStatus::Unstaged   => print!("%F{{red}}{}%f", branch),
                BranchStatus::Conflicted => print!("%F{{red}}{}%f", branch),
            };
        },
        "stdout" => {
            match status {
                BranchStatus::NotChanged => print!("{}", Green.paint(branch)),
                BranchStatus::Staged     => print!("{}", Yellow.paint(branch)),
                BranchStatus::Unstaged   => print!("{}", Red.paint(branch)),
                BranchStatus::Conflicted => print!("{}", Red.paint(branch)),
            };
        },
        _ => panic!("unsupported mode is specified: {}", mode),
    };
}
