#!/bin/bash

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")

for coin in "${coins[@]}"; do
   ## echo "$coin"|xargs -I {} near --nodeUrl  http://123.56.252.201:8061 deploy {}.multi_sig7_test.node0 ./target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ./coin_key/{}.multi_sig7_test.node0.json
   ## echo "$coin" |xargs -I {} near --nodeUrl  http://123.56.252.201:8061 call {}.multi_sig7_test.node0 new_default_meta '{"owner_id":"{}.multi_sig7_test.node0","total_supply":"22345678900000000"}'  --accountId {}.multi_sig7_test.node0 --keyPath ./coin_key/{}.multi_sig7_test.node0.json
    echo "$coin" |xargs -I {} near --nodeUrl http://123.56.252.201:8061 call {}.node0  ft_transfer '{"receiver_id":"b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7","amount":"900000000"}' --accountId {}.node0 --keyPath ./coin_key/{}.node0.json
    echo "$coin" |xargs -I {} near --nodeUrl http://123.56.252.201:8061 view {}.node0 ft_balance_of '{"account_id":"b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7"}'


done
