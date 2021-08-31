#!/bin/zsh

DATE="gdate"
GIT_BRANCH_STATUS="./target/release/git-branch-status"

function timestamp() {
  echo $(($($DATE +%s%0N)/1000000))
}

function bench() {
  cmd="$1"
  n="$2"

  echo "Run '$cmd' $n times"
  start=`timestamp`
  for i in `seq $n`; do
    echo -n "."
    eval "$cmd" > /dev/null
  done
  echo "done!"
  stop=`timestamp`
  echo "Elapsed time: $(($stop-$start))ms"
}

echo -n 'Setup vcs_info'
autoload -Uz vcs_info
zstyle ':vcs_info:git:*' check-for-changes true
zstyle ':vcs_info:git:*' stagedstr     '%F{yellow}'           # %c
zstyle ':vcs_info:git:*' unstagedstr   '%F{red}'              # %u
zstyle ':vcs_info:*'     formats       '%F{green}%c%u%b%f'    # %b: branch
zstyle ':vcs_info:*'     actionformats '%F{green}%c%u%b:%a%f' # %a: action
echo '...done!'

echo -n 'Setup git-branch-status'
cargo build --release > /dev/null 2>&1
echo '...done!'

echo
bench 'vcs_info; echo $vcs_info_msg_0_' 100
echo
bench "$GIT_BRANCH_STATUS --mode zsh" 100
