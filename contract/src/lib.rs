mod external_coin;
mod external_bridge;
pub use crate::external_coin::*;
pub use crate::external_bridge::*;


use ed25519_dalek::Verifier;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json, AccountId, Gas, Promise, PromiseError,
};
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::format;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use uint::hex;
use std::collections::BTreeMap;

#[macro_use]
extern crate near_sdk;



type Index = u64;
const COIN_CONTRACT_IDS:[&'static str; 6] = ["btc.node0","eth.node0","usdt.node0","usdc.node0","dw20.node0","cly.node0"];

#[derive(PartialEq,Debug)]
pub enum TxType{
    //主账户给主账户转，要手续费，用户签
    Normal,
    Main2Sub,
    Sub2Main,
    Main2Bridge,
}

// Define the contract structure
//#[near_bindgen]
#[near(contract_state)]
pub struct Contract {
    owner: AccountId,
    //user_strategy: LookupMap<AccountId, StrategyData>,
    //success_tx: LookupSet<Index>,
    user_strategy: HashMap<AccountId, StrategyData>,
    success_tx: HashSet<Index>,
}

//delete it
impl Default for Contract {
    // The default trait with which to initialize the contract
    fn default() -> Self {
        Self {
            owner: AccountId::from_str("node0").unwrap(),
            //user_strategy: LookupMap::new(b"m"),
            //success_tx: LookupSet::new(b"m"),
            user_strategy: HashMap::new(),
            success_tx: HashSet::new(),
        }
    }
}

/// calculate transfer_value, get number of needing servant's sig
fn get_servant_need(
    strategy: &Vec<MultiSigRank>,
    coin_account_id: &AccountId,
    amount: u128,
) -> Option<u8> {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let coin_price = 1;
    let transfer_value = amount * coin_price;
    strategy
        .iter()
        .find(|&rank| transfer_value >= rank.min && transfer_value < rank.max_eq)
        .map(|rank| rank.sig_num)
}

//获取持仓价值
/***
fn get_account_hold_value(
    account_id: &AccountId,
) -> u128 {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    COIN_CONTRACT_IDS.iter().map(|&addr|{
        let coin_account = AccountId::from_str(addr).unwrap();
        coin::ext(coin_account)
        .with_static_gas(Gas(5 * TGAS))
        .chainless_transfer_from(sender_id.to_owned(), receiver_id.to_owned(), amount, memo)
        .then(
            // Create a callback change_greeting_callback
            Self::ext(env::current_account_id())
                //todo: how many gas?
                .with_static_gas(Gas(5 * TGAS))
                .call_chainless_transfer_from_callback(),
        )
    })
    .sum::<u128>()
}
**/
#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct SubAccConf {
    pubkey:String,
    hold_value_limit: u128,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct StrategyData {
    //考虑到链上master变更之后,主账户转给子账户，主账户的签名需要验证是否是对应的master_key签的
    master_pubkey: String,
    multi_sig_ranks: Vec<MultiSigRank>,
    //maybe  user_account_id unequal to main pub key
    servant_pubkeys: Vec<String>,
    sub_confs: BTreeMap<AccountId,SubAccConf>,
}

#[near(serializers=[borsh, json])]
pub struct CoinTx {
    from: AccountId,
    to: AccountId,
    coin_id: AccountId,
    amount: u128,
    expire_at: u64,
    memo: Option<String>,
}

#[near(serializers=[borsh, json])]
pub struct SubAccCoinTx {
    coin_id: AccountId,
    amount: u128,
}


#[near(serializers=[borsh, json])]
pub struct WithdrawInfo {
    from: AccountId,
    kind: String,
    coin_id: AccountId,
    amount: u128,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct MultiSigRank {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

#[near(serializers=[borsh, json])]
pub struct PubkeySignInfo {
    pubkey: String,
    signature: String,
}

#[near(serializers=[borsh, json])]
pub struct AccountSignInfo {
    account_id: String,
    signature: String,
}

#[near(serializers=[borsh, json])]
impl Contract {
    pub fn get_txs_state(&self, txs_index: Vec<Index>) -> Vec<(Index, bool)> {
        let values: Vec<bool> = txs_index
            .iter()
            .map(|index| self.success_tx.contains(index))
            .collect();
        txs_index.into_iter().zip(values.into_iter()).collect()
    }

    fn call_chainless_transfer_from(
        &mut self,
        tx_index: Index,
        sender_id: &AccountId,
        coin_id: &AccountId,
        receiver_id: &AccountId,
        amount: U128,
        memo: Option<String>,
        tx_type: TxType
    ) -> Promise {
        log!(
            "start transfer {}(coin_id) {}(sender_id) {}(receiver_id) {}(amount)",
            coin_id.to_string(),
            sender_id.to_string(),
            receiver_id.to_string(),
            amount.0
        );
        //todo: move to callback
        //self.success_tx.insert(&tx_index);
        self.success_tx.insert(tx_index);
        log!("index {} ft_transfer was successful2!", tx_index);
        //子账户给主账户转账要求免费,提现要有手续费
        let transfer_promise = if tx_type == TxType::Sub2Main || tx_type == TxType::Main2Sub {
            coin::ext(coin_id.to_owned())
            .with_static_gas(Gas::from_tgas(5))
            .transfer_from_nongas(sender_id.to_owned(), receiver_id.to_owned(), amount, memo)
        }else{
            coin::ext(coin_id.to_owned())
            .with_static_gas(Gas::from_tgas(5))
            .transfer_from(sender_id.to_owned(), receiver_id.to_owned(), amount, memo)
        };

        transfer_promise.then(
                {
                    let call_handle = Self::ext(env::current_account_id()).with_static_gas(Gas::from_tgas(5));
                    if tx_type == TxType::Main2Bridge{
                        call_handle.call_transfer_to_bridge_callback(1500,sender_id.to_owned(),amount,coin_id.to_owned())
                    }else{
                        call_handle.call_chainless_transfer_from_callback()
                    }
                
                } 
        )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn call_chainless_transfer_from_callback(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        // Return whether or not the promise succeeded using the method outlined in external_coin
        //fixme: get return info
        if call_result.is_err() {
            env::log_str("ft_transfer failed...");
            return false;
        } else {
            //todo: get result
            //  let greeting: String = call_result.unwrap();

            env::log_str("ft_transfer was successful2!");
            //todo: get tx_index from memo
            //self.success_tx.insert(tx_index);
            return true;
        }
    }


    /*** 
    fn call_transfer_bridge(
        &mut self,
        tx_index: Index,
        sender_id: &AccountId,
        coin_id: &AccountId,
        bridge_addr: &AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> Promise {
        log!(
            "start12 call_transfer_bridge {}(coin_id) {}(sender_id) {}(receiver_id) {}(amount)",
            coin_id.to_string(),
            sender_id.to_string(),
            bridge_addr.to_string(),
            amount.0
        );
        //todo: move to callback
        //self.success_tx.insert(&tx_index);
        self.success_tx.insert(tx_index);
        log!("line: {}. index {} ft_transfer was successful32!",line!(), tx_index);
        coin::ext(coin_id.to_owned())
            .with_static_gas(Gas(5 * TGAS))
            .transfer_from(sender_id.to_owned(), bridge_addr.to_owned(), amount, memo)
            .then(
                {
                    log!("line {} ft_transfer was successful3!", line!());
                    Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .call_transfer_to_bridge_callback(1500,sender_id.to_owned(),amount,coin_id.to_owned())
                }
                
            )
    }
    ***/

    #[private] // Public - but only callable by env::current_account_id()
    pub fn call_transfer_to_bridge_callback(
        &mut self,
        chain_id: u128,
        account_id: AccountId,
        amount: U128,
        token: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Promise {
        // Return whether or not the promise succeeded using the method outlined in external_coin
        //fixme: get return info
        if call_result.is_err() {
            log!("line {} transfer_to_bridge_ failed...!", line!());
            env::log_str("transfer_to_bridge_ failed...");
            env::panic_str("transfer_to_bridge_ failed...");
        } else {
            log!("line {} transfer_to_bridge_ failed...!", line!());
            env::log_str("transfer_to_bridge was successful!");
            let bridge_addr = AccountId::from_str(external_bridge::BRIDGE_ADDRESS).unwrap();
            bridge::ext(bridge_addr)
            .with_static_gas(Gas::from_tgas(5))
            .new_order(chain_id,account_id,amount.0,token)
            .then(
                Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(5))
                .call_new_withdraw_order_callback(),
            )
        }
    }

    #[private] 
    pub fn call_new_withdraw_order_callback(
        &mut self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> bool {
        if call_result.is_err() {
            env::log_str("call_new_withdraw_order  failed...");
            return false;
        } else {
            env::log_str("call_new_withdraw_order  was successful!");
            return true;
        }
    }

 

    /***
         #[init]
    #[private]
    pub fn new() -> Self{
        let caller = env::signer_account_id();
        Contract{
            owner: caller,
            user_strategy: LookupMap::new(b"m"),
            success_tx: LookupSet::new(b"m"),
            success_tx2: LookupSet::new(b"m"),
            TestBool:false,
        }
    }
     */

    pub fn get_owner() {
        unimplemented!()
    }

    #[private]
    pub fn update_owner() {
        unimplemented!()
    }

    pub fn set_strategy2(
        &mut self,
        master_pubkey: String,
        user_account_id: AccountId,
        servant_pubkeys: Vec<String>,
        sub_confs: BTreeMap<AccountId,SubAccConf>,
        rank_arr: Vec<MultiSigRank>,
        
    ) {
        //todo: span must be serial
        //todo: must be called by owner
        //let multi_sig_ranks = rank_arr.iter().map(|&x| x.into()).collect();
        let multi_sig_ranks = rank_arr;
        let strategy = StrategyData {
            master_pubkey,
            multi_sig_ranks,
            sub_confs,
            servant_pubkeys,
        };
        self.user_strategy.insert(user_account_id.clone(), strategy);
        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }

    pub fn add_subaccounts(&mut self, main_account_id: AccountId, new_sub: HashMap<AccountId,SubAccConf>) {
        //todo: call must be relayer
        let my_strategy = self.user_strategy.get(&main_account_id);
        require!(my_strategy.is_some(), "main_account_id isn't exsit");
        let mut my_strategy = my_strategy.unwrap().to_owned();
        my_strategy.sub_confs.extend(new_sub);
        self.user_strategy.insert(main_account_id, my_strategy).unwrap();
    }

    //仅仅是合约解除绑定，但是链底层不删，上层检查余额是否为零
    pub fn remove_subaccounts(&mut self, main_account_id: AccountId, accounts: Vec<AccountId>) {
        //todo: call must be relayer
        let my_strategy = self.user_strategy.get(&main_account_id);
        require!(my_strategy.is_some(), "main_account_id isn't exsit");
        let mut my_strategy = my_strategy.unwrap().to_owned();
        
        my_strategy.sub_confs = my_strategy
            .sub_confs
            .into_iter()
            .filter(|item| !accounts.contains(&item.0))
            .collect();
        
        //self.user_strategy.insert(&main_account_id, &my_strategy);
        self.user_strategy
            .insert(main_account_id, my_strategy)
            .unwrap();
    }

    pub fn clear_all(&mut self) {
        self.user_strategy.clear();
        self.success_tx.clear();
    }

    pub fn remove_account_strategy(&mut self, acc: AccountId) {
        self.user_strategy.remove(&acc);
    }
    pub fn remove_tx_index(&mut self, index: Index) {
        self.success_tx.remove(&index);
    }

    //必须是设置安全问答之后才能变更策略
    //cover origin
    pub fn update_rank(&mut self, user_account_id: AccountId, rank_arr: Vec<MultiSigRank>) {
        let mut strategy = self.user_strategy.get(&user_account_id).unwrap().to_owned();
        //todo: 更多的校验
        if rank_arr.len() > strategy.servant_pubkeys.len() + 1{
            require!(false, "rank size must be equal to servant size");
        }
        strategy.multi_sig_ranks = rank_arr;
        //self.user_strategy.insert(&user_account_id, &strategy);
        self.user_strategy
            .insert(user_account_id.clone(), strategy)
            .unwrap();
        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }

    //cover origin
    pub fn update_servant_pubkey(
        &mut self,
        user_account_id: AccountId,
        servant_device_pubkey: Vec<String>,
    ) {
        let mut strategy = self.user_strategy.get(&user_account_id).unwrap().to_owned();
        let new_servant_num = servant_device_pubkey.len() as u8;
        if strategy.servant_pubkeys.len() as u8 != new_servant_num {
            strategy.multi_sig_ranks = vec![MultiSigRank {
                min: 0u128,
                max_eq: 1_000_000_000_000_000_000_000_000u128,//one million
                sig_num: new_servant_num,
            }];
        }
        strategy.servant_pubkeys = servant_device_pubkey;
        //self.user_strategy.insert(&user_account_id, &strategy);
        self.user_strategy
            .insert(user_account_id.clone(), strategy)
            .unwrap();
        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }


    pub fn update_servant_pubkey_and_master(
        &mut self,
        user_account_id: AccountId,
        servant_device_pubkey: Vec<String>,
        master_pubkey: String
    ) {
        let mut strategy = self.user_strategy.get(&user_account_id).unwrap().to_owned();
        let new_servant_num = servant_device_pubkey.len() as u8;
        if strategy.servant_pubkeys.len() as u8 != new_servant_num {
            strategy.multi_sig_ranks = vec![MultiSigRank {
                min: 0u128,
                max_eq: u128::MAX,
                sig_num: new_servant_num,
            }];
        }
        strategy.master_pubkey = master_pubkey;

        strategy.servant_pubkeys = servant_device_pubkey;
        //self.user_strategy.insert(&user_account_id, &strategy);
        self.user_strategy
            .insert(user_account_id.clone(), strategy.to_owned())
            .unwrap();
        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }


    pub fn update_master(
        &mut self,
        user_account_id: AccountId,
        master_pubkey: String
    ) {
        let mut strategy = self.user_strategy.get(&user_account_id).unwrap().to_owned();
        strategy.master_pubkey = master_pubkey;

        //self.user_strategy.insert(&user_account_id, &strategy);
        self.user_strategy
            .insert(user_account_id.clone(), strategy.to_owned())
            .unwrap();
        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }

    pub fn update_subaccount_hold_limit(
        &mut self,
        user_account_id: AccountId,
        subaccount: AccountId,
        hold_limit: u128
    ) {

        if let Some(strategy) = self.user_strategy.get_mut(&user_account_id){
            if let Some(sub_conf) = strategy.sub_confs.get_mut(&subaccount) {
                sub_conf.hold_value_limit = hold_limit;
            } else {
                require!(false, "Not found subaccount");
            }
        }else{
            require!(false, "Not found mainaccount");
        }

        log!(
            "set {}'s hold limit to {} successfully",
            subaccount.to_string(),hold_limit
        );
    }
    

    pub fn get_strategy(&self, user_account_id: AccountId) -> Option<StrategyData> {
        //self.user_strategy.get(&user_account_id).as_ref().map(|data| data.to_owned())
        self.user_strategy
            .get(&user_account_id).map(|x| x.clone())
    }
    pub fn send_money(
        &mut self,
        tx_index: Index,
        servant_device_sigs: Vec<PubkeySignInfo>,
        coin_tx: CoinTx,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let CoinTx {
            from,
            to,
            coin_id,
            amount,
            memo,
            expire_at,
        } = coin_tx;
        let caller = env::predecessor_account_id();

        require!(caller.eq(&from), "from must be  equal caller");

        let check_inputs = || -> Result<(), String> {
            let my_strategy = self.user_strategy.get(&caller).ok_or(format!(
                "{} haven't register multi_sig account!",
                caller.to_string()
            ))?;

            let now = env::block_timestamp_ms();
            if now > expire_at {
                Err(format!(
                    "signature have been expired: now {} and expire_at {}",
                    now, expire_at
                ))?
            }

            let servant_need =
                get_servant_need(&my_strategy.multi_sig_ranks, &coin_id, amount)
                .unwrap_or(my_strategy.servant_pubkeys.len() as u8);

            if servant_device_sigs.len() < servant_need as usize {
                Err(format!(
                    "servant device sigs is insufficient,  need {} at least",
                    servant_need
                ))?
            }

            for servant_device_sig in servant_device_sigs {
                if !my_strategy
                    .servant_pubkeys
                    .contains(&servant_device_sig.pubkey)
                {
                    Err(format!(
                        "{} is not belong this multi_sig_account",
                        servant_device_sig.pubkey
                    ))?
                }

                //check servant's sig
                let public_key_bytes: Vec<u8> = hex::decode(servant_device_sig.pubkey).unwrap();
                let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                let signature =
                    ed25519_dalek::Signature::from_str(&servant_device_sig.signature).unwrap();
                if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                    Err(format!("signature check failed:{}", error.to_string()))?
                }
            }
            Ok(())
        };
        //as far as possible to chose require rather than  panic_str
        if let Err(error) = check_inputs() {
            require!(false, error)
        }
        let tx_type = if to.to_string() == BRIDGE_ADDRESS.to_string() {
            TxType::Main2Bridge
        }else{
            TxType::Normal
        };
        self.call_chainless_transfer_from(tx_index, &caller, &coin_id, &to, amount.into(), memo,tx_type)
    }


    //官方账号交互、免所有手续费、需要多签名
    pub fn internal_transfer_main_to_sub(
        &mut self,
        master_sig: PubkeySignInfo,
        servant_sigs: Vec<PubkeySignInfo>,
        coin_tx: CoinTx,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let CoinTx {
            from,
            to,
            coin_id,
            amount,
            expire_at,
            memo,
        } = coin_tx;
        let caller = env::signer_account_id();
        //todo: from == master_sig.account_id
        let main_account_id = from.clone();
        //require!(caller.eq(&from),"from must be  equal caller");

        let check_inputs = || -> Result<(), String> {
            let my_strategy = self.user_strategy.get(&main_account_id).ok_or(format!(
                "{} haven't register account!",
                main_account_id.to_string()
            ))?;
            
            //主账户的master_key和签名的master进行对比
            if master_sig.pubkey != my_strategy.master_pubkey {
                Err(format!(
                    "account's master pubkey is {},but input master key is {}",
                    my_strategy.master_pubkey, master_sig.pubkey
                ))?
            }


            //主账户给子账户转需要验证过期时间和主账户签名和从设备签名
            //子账户给主账户签名只验证子账户签名，因为子账户的从设备数量为零
            let subaccount_ids: Vec<AccountId> = my_strategy.clone().sub_confs.into_keys().collect();
            if subaccount_ids.contains(&to) {
                log!(
                    "internal transfer from main_account({}) to subaccount({})",
                    from.to_string(),
                    to.to_string()
                );
                let now = env::block_timestamp_ms();
                if now > expire_at {
                    Err(format!(
                        "signature have been expired: now {} and expire_at {}",
                        now, expire_at
                    ))?
                }

                let servant_need =
                    get_servant_need(&my_strategy.multi_sig_ranks, &coin_id, amount).unwrap();
                if servant_sigs.len() < servant_need as usize {
                    Err(format!(
                        "servant device sigs is insufficient,  need {} at least",
                        servant_need
                    ))?
                }

                //check master sig
                let public_key_bytes: Vec<u8> = hex::decode(master_sig.pubkey).unwrap();
                let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                let signature = ed25519_dalek::Signature::from_str(&master_sig.signature).unwrap();
                if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                    Err(format!(
                        "master signature check failed:{}",
                        error.to_string()
                    ))?
                }

                for servant_device_sig in servant_sigs {
                    if !my_strategy
                        .servant_pubkeys
                        .contains(&servant_device_sig.pubkey)
                    {
                        Err(format!(
                            "{} is not belong this multi_sig_account",
                            servant_device_sig.pubkey
                        ))?
                    }

                    //check servant's sig
                    let public_key_bytes: Vec<u8> = hex::decode(servant_device_sig.pubkey).unwrap();
                    let public_key =
                        ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                    let signature =
                        ed25519_dalek::Signature::from_str(&servant_device_sig.signature).unwrap();
                    if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                        Err(format!(
                            "servant signature check failed:{}",
                            error.to_string()
                        ))?
                    }
                }
            } else {
                Err("input is illegal")?
            }
            
            Ok(())
        };
        //as far as possible to chose require rather than  panic_str
        if let Err(error) = check_inputs() {
            require!(false, error)
        }
        //todo: call_chainless_transfer_from_no_fee
        self.call_chainless_transfer_from(0u64, &from, &coin_id, &to, amount.into(), memo,TxType::Main2Sub)
    }

    //官方账号交互、免所有手续费
    pub fn internal_transfer_sub_to_main(
        &mut self,
        main_account_id: AccountId,
        sub_sig: AccountSignInfo,
        coin_tx: SubAccCoinTx,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let SubAccCoinTx { coin_id, amount } = coin_tx;
        let caller = env::signer_account_id();
        let sub_account = AccountId::from_str(&sub_sig.account_id).unwrap();
        //require!(caller.eq(&from),"from must be  equal caller");

        let check_inputs = || -> Result<(), String> {
            let my_strategy = self.user_strategy.get(&main_account_id).ok_or(format!(
                "{} haven't register account!",
                main_account_id.to_string()
            ))?;

            //main_account就是to，sub就是from
            //if subaccounts.contains(&sub_account) {
            if let Some(conf) = my_strategy.sub_confs.get(&sub_account){
                log!(
                    "internal transfer from main_account({}) to subaccount({})",
                    main_account_id.to_string(),
                    sub_account.to_string()
                );

                //check master sig
                //let pubkey_str = 
                let public_key_bytes: Vec<u8> = hex::decode(&conf.pubkey).unwrap();
                let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                let signature = ed25519_dalek::Signature::from_str(&sub_sig.signature).unwrap();
                if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                    Err(format!(
                        "subaccount signature check failed:{}",
                        error.to_string()
                    ))?
                }
            } else {
                Err("input is illegal")?
            };
            Ok(())
        };
        //as far as possible to chose require rather than  panic_str
        if let Err(error) = check_inputs() {
            require!(false, error)
        }
        //todo: call_chainless_transfer_from_no_fee
        self.call_chainless_transfer_from(
            0u64,
            &sub_account,
            &coin_id,
            &main_account_id,    
            amount.into(),
            None,
            TxType::Sub2Main
        )
    }



     //从跨链桥提币,admin签名免手续费，弃用
    fn internal_withdraw(
        &mut self,
        master_sig: PubkeySignInfo,
        servant_sigs: Vec<PubkeySignInfo>,
        coin_tx: CoinTx,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let CoinTx {
            from,
            to: bridge_addr,
            coin_id,
            amount,
            expire_at,
            memo,
        } = coin_tx;
        let caller = env::signer_account_id();
        let main_account_id = from.clone();
        //require!(caller.eq(&from),"from must be  equal caller");

        let check_inputs = || -> Result<(), String> {
            let my_strategy = self.user_strategy.get(&main_account_id).ok_or(format!(
                "{} haven't register account!",
                main_account_id.to_string()
            ))?;
            
            //todo: check bridge_address

            //主账户的master_key和签名的master进行对比
            if master_sig.pubkey != my_strategy.master_pubkey {
                Err(format!(
                    "account's master pubkey is {},but input master key is {}",
                    my_strategy.master_pubkey, master_sig.pubkey
                ))?
            }


            //todo: setup bridge_address at initial
            if bridge_addr.to_string().contains("chainless") {
                log!(
                    "internal transfer from main_account({}) to subaccount({})",
                    from.to_string(),
                    bridge_addr.to_string()
                );
                let now = env::block_timestamp_ms();
                if now > expire_at {
                    Err(format!(
                        "signature have been expired: now {} and expire_at {}",
                        now, expire_at
                    ))?
                }

                let servant_need =
                    get_servant_need(&my_strategy.multi_sig_ranks, &coin_id, amount).unwrap();
                if servant_sigs.len() < servant_need as usize {
                    Err(format!(
                        "servant device sigs is insufficient,  need {} at least",
                        servant_need
                    ))?
                }

                //check master sig
                let public_key_bytes: Vec<u8> = hex::decode(master_sig.pubkey).unwrap();
                let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                let signature = ed25519_dalek::Signature::from_str(&master_sig.signature).unwrap();
                if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                    Err(format!(
                        "master signature check failed:{}",
                        error.to_string()
                    ))?
                }

                for servant_device_sig in servant_sigs {
                    if !my_strategy
                        .servant_pubkeys
                        .contains(&servant_device_sig.pubkey)
                    {
                        Err(format!(
                            "{} is not belong this multi_sig_account",
                            servant_device_sig.pubkey
                        ))?
                    }

                    //check servant's sig
                    let public_key_bytes: Vec<u8> = hex::decode(servant_device_sig.pubkey).unwrap();
                    let public_key =
                        ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                    let signature =
                        ed25519_dalek::Signature::from_str(&servant_device_sig.signature).unwrap();
                    if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                        Err(format!(
                            "servant signature check failed:{}",
                            error.to_string()
                        ))?
                    }
                }
            } else {
                Err("input is illegal")?
            }
            
            Ok(())
        };
        //as far as possible to chose require rather than  panic_str
        if let Err(error) = check_inputs() {
            require!(false, error)
        }
        //todo: call_chainless_transfer_from_no_fee
        //self.call_chainless_transfer_from(0u64, &from, &coin_id, &to, amount.into(), memo)

        self.call_chainless_transfer_from(
            0u64,
            &from,
            &coin_id,
            &bridge_addr,    
            amount.into(),
            None,
            TxType::Main2Bridge
        )
    }
}

