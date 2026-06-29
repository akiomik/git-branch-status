# git-branch-status

[![Rust CI](https://github.com/akiomik/git-branch-status/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/akiomik/git-branch-status/actions/workflows/rust-ci.yml)
[![codecov](https://codecov.io/gh/akiomik/git-branch-status/graph/badge.svg?token=3DQ0VAV2NP)](https://codecov.io/gh/akiomik/git-branch-status)

A command line tool for displaying git branch colored by status, like zsh's [vcs_info](https://zsh.sourceforge.io/Doc/Release/User-Contributions.html#Version-Control-Information).

![screenshot](screenshot.png?raw=true)

## Installation

### From crates.io

```sh
cargo install git-branch-status
```

### Prebuilt binaries

Download the archive for your platform from the
[releases page](https://github.com/akiomik/git-branch-status/releases), extract
it, and place the `git-branch-status` binary somewhere on your `PATH` (Windows
builds are distributed as a `.zip`):

```sh
tar xzf git-branch-status-<version>-<target>.tar.gz
cp git-branch-status ~/bin
```

### From source

```sh
git clone https://github.com/akiomik/git-branch-status.git && cd git-branch-status
cargo build --release
cp target/release/git-branch-status ~/bin
```

## Usage

### Zsh

Add the following to `~/.zshrc`:

```sh
# ~/.zshrc
setopt prompt_subst
RPROMPT='$(git branch-status --mode zsh)'
```

### Zsh with Starship 🚀

Add the following to `~/.config/starship.toml`:

```toml
format = """
$directory\
$custom\
$line_break\
$character"""

[custom.branchstatus]
command = "git branch-status --mode zsh"
when = "git rev-parse --is-inside-work-tree 2>/dev/null"
format = " on $output"
```

## Benchmark

### Against vcs_info

Run `./scripts/bench-vcs-info.sh`. `git-branch-status` is about 7x faster than `vcs_info` on M1 MacBook Pro (2021).

```sh
❯ ./scripts/bench-vcs-info.sh
Setup vcs_info...done!
Setup git-branch-status...done!

Run 'vcs_info; echo $vcs_info_msg_0_' 100 times
....................................................................................................done!
Elapsed time: 7127ms

Run './target/release/git-branch-status --mode zsh' 100 times
....................................................................................................done!
Elapsed time: 1061ms
```

### git-branch-status on its own

Run `./scripts/bench.sh` to benchmark `git-branch-status` alone with
[hyperfine](https://github.com/sharkdp/hyperfine). This is handy when working on
performance, since it reports the mean, standard deviation, and min/max across
many runs.

```sh
❯ ./scripts/bench.sh
Building git-branch-status...
Benchmark 1: ./target/release/git-branch-status --mode zsh
  Time (mean ± σ):       7.1 ms ±   0.4 ms    [User: 2.3 ms, System: 3.7 ms]
  Range (min … max):     6.3 ms …   8.5 ms    418 runs
```
