#!/bin/sh
echo ">> Building contract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release%

echo ">> Deploying contract"

near --nodeUrl  http://123.56.252.201:8061 deploy dw20.node0 ./target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ./dw20.node0.json
