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

use std::process::exit;

use ansi_term::Colour::{Green, Red, Yellow};
use clap::Parser;

use git_branch_status::branch_status::BranchStatus;
use git_branch_status::cli::Cli;
use git_branch_status::mode::Mode;
use git_branch_status::repository::Repository;

fn main() {
    let cli = Cli::parse();

    let Ok(repo) = Repository::discover(cli.dir) else {
        exit(1)
    };

    let Ok(branch) = repo.branch_name() else {
        exit(1)
    };

    let Ok(status) = repo.branch_status() else {
        exit(1)
    };

    match cli.mode {
        Mode::Stdout => match status {
            BranchStatus::NotChanged => print!("{}", Green.paint(branch)),
            BranchStatus::Staged => print!("{}", Yellow.paint(branch)),
            BranchStatus::Unstaged | BranchStatus::Conflicted => print!("{}", Red.paint(branch)),
        },
        Mode::Zsh => match status {
            BranchStatus::NotChanged => print!("%F{{green}}{branch}%f"),
            BranchStatus::Staged => print!("%F{{yellow}}{branch}%f"),
            BranchStatus::Unstaged | BranchStatus::Conflicted => print!("%F{{red}}{branch}%f"),
        },
    }
}
