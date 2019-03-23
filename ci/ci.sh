#!/usr/bin/env bash

set -e

echo "Going to build (and test) documentation..."
make -C docs/ html

echo "Going to run Rust tests..."
cd rust
cargo test
cargo clippy -- -D warnings
cargo fmt --all -- --check
cargo doc --no-deps
cd ..

mkdir html
touch html/.nojekyll # docs are not based on Jekyll
cp -r docs/_build/* html/

mkdir html/rust
cp -r rust/target/doc/* html/rust/