/****
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_strategy() {
        let user_account_id = AccountId::from_str("test1.node0").unwrap();
        let user_account_id =
            AccountId::from_str("c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0dd6")
                .unwrap();
        let main_device_pubkey =
            "c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0dd6".to_string();
        let servant_device_pubkey = vec![
            "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000003".to_string(),
        ];
        let rank1 = MultiSigRank {
            min: 0,
            max_eq: 100,
            sig_num: 1,
        };

        let rank2 = MultiSigRank {
            min: 100,
            max_eq: 10000,
            sig_num: 2,
        };
        let rank3 = MultiSigRank {
            min: 10000,
            max_eq: 999999999999,
            sig_num: 3,
        };

        let rank_arr = vec![rank1, rank2, rank3];
        let mut contract = Contract::new();
        contract.set_strategy(
            user_account_id,
            main_device_pubkey,
            servant_device_pubkey,
            rank_arr,
        );
    }

    #[test]
    fn test_send_money() {
        let receiver_id =
            AccountId::from_str("c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0d06")
                .unwrap();
        let coin_account_id = AccountId::from_str("dw20.node0").unwrap();
        let amount = 200u128;
        let sign_info1 = PubkeySignInfo {
            account_id: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            signature:
                "11bfe4d0b7705f6c57282a9030b22505ce2641547e9f246561d75a284f5a6e0a10e596fa7e702b6f89\
             7ad19c859ee880d4d1e80e521d91c53cc8827b67550001"
                    .to_string(),
        };
        let sign_info2 = PubkeySignInfo {
            account_id: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            signature:
                "11bfe4d0b7705f6c57282a9030b22505ce2641547e9f246561d75a284f5a6e0a10e596fa7e70\
             2b6f897ad19c859ee880d4d1e80e521d91c53cc8827b67550002"
                    .to_string(),
        };
        let servant_device_sigs = vec![sign_info1, sign_info2];
        let mut contract = Contract::new();
        contract.send_money(servant_device_sigs, receiver_id, coin_account_id, amount);
    }
}
***/
