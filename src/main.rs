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

extern crate clap;

use ansi_term::Colour::{Green, Red, Yellow};
use clap::{Arg, Command};

use git_branch_status::branch_status::BranchStatus;
use git_branch_status::repository::Repository;

fn main() {
    let matches = Command::new("git-branch-status")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Show git branch name colored by status")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .default_value("stdout")
                .value_parser(["zsh", "stdout"])
                .help("Sets a mode"),
        )
        .get_matches();

    let Ok(repo) = Repository::discover(".") else {
        std::process::exit(1)
    };

    let Ok(branch) = repo.branch_name() else {
        std::process::exit(1)
    };

    let Ok(status) = repo.branch_status() else {
        std::process::exit(1)
    };

    let mode = matches.get_one::<String>("mode").unwrap().as_str();
    match mode {
        "zsh" => match status {
            BranchStatus::NotChanged => print!("%F{{green}}{branch}%f"),
            BranchStatus::Staged => print!("%F{{yellow}}{branch}%f"),
            BranchStatus::Unstaged | BranchStatus::Conflicted => print!("%F{{red}}{branch}%f"),
        },
        _ => match status {
            BranchStatus::NotChanged => print!("{}", Green.paint(branch)),
            BranchStatus::Staged => print!("{}", Yellow.paint(branch)),
            BranchStatus::Unstaged | BranchStatus::Conflicted => print!("{}", Red.paint(branch)),
        },
    }
}
