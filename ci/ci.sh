#!/usr/bin/env bash

set -e

echo "\n\n"
echo "Going to build (and test) documentation"
echo "=======================================\n"
make -C docs/ html

echo "\n\n"
echo "Going to run Rust tests"
echo "=======================\n"
cd rust
cargo --locked test
cargo clippy -- -D warnings
cargo fmt --all -- --check
cargo doc --no-deps
cd ..

echo "\n\n"
echo "Going to cross-compile for Raspberry Pi"
echo "=======================================\n"
cd rust
PKG_CONFIG_ALLOW_CROSS=1 cargo build --target=armv7-unknown-linux-gnueabihf --release
cd ..

echo "\n\n"
echo "Going to prepare deployment artifacts"
echo "=====================================\n"
mkdir html
touch html/.nojekyll # docs are not based on Jekyll
cp -r docs/_build/* html/

mkdir html/rust
cp -r rust/target/doc/* html/rust/

echo $TRAVIS_COMMIT > latest.txt
ARTIFACTS_DIR=artifacts/commits/$TRAVIS_COMMIT
mkdir -p $ARTIFACTS_DIR
cp rust/target/armv7-unknown-linux-gnueabihf/release/irro-cli $ARTIFACTS_DIR/irro-cli
