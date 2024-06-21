use ed25519_dalek::Verifier;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::NearToken;
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


impl Default for Contract {
        // The default trait with which to initialize the contract
        fn default() -> Self {
            Self {
                owner: AccountId::from_str("node0").unwrap(),
                user_strategy: HashMap::new(),
            }
        }
}

// Define the contract structure
//#[near_bindgen]
#[near(contract_state)]
pub struct Contract {
    owner: AccountId,
    //user_strategy: LookupMap<AccountId, StrategyData>,
    user_strategy: HashMap<AccountId, StrategyData>,
}

/// calculate transfer_value, get number of needing servant's sig
fn get_servant_need(
    strategy: &Vec<MultiSigRank>,
    coin_account_id: &str,
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

#[derive(Clone,Debug)]
#[near(serializers=[borsh, json])]
pub struct SubAccConf {
    account_id: AccountId,
    hold_value_limit: u128,
}

#[derive(Clone,Debug)]
#[near(serializers=[borsh, json])]
pub struct StrategyData {
    multi_sig_ranks: Vec<MultiSigRank>,
    //maybe  user_account_id unequal to main pub key
    servant_pubkeys: Vec<String>,
    sub_confs: BTreeMap<AccountId,SubAccConf>,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
//todo: get transfer_mt,fee_mt,amount from env
pub struct MtTransfer {
    pub from: AccountId,
    pub to: AccountId,
    pub transfer_mt: String,
    pub fee_mt: String,
    pub amount: u128,
    pub expire_at: u64,
    pub memo: Option<String>,
}

#[derive(Clone,Debug)]
#[near(serializers=[borsh, json])]
pub struct MultiSigRank {
    pub min: u128,
    pub max_eq: u128,
    pub sig_num: u8,
}

#[near(serializers=[borsh, json])]
pub struct PubkeySignInfo {
    pub pubkey: String,
    pub signature: String,
}

#[near(serializers=[borsh, json])]
pub struct AccountSignInfo {
    account_id: String,
    signature: String,
}

#[near(serializers=[borsh, json])]
impl Contract {
    pub fn get_owner() {
        unimplemented!()
    }

    #[private]
    pub fn update_owner() {
        unimplemented!()
    }

    pub fn set_strategy(
        &mut self,
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
    }

    pub fn remove_account_strategy(&mut self, acc: AccountId) {
        self.user_strategy.remove(&acc);
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
                max_eq: u128::MAX,
                sig_num: new_servant_num,
            }];
        }
        strategy.servant_pubkeys = servant_device_pubkey;
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
        servant_device_sigs: Vec<PubkeySignInfo>,
        coin_tx: MtTransfer,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let MtTransfer {
            from,
            to,
            transfer_mt,
            fee_mt,
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
                get_servant_need(&my_strategy.multi_sig_ranks, &transfer_mt, amount)
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

        //合约在白名单不扣钱,fee_mt是什么无所谓
        let fee_mt = "USDT".to_string();
        Promise::new(to).transfer(transfer_mt,NearToken::from_yoctonear(amount),fee_mt)
    }
}
