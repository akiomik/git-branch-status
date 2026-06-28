# git-branch-status

A command line tool for displaying git branch colored by status, like zsh's [vcs_info](https://zsh.sourceforge.io/Doc/Release/User-Contributions.html#Version-Control-Information).

![screenshot](screenshot.png?raw=true)

## Installation

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

```sh
format = """
$directory\
$custom\
$line_break\
$character"""

[custom.branchstatus]
command = "git branch-status --mode zsh"
when = "[[ -d .git ]] || [[ `git rev-parse --git-dir > /dev/null 2>&1; echo $?` -eq 0 ]]"
format = " on $output"
```

## Benchmark

### Against vcs_info

Run `./scripts/bench-vcs-info.sh`. `git-branch-status` is about 5x faster than `vcs_info` on M1 MacBook Pro (2021).

```sh
❯ ./scripts/bench-vcs-info.sh
Setup vcs_info...done!
Setup git-branch-status...done!

Run 'vcs_info; echo $vcs_info_msg_0_' 100 times
....................................................................................................done!
Elapsed time: 2029ms

Run './target/release/git-branch-status --mode zsh' 100 times
....................................................................................................done!
Elapsed time: 404ms
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
  Time (mean ± σ):       8.2 ms ±   0.4 ms    [User: 2.8 ms, System: 2.9 ms]
  Range (min … max):     7.7 ms …  10.7 ms    327 runs
```
