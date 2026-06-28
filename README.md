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

```toml
command_timeout = 1000

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

Starship aborts any custom command that runs longer than
[`command_timeout`](https://starship.rs/config/#prompt) (500ms by default) and
shows a warning instead of its output. In large repositories `git-branch-status`
can take longer than that, so raise `command_timeout` (e.g. to 1000ms) if the
module disappears or you see a timeout warning.

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
