#!/bin/sh

if [ $? -ne 0 ]; then
  echo ">> Error building Lists contract"
  exit 1
fi

echo ">> Deploying Lists contract!"

# https://docs.near.org/tools/near-cli#near-dev-deploy
near dev-deploy --wasmFile ./out/main.wasm