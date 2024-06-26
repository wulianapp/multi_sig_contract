#!/bin/sh

./build.sh

echo ">> Deploying contract"

##near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm

near --nodeUrl  http://120.232.251.101:29162 deploy test.multiwallet.chainless ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ../tmp/test.multiwallet.chainless.json
near --nodeUrl  http://120.232.251.101:29162  deploy local.multiwallet.chainless ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ../tmp/local.multiwallet.chainless.json
