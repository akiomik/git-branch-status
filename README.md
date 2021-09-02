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

## Benchmark

Run `./scripts/bench.sh`. `git-branch-status` is about 3.3x faster than `vcs_info`.

```sh
❯ ./scripts/bench.sh
Setup vcs_info...done!
Setup git-branch-status...done!

Run 'vcs_info; echo $vcs_info_msg_0_' 100 times
....................................................................................................done!
Elapsed time: 6793ms

Run './target/release/git-branch-status --mode zsh' 100 times
....................................................................................................done!
Elapsed time: 2002ms
```
