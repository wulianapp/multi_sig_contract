#!/bin/bash
##multi_sig_test:relayer
set -xv
##test_relayer_id=b0cd4ec0ef9382a7ca42c8a68d8d250c70c1bead7c004d8d78aa00c5a3cef7f7
##local_relayer_id=83a666efeed6ffd0bc54c30ad1d1b904e8e49608a7298138a85ea428ce15b902
test_relayer_id=test
local_relayer_id=local
test_multi_sig_ca=multi_sig7_test.node0 
local_multi_sig_ca=multi_sig7_local.node0 



near --nodeUrl http://120.232.251.101:3030  send multi_sig $local_relayer_id 1 --keyPath ./multi_sig.json
near --nodeUrl http://120.232.251.101:3030  send multi_sig  $test_relayer_id 1 --keyPath ./multi_sig.json
##1
base_amount=1000000000000000000
##100000000
mint_amount=100000000000000000000000000 
##relayer
relayer_hold=10000000000000000000000


##send coin as gas
near --nodeUrl http://120.232.251.101:3030 call btc mint_amount '{"account_id":"node0","amount":"'$mint_amount'"}'  --accountId fees_call --keyPath fees_call.json --gas 600000000000000
near --nodeUrl http://120.232.251.101:3030 call btc mint_amount '{"account_id":"'$test_relayer_id'","amount":"'$mint_amount'"}'  --accountId fees_call --keyPath fees_call.json --gas 600000000000000
near --nodeUrl http://120.232.251.101:3030 call btc mint_amount '{"account_id":"'$local_relayer_id'","amount":"'$mint_amount'"}'  --accountId fees_call --keyPath fees_call.json --gas 600000000000000

##todo: by nodejs
##serairtoken-contract/scripts/wallet.js
##near --nodeUrl http://192.168.1.152:3030  create-account test --publicKey CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY  --keyPath ./chainless.json

##create catest_relayer_idu
near --nodeUrl http://120.232.251.101:3030 create-account $test_multi_sig_ca --publicKey CuAL8qaTLg3nMQ3Jz3B2yq6SYCSygGoR2q5nEACHxVyY  --masterAccount node0 --keyPath ~/.near-credentials/local/node0.json
near --nodeUrl http://120.232.251.101:3030 create-account $local_multi_sig_ca --publicKey 9ruaNCMS1BvXfWT6MySeveTXrn2fLekbVCaWwETL18ZP  --masterAccount node0 --keyPath ~/.near-credentials/local/node0.json

##send gas and then deploy
near --nodeUrl http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"'$test_multi_sig_ca'","amount":"'$relayer_hold'"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
near --nodeUrl http://120.232.251.101:3030 call usdt   ft_transfer '{"receiver_id":"'$local_multi_sig_ca'","amount":"'$relayer_hold'"}'  --accountId node0 --keyPath ~/.near-credentials/local/node0.json --gas 600000000000000
near --nodeUrl http://120.232.251.101:3030 deploy $test_multi_sig_ca ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./$test_multi_sig_ca.json 
near --nodeUrl http://120.232.251.101:3030 deploy $local_multi_sig_ca ./target/wasm32-unknown-unknown/release/contract.wasm  --keyPath ./$local_multi_sig_ca.json 

coins=("btc" "eth" "usdt" "usdc" "cly" "dw20")
for coin in "${coins[@]}"; do
    near --nodeUrl http://120.232.251.101:3030 call $coin mint_amount '{"account_id":"'$test_relayer_id'","amount":"'$mint_amount'"}'  --accountId fees_call --keyPath fees_call.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin ft_balance_of '{"account_id":"'$local_relayer_id'"}'

    near --nodeUrl http://120.232.251.101:3030 call $coin mint_amount '{"account_id":"'$test_relayer_id'","amount":"'$mint_amount'"}'  --accountId fees_call --keyPath fees_call.json --gas 600000000000000
    near --nodeUrl http://120.232.251.101:3030 view $coin ft_balance_of '{"account_id":"'$test_relayer_id'"}'

	near --nodeUrl http://120.232.251.101:3030 call $coin set_owner '{"account_id":"'$test_multi_sig_ca'","is_owner":true}' --accountId multi_sig --keyPath ./multi_sig.json --gas 600000000000000
	near --nodeUrl http://120.232.251.101:3030 call $coin set_owner '{"account_id":"'$local_multi_sig_ca'","is_owner":true}' --accountId multi_sig --keyPath ./multi_sig.json --gas 600000000000000
done
