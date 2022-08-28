cargo check --all-targets --all-features --verbose
cargo clippy --all-targets --all-features -- -D warnings
cargo test --doc
cargo fmt -- --check