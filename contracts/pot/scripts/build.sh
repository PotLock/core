#!/bin/sh

echo ">> Building Pot contract"

set -e

export CARGO_TARGET_DIR=target
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir -p ./out
cp target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm