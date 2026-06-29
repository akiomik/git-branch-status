default: fmt lint test

fmt:
  cargo fmt --all

lint:
  cargo clippy -- -D warnings

test:
  cargo test

samply target=".":
    cargo build --profile bench && samply record ./target/release/git-branch-status --mode zsh {{ target }}

[macos]
flamegraph target=".":
    # See https://github.com/flamegraph-rs/flamegraph#dtrace-on-macos
    cargo flamegraph --root --profile bench --open -- --mode zsh {{ target }}
