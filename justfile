set dotenv-load := false

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

check:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

build:
    cargo build --workspace

validate-cases:
    cargo run -p xtask -- validate-cases cases

gate: fmt-check check test build validate-cases
