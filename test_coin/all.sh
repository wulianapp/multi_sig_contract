#!/bin/bash

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")

for coin in "${coins[@]}"; do
   ## echo "$coin"|xargs -I {} near --nodeUrl  http://123.56.252.201:8061 deploy {}.node0 ./target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ./coin_key/{}.node0.json
   ## echo "$coin" |xargs -I {} near --nodeUrl  http://123.56.252.201:8061 call {}.node0 new_default_meta '{"owner_id":"{}.node0","total_supply":"22345678900000000"}'  --accountId {}.node0 --keyPath ./coin_key/{}.node0.json
    echo "$coin" |xargs -I {} near --nodeUrl http://123.56.252.201:8061 call {}.node0  ft_transfer '{"receiver_id":"node0","amount":"900000000"}' --accountId {}.node0 --keyPath ./coin_key/{}.node0.json
    echo "$coin" |xargs -I {} near --nodeUrl http://123.56.252.201:8061 view {}.node0 ft_balance_of '{"account_id":"node0"}'


done
