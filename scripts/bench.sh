#!/bin/sh

# Copyright 2021 Akiomi Kamakura
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Benchmark git-branch-status on its own using hyperfine. Use this when
# working on performance to compare before/after with reliable statistics.
# For a comparison against zsh's vcs_info, see ./scripts/bench-vcs-info.sh.

set -e

GIT_BRANCH_STATUS="./target/release/git-branch-status"

echo "Building git-branch-status..."
cargo build --release > /dev/null 2>&1

hyperfine --warmup 10 --shell=none "$GIT_BRANCH_STATUS --mode zsh"
