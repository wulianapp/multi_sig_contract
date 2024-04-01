#!/bin/bash
##multi_sig_test:relayer
set -xv
relayer_test=b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7
relayer_local=83a666efeed6ffd0bc54c30ad1d1b904e8e49608a7298138a85ea428ce15b902

##create implicit account 
near --nodeUrl http://46.250.225.58:8061  send node0 $relayer_test  100000 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl http://46.250.225.58:8061  send node0 $relayer_local  100000 --keyPath ~/.near-credentials/local/node0.json

##create ca
near --nodeUrl  http://46.250.225.58:8061 create-account multi_sig7_test.node0 --publicKey CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account multi_sig7_local.node0 --publicKey 9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json

##deploy ca
near --nodeUrl  http://46.250.225.58:8061 deploy multi_sig7_test.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_test.node0.json
near --nodeUrl  http://46.250.225.58:8061 deploy multi_sig7_local.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_local.node0.json

##create coin account
near --nodeUrl  http://46.250.225.58:8061 create-account  btc.node0 --publicKey G4J3YUfzKcwkshpjBSpZSjfqM7ip6oHZuMeeW5Y4oVwk   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account  eth.node0 --publicKey Ht7qqudG6gpLFonMq95v4gqVYkrbtUj4KXvxUFCBjCj8   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account  usdt.node0 --publicKey 9QF6ahwEKx8QLYknjLm22tHcebmcQXcw6eC9fAvu5oZu   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account  usdc.node0 --publicKey FJPPSCSBahWskJHRjtNDE5H68XvrZct8e7dNWcQXNhqv   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account  cly.node0 --publicKey 6F6hrmgEmGLkXv52hCmPrZhdjrSBnb4HXKkiBxJ1JkmR   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://46.250.225.58:8061 create-account  dw20.node0 --publicKey Din36NbVZ9XGX6S2UDrCxEMdEngQ6Dr1qcmYfkqACaad   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json

##deploy coin
#near --nodeUrl  http://46.250.225.58:8061 deploy btc.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/btc.node0.json
#near --nodeUrl  http://46.250.225.58:8061 deploy eth.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/eth.node0.json
#near --nodeUrl  http://46.250.225.58:8061 deploy usdt.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/usdt.node0.json
#near --nodeUrl  http://46.250.225.58:8061 deploy usdc.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/usdc.node0.json
#near --nodeUrl  http://46.250.225.58:8061 deploy cly.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/cly.node0.json
#near --nodeUrl  http://46.250.225.58:8061 deploy dw20.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/dw20.node0.json

#!/bin/bash

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")

for coin in "${coins[@]}"; do
	##test and local use same ca
	near --nodeUrl  http://46.250.225.58:8061 deploy $coin.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/$coin.node0.json
    near --nodeUrl  http://46.250.225.58:8061 call $coin.node0 new_default_meta '{"owner_id":"'$coin'.node0","total_supply":"22345678900000000"}'  --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json
    near --nodeUrl http://46.250.225.58:8061 call $coin.node0  ft_transfer '{"receiver_id":"'$relayer_test'","amount":"900000000"}' --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json
    near --nodeUrl http://46.250.225.58:8061 view $coin.node0 ft_balance_of '{"account_id":"'$relayer_test'"}'

    near --nodeUrl http://46.250.225.58:8061 call $coin.node0  ft_transfer '{"receiver_id":"'$relayer_local'","amount":"900000000"}' --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json
    near --nodeUrl http://46.250.225.58:8061 view $coin.node0 ft_balance_of '{"account_id":"'$relayer_local'"}'
done
