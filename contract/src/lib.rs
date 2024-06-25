use ed25519_dalek::Verifier;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{NearToken, PublicKey};
use near_sdk::{
    env, log, near_bindgen, require, serde_json, AccountId, Gas, Promise, PromiseError,
};
use std::cmp::max;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::format;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use uint::hex;

#[macro_use]
extern crate near_sdk;

/*** 
impl Default for Contract {
    // The default trait with which to initialize the contract
    fn default() -> Self {
        Self {
            amount_points: HashMap::new(),
            salve_needs: HashMap::new(),
            slaves: HashMap::new(),
            sub_confs: BTreeMap::new(),
        }
    }
}
***/
#[derive(Default)]
#[near(contract_state)]
pub struct Contract {
    amount_points: HashMap<AccountId, Vec<u128>>,
    slave_needs: HashMap<AccountId, Vec<u8>>,
    slaves: HashMap<AccountId, Vec<PublicKey>>,
    sub_confs: BTreeMap<AccountId, u128>,
}

/// calculate transfer_value, get number of needing slave's sig
fn slave_need(
    strategy: &Vec<MultiSigRank>,
    symbol: &str,
    amount: u128,
) -> Result<u8, String> {
    let (base_amount, quote_amount) =
        env::mt_price(symbol).ok_or("symbol not support".to_string())?;
    let transfer_value = amount * base_amount / quote_amount;
    let mut need_num = strategy.len() as u8;
    for rank in strategy {
        if transfer_value >= rank.min && transfer_value < rank.max_eq {
            need_num = rank.sig_num;
            break;
        }
    }
    Ok(need_num)
}

/*** 
#[derive(Clone, Debug)]
#[near(serializers=[borsh, json])]
pub struct StrategyData {
    multi_sig_ranks: Vec<MultiSigRank>,
    slave_pubkeys: Vec<String>,
}
***/

#[derive(Clone)]
#[near(serializers=[borsh, json])]
//todo: get transfer_mt,fee_mt,amount from env
pub struct MtTransfer {
    pub to: AccountId,
    pub transfer_mt: String,
    pub fee_mt: String,
    pub amount: u128,
    pub memo: Option<String>,
}


//todo: num+max改成两个数组
//salve_pubkey改为slave
#[derive(Clone, Debug)]
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


#[near]
impl Contract {

    // todo: 增加全局配置，设置可设置的梯度数量
    
    pub fn set_strategy(&mut self, slaves: Vec<PublicKey>, amount_points:Vec<u128>,slave_needs: Vec<u8>) {
        let user_account_id = env::predecessor_account_id();
        self.set_slaves(slaves);
        self.set_rank(amount_points, slave_needs);

        log!(
            "set {}'s strategy successfully",
            user_account_id.to_string()
        );
    }

    pub fn remove_account(&mut self) {
        let user_account_id = env::predecessor_account_id();
        self.amount_points.remove(&user_account_id);
        self.slave_needs.remove(&user_account_id);
        self.slaves.remove(&user_account_id);
        self.sub_confs.remove(&user_account_id);

    }

    pub fn set_rank(&mut self, amount_points:Vec<u128>,slave_needs: Vec<u8>) {
        let user_account_id = env::predecessor_account_id();

        //todo: 检查amount_points从小到大，且和salve_needs保持数量一致,salve_needs可以自由
        require!(amount_points.len() == slave_needs.len(),"amount_points size not equal salves");

        self.amount_points.insert(user_account_id.clone(), amount_points);
        self.slave_needs.insert(user_account_id.clone(), slave_needs);

        log!(
            "set {}'s rank successfully",
            user_account_id.to_string()
        );
    }

    pub fn set_slaves(&mut self, slaves: Vec<PublicKey>) {
        let user_account_id = env::predecessor_account_id();
        let _salve_needs = self.slave_needs.get(&user_account_id).unwrap().to_owned();
        let _amount_points = self.amount_points.get(&user_account_id).unwrap().to_owned();

        //todo: 减少设备后和当前策略有冲突的话直接报错   
        self.slaves.insert(user_account_id.clone(), slaves);
        log!(
            "set {}'s slaves successfully",
            user_account_id.to_string()
        );
    }

    pub fn set_subaccount_hold_limit(&mut self, hold_limit: u128) {
        let subaccount = env::predecessor_account_id();
        if let Some(value) = self.sub_confs.get_mut(&subaccount) {
            *value = hold_limit;
            log!(
                "set {}'s hold limit to {} successfully",
                subaccount.to_string(),
                hold_limit
            );
        } else {
            log!(
                "insert {}'s hold limit to {} successfully",
                subaccount,
                hold_limit
            );
            self.sub_confs.insert(subaccount, hold_limit);
        }
    }

    pub fn get_strategy(&self, user_account_id: AccountId) -> Option<(Vec<u128>,Vec<u8>,Vec<PublicKey>,u128)> {
        let amount_points =  self.amount_points.get(&user_account_id).map(|x| x.clone());
        let slave_needs = self.slave_needs.get(&user_account_id).map(|x| x.clone());
        let slaves = self.slaves.get(&user_account_id).map(|x| x.clone());
        let sub_confs = self.sub_confs.get(&user_account_id).map(|x| x.clone());
        match (amount_points,slave_needs,slaves,sub_confs) {
            (Some(points),Some(needs),Some(keys),Some(limit)) => Some((points,needs,keys,limit)),
            _ => None
        }
    }

    pub fn send_mt(
        &mut self,
        slave_device_sigs: Vec<PubkeySignInfo>,
        coin_tx: MtTransfer,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let MtTransfer {
            to,
            transfer_mt,
            fee_mt: _fee_mt,
            amount,
            memo: _memo,
        } = coin_tx;

        let caller = env::predecessor_account_id();

        let check_inputs = || -> Result<(), String> {
            let my_strategy = self.user_strategy.get(&caller).ok_or(format!(
                "{} haven't register multi_sig account!",
                caller.to_string()
            ))?;

            let slave_need =
                slave_need(&my_strategy.multi_sig_ranks, &transfer_mt, amount)?;

            if slave_device_sigs.len() < slave_need as usize {
                Err(format!(
                    "slave device sigs is insufficient,  need {} at least",
                    slave_need
                ))?
            }

            for slave_device_sig in slave_device_sigs {
                if !my_strategy
                    .slave_pubkeys
                    .contains(&slave_device_sig.pubkey)
                {
                    Err(format!(
                        "{} is not belong this multi_sig_account",
                        slave_device_sig.pubkey
                    ))?
                }

                //check slave's sig
                let public_key_bytes: Vec<u8> = hex::decode(slave_device_sig.pubkey).unwrap();
                let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                let signature =
                    ed25519_dalek::Signature::from_str(&slave_device_sig.signature).unwrap();
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
        Promise::new(to).transfer(transfer_mt, NearToken::from_yoctonear(amount), fee_mt)
    }
}
