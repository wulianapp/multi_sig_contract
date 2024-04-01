#!/bin/bash
##multi_sig_test:relayer
set -xv
relayer_test=b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7
relayer_local=83a666efeed6ffd0bc54c30ad1d1b904e8e49608a7298138a85ea428ce15b902

##create ca
near --nodeUrl  http://120.232.251.101:3030 create-account multi_sig7_test.node0 --publicKey CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account multi_sig7_local.node0 --publicKey 9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
    near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"multi_sig7_test.node0","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
    near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"multi_sig7_local.node0","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000

near --nodeUrl  http://120.232.251.101:3030 create-account  btc.node0 --publicKey G4J3YUfzKcwkshpjBSpZSjfqM7ip6oHZuMeeW5Y4oVwk   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account  eth.node0 --publicKey Ht7qqudG6gpLFonMq95v4gqVYkrbtUj4KXvxUFCBjCj8   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account  usdt.node0 --publicKey 9QF6ahwEKx8QLYknjLm22tHcebmcQXcw6eC9fAvu5oZu   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account  usdc.node0 --publicKey FJPPSCSBahWskJHRjtNDE5H68XvrZct8e7dNWcQXNhqv   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account  cly.node0 --publicKey 6F6hrmgEmGLkXv52hCmPrZhdjrSBnb4HXKkiBxJ1JkmR   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account  dw20.node0 --publicKey Din36NbVZ9XGX6S2UDrCxEMdEngQ6Dr1qcmYfkqACaad   --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
##deploy ca
near --nodeUrl  http://120.232.251.101:3030 deploy multi_sig7_test.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_test.node0.json 
near --nodeUrl  http://120.232.251.101:3030 deploy multi_sig7_local.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_local.node0.json 

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")

for coin in "${coins[@]}"; do
	##test and local use same ca
    near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"'$coin'.node0","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
	near --nodeUrl  http://120.232.251.101:3030 deploy $coin.node0 ../test_coin/target/wasm32-unknown-unknown/release/fungible_token.wasm  --keyPath ../test_coin/coin_key/$coin.node0.json 
    near --nodeUrl  http://120.232.251.101:3030 call $coin.node0 new_default_meta '{"owner_id":"'$coin'.node0","total_supply":"22345678900000000"}'  --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 call $coin.node0  ft_transfer '{"receiver_id":"'$relayer_test'","amount":"900000000"}' --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin.node0 ft_balance_of '{"account_id":"'$relayer_test'"}'

    near --nodeUrl http://120.232.251.101:3030 call $coin.node0  ft_transfer '{"receiver_id":"'$relayer_local'","amount":"900000000"}' --accountId $coin.node0 --keyPath ../test_coin/coin_key/$coin.node0.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin.node0 ft_balance_of '{"account_id":"'$relayer_local'"}'ls
done