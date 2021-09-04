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

use git2::StatusEntry;

pub trait StatusEntryExt {
    fn is_conflicted(&self) -> bool;
    fn is_unstaged(&self) -> bool;
    fn is_staged(&self) -> bool;
}

impl StatusEntryExt for StatusEntry<'_> {
    fn is_conflicted(&self) -> bool {
        self.status().is_conflicted()
    }

    // ignores is_wt_new
    fn is_unstaged(&self) -> bool {
        self.status().is_wt_modified()
            || self.status().is_wt_deleted()
            || self.status().is_wt_typechange()
            || self.status().is_wt_renamed()
    }

    fn is_staged(&self) -> bool {
        self.status().is_index_new()
            || self.status().is_index_modified()
            || self.status().is_index_deleted()
            || self.status().is_index_typechange()
            || self.status().is_index_renamed()
    }
}
