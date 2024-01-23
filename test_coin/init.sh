#/!/bin/sh

near --nodeUrl http://123.56.252.201:8061 call dw20.node0 new_default_meta '{"owner_id":"dw20.node0","total_supply":"22345678900000000"}' --accountId dw20.node0 --keyPath ./dw20.node0.json
near --nodeUrl http://123.56.252.201:8061 view dw20.node0 ft_balance_of '{"account_id":"dw20.node0"}' --accountId dw20.node0 --keyPath ./dw20.node0.json
##near --nodeUrl http://123.56.252.201:8061 call dw20.node0  ft_transfer '{"receiver_id":"node0","amount":"900000000"}' --accountId dw20.node0 --keyPath ./dw20.node0.json --depositYocto 1
