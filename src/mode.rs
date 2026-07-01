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

use ansi_term::Colour::{Green, Red, Yellow};
use clap::ValueEnum;

use crate::branch::{Branch, Status};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
pub enum Mode {
    Stdout,
    Zsh,
}

impl Mode {
    fn format_stdout(branch: &Branch) -> String {
        let color = match branch.status {
            Status::NotChanged => Green,
            Status::Staged => Yellow,
            Status::Unstaged | Status::Conflicted => Red,
        };
        format!("{}", color.paint(branch.name.as_str()))
    }

    fn format_zsh(branch: &Branch) -> String {
        let color = match branch.status {
            Status::NotChanged => "green",
            Status::Staged => "yellow",
            Status::Unstaged | Status::Conflicted => "red",
        };
        let name = branch.name.replace('%', "%%");
        format!("%F{{{color}}}{name}%f")
    }

    #[must_use]
    pub fn format(&self, branch: &Branch) -> String {
        match self {
            Self::Stdout => Self::format_stdout(branch),
            Self::Zsh => Self::format_zsh(branch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdout_not_changed() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::NotChanged,
        };
        let actual = Mode::Stdout.format(&branch);
        assert_eq!(actual, format!("{}", Green.paint("main")));
    }

    #[test]
    fn test_stdout_staged() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Staged,
        };
        let actual = Mode::Stdout.format(&branch);
        assert_eq!(actual, format!("{}", Yellow.paint("main")));
    }

    #[test]
    fn test_stdout_unstaged() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Unstaged,
        };
        let actual = Mode::Stdout.format(&branch);
        assert_eq!(actual, format!("{}", Red.paint("main")));
    }

    #[test]
    fn test_stdout_conflicted() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Conflicted,
        };
        let actual = Mode::Stdout.format(&branch);
        assert_eq!(actual, format!("{}", Red.paint("main")));
    }

    #[test]
    fn test_zsh_not_changed() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::NotChanged,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{green}main%f");
    }

    #[test]
    fn test_zsh_staged() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Staged,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{yellow}main%f");
    }

    #[test]
    fn test_zsh_unstaged() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Unstaged,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{red}main%f");
    }

    #[test]
    fn test_zsh_conflicted() {
        let branch = Branch {
            name: "main".to_owned(),
            status: Status::Conflicted,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{red}main%f");
    }

    #[test]
    fn test_zsh_escapes_percent_in_branch_name() {
        let branch = Branch {
            name: "feature/%n".to_owned(),
            status: Status::NotChanged,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{green}feature/%%n%f");
    }

    #[test]
    fn test_zsh_escapes_trailing_percent_in_branch_name() {
        let branch = Branch {
            name: "main%".to_owned(),
            status: Status::NotChanged,
        };
        let actual = Mode::Zsh.format(&branch);
        assert_eq!(actual, "%F{green}main%%%f");
    }
}
