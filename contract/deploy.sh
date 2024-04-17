#!/bin/sh

./build.sh

echo ">> Deploying contract"

##near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm

near --nodeUrl  http://120.232.251.101:3030 deploy multi_sig7_test.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_test.node0.json
near --nodeUrl  http://120.232.251.101:3030  deploy multi_sig7_local.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_local.node0.json
