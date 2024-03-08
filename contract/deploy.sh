#!/bin/sh

./build.sh

echo ">> Deploying contract"

##near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm

near --nodeUrl  http://123.56.252.201:8061 deploy multi_sig6.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig6.node0.json
