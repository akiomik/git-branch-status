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

/// The worst change present in the working tree.
///
/// In increasing order of severity: `Conflicted` > `Unstaged` > `Staged` >
/// `NotChanged`. This precedence is a domain decision applied explicitly by
/// [`Repository::branch_status`](crate::repository::Repository::branch_status),
/// not derived from this declaration order, so the variants can be reordered
/// freely without changing behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    NotChanged,
    Staged,
    Unstaged,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
    pub status: Status,
}
