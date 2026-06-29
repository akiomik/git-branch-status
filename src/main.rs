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

use clap::Parser;

use git_branch_status::branch::Branch;
use git_branch_status::cli::Cli;
use git_branch_status::error::Error;
use git_branch_status::repository::Repository;

fn run(cli: Cli) -> Result<String, Error> {
    let repo = Repository::discover(cli.dir)?;
    let branch = Branch {
        name: repo.branch_name()?,
        status: repo.branch_status()?,
    };
    let output = cli.mode.format(&branch);

    Ok(output)
}

fn main() {
    let cli = Cli::parse();

    match run(cli) {
        Ok(output) => print!("{output}"),
        Err(_) => exit(1),
    }
}
