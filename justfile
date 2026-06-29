default: fmt lint test

fmt:
  cargo fmt --all

lint:
  cargo clippy -- -D warnings

test:
  cargo test
