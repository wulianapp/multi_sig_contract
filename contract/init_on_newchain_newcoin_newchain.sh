#!/bin/bash
##multi_sig_test:relayer
set -xv
relayer_test=b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7
relayer_local=83a666efeed6ffd0bc54c30ad1d1b904e8e49608a7298138a85ea428ce15b902

##create carelayer_testu
near --nodeUrl  http://120.232.251.101:3030 create-account multi_sig7_test.node0 --publicKey CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl  http://120.232.251.101:3030 create-account multi_sig7_local.node0 --publicKey 9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP  --masterAccount node0 --initialBalance 100 --keyPath ~/.near-credentials/local/node0.json

near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"'$relayer_test'","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"'$relayer_local'","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
near --nodeUrl  http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"multi_sig","amount":"1000000000000000000000"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000

##deploy ca
near --nodeUrl  http://120.232.251.101:3030 deploy multi_sig7_test.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_test.node0.json 
near --nodeUrl  http://120.232.251.101:3030 deploy multi_sig7_local.node0 ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./multi_sig7_local.node0.json 

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")

for coin in "${coins[@]}"; do
    near --nodeUrl http://120.232.251.101:3030 call $coin  ft_transfer '{"receiver_id":"'$relayer_local'","amount":"100000000000000000000000"}' --accountId node0 --keyPath ../test_coin/coin_key/node0.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin ft_balance_of '{"account_id":"'$relayer_local'"}'

    near --nodeUrl http://120.232.251.101:3030 call $coin  ft_transfer '{"receiver_id":"'$relayer_test'","amount":"100000000000000000000000"}' --accountId node0 --keyPath ../test_coin/coin_key/node0.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin ft_balance_of '{"account_id":"'$relayer_test'"}'
	near --nodeUrl  http://120.232.251.101:3030 call $coin set_owner '{"account_id":"multi_sig7_test.node0","is_owner":true}' --accountId multi_sig --keyPath ./multi_sig.json --gas 600000000000000
	near --nodeUrl  http://120.232.251.101:3030 call $coin set_owner '{"account_id":"multi_sig7_local.node0","is_owner":true}' --accountId multi_sig --keyPath ./multi_sig.json --gas 600000000000000
done
