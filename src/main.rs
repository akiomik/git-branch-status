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
use git2::Repository;

use git_branch_status::branch_status::BranchStatus;
use git_branch_status::repository_ext::RepositoryExt;

fn main() {
    let matches = Command::new("git-branch-status")
        .version("0.1.0")
        .author("Akiomi Kamakura <akiomik@gmail.com>")
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

    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(_) => std::process::exit(1),
    };

    let branch = match repo.branch_name() {
        Ok(branch) => branch,
        Err(e) => panic!("failed to get branch: {}", e),
    };

    let status = match repo.branch_status() {
        Ok(status) => status,
        Err(e) => panic!("failed to get branch status: {}", e),
    };

    let mode = matches.get_one::<String>("mode").unwrap().as_str();
    match mode {
        "zsh" => {
            match status {
                BranchStatus::NotChanged => print!("%F{{green}}{}%f", branch),
                BranchStatus::Staged => print!("%F{{yellow}}{}%f", branch),
                BranchStatus::Unstaged => print!("%F{{red}}{}%f", branch),
                BranchStatus::Conflicted => print!("%F{{red}}{}%f", branch),
            };
        }
        "stdout" => {
            match status {
                BranchStatus::NotChanged => print!("{}", Green.paint(branch)),
                BranchStatus::Staged => print!("{}", Yellow.paint(branch)),
                BranchStatus::Unstaged => print!("{}", Red.paint(branch)),
                BranchStatus::Conflicted => print!("{}", Red.paint(branch)),
            };
        }
        _ => panic!("unsupported mode is specified: {}", mode),
    };
}
