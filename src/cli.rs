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

use std::path::PathBuf;

use clap::{Parser, ValueHint};

#[derive(Parser)]
#[command(
    name = "git-branch-status",
    bin_name = "git branch-status",
    version,
    author,
    about = "Show git branch name colored by status",
    long_about = None,
)]
#[non_exhaustive]
pub struct Cli {
    /// Sets a mode
    #[arg(short, long, value_parser = ["zsh", "stdout"], default_value = "stdout")]
    pub mode: String,

    /// Path to the git repository (default: current directory)
    #[arg(value_name = "DIR", value_hint = ValueHint::DirPath, default_value = ".")]
    pub dir: PathBuf,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory as _;

    use super::*;

    #[test]
    fn command() {
        Cli::command().debug_assert();
    }
}
