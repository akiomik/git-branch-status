default: fmt build lint test

build:
  cargo build

fmt:
  cargo fmt --all

lint:
  cargo clippy -- -D warnings

test:
  cargo test

doc-check:
  RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

samply target=".":
  cargo build --profile bench && samply record ./target/release/git-branch-status --mode zsh {{ target }}

[macos]
flamegraph target=".":
  # See https://github.com/flamegraph-rs/flamegraph#dtrace-on-macos
  cargo flamegraph --root --profile bench --open -- --mode zsh {{ target }}
