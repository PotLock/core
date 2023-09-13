#!/bin/sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying Pot contract"

# https://docs.near.org/tools/near-cli#near-dev-deploy
near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm