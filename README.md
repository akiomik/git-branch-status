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
