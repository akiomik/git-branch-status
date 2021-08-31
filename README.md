# git-branch-status

A command for displaying git branch colored by status.

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
