#!/usr/bin/env bash

set -e

echo "Going to build (and test) documentation..."
make -C docs/ html

echo "Going to run Rust tests..."
cd rust
cargo test
cargo clippy -- -D warnings
cargo fmt --all -- --check
cd ..
