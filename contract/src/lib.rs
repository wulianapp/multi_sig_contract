use ed25519_dalek::Verifier;
use near_sdk::PublicKey;
use near_sdk::{env, log, serde_json, AccountId, Promise};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::str::FromStr;

#[macro_use]
extern crate near_sdk;

#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct Conf {
    rank_max_num: u8,
    slave_max_num: u8,
}
impl Default for Conf {
    fn default() -> Self {
        Self {
            rank_max_num: 10,
            slave_max_num: 12,
        }
    }
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
pub struct MtTransfer {
    pub to: AccountId,
    //todo: 直接从env拿
    pub transfer_mt: String,
}

#[near(serializers=[borsh, json])]
pub struct PubkeySignInfo {
    pub pubkey: PublicKey,
    pub signature: String,
}

type Strategy = (Vec<u128>, Vec<u8>, Vec<PublicKey>, u128);

#[derive(Default)]
#[near(contract_state)]
pub struct Contract {
    amount_points: HashMap<AccountId, Vec<u128>>,
    slave_needs: HashMap<AccountId, Vec<u8>>,
    slaves: HashMap<AccountId, Vec<PublicKey>>,
    sub_confs: BTreeMap<AccountId, u128>,
    config: Conf,
}

/// calculate transfer_value, get number of needing slave's sig
fn get_slave_needs(
    amount_points: &Vec<u128>,
    slave_needs: &Vec<u8>,
    symbol: &str,
    amount: u128,
) -> Result<u8, String> {
    assert_eq!(amount_points.len(), slave_needs.len());

    if amount_points.is_empty() {
        return Ok(0);
    }

    if amount_points.len() == 1 {
        return Ok(slave_needs[0]);
    }

    let (base_amount, quote_amount) =
        env::mt_price(symbol).ok_or("symbol not support".to_string())?;
    let transfer_value = amount * base_amount / quote_amount;

    let mut need_num = 0;
    for index in 0..amount_points.len() {
        if transfer_value > amount_points[index] && transfer_value <= amount_points[index + 1] {
            need_num = slave_needs[index];
            break;
        }
    }
    Ok(need_num)
}

#[near]
impl Contract {
    pub fn set_config(&mut self, rank_max_num: u8, slave_max_num: u8) {
        let user_account_id = env::predecessor_account_id();
        let contract_account = env::current_account_id();
        require!(
            user_account_id == contract_account,
            "caller must be deployer"
        );
        self.config.rank_max_num = rank_max_num;
        self.config.slave_max_num = slave_max_num;
    }

    pub fn set_strategy(
        &mut self,
        slaves: Vec<PublicKey>,
        amount_points: Vec<u128>,
        slave_needs: Vec<u8>,
    ) {
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
    }

    pub fn set_rank(&mut self, amount_points: Vec<u128>, slave_needs: Vec<u8>) {
        let user_account_id = env::predecessor_account_id();

        //todo: 检查amount_points从小到大，且和salve_needs保持数量一致,salve_needs可以自由
        require!(
            amount_points.len() == slave_needs.len(),
            "amount_points size not equal salves"
        );

        self.amount_points
            .insert(user_account_id.clone(), amount_points);
        self.slave_needs
            .insert(user_account_id.clone(), slave_needs);

        log!("set {}'s rank successfully", user_account_id.to_string());
    }

    pub fn set_slaves(&mut self, slaves: Vec<PublicKey>) {
        let user_account_id = env::predecessor_account_id();
        let _salve_needs = self.slave_needs.get(&user_account_id).unwrap().to_owned();
        let _amount_points = self.amount_points.get(&user_account_id).unwrap().to_owned();

        //todo: 减少设备后和当前策略有冲突的话直接报错
        self.slaves.insert(user_account_id.clone(), slaves);
        log!("set {}'s slaves successfully", user_account_id.to_string());
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

    pub fn get_strategy(&self, user_account_id: AccountId) -> Option<Strategy> {
        let amount_points = self.amount_points.get(&user_account_id).map(|x| x.clone());
        let slave_needs = self.slave_needs.get(&user_account_id).map(|x| x.clone());
        let slaves = self.slaves.get(&user_account_id).map(|x| x.clone());
        let sub_confs = self.sub_confs.get(&user_account_id).map(|x| x.clone());
        match (amount_points, slave_needs, slaves, sub_confs) {
            (Some(points), Some(needs), Some(keys), Some(limit)) => {
                Some((points, needs, keys, limit))
            }
            _ => None,
        }
    }

    pub fn send_mt(
        &mut self,
        slave_device_sigs: Vec<PubkeySignInfo>,
        coin_tx: MtTransfer,
    ) -> Promise {
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let MtTransfer { to, transfer_mt } = coin_tx;

        let caller = env::predecessor_account_id();
        let amount = env::attached_deposit(&transfer_mt);

        let check_inputs = || -> Result<(), String> {
            let (points, slave_needs, slaves, ..) = self.get_strategy(caller.clone()).ok_or(
                format!("{} haven't register multi_sig account!", caller.to_string()),
            )?;

            let slave_need =
                get_slave_needs(&points, &slave_needs, &transfer_mt, amount.as_yoctonear())?;

            if slave_device_sigs.len() < slave_need as usize {
                Err(format!(
                    "slave device sigs is insufficient,  need {} at least",
                    slave_need
                ))?
            }

            for slave_device_sig in slave_device_sigs {
                if !slaves.contains(&slave_device_sig.pubkey) {
                    Err(format!(
                        "{:?} is not belong this multi_sig_account",
                        slave_device_sig.pubkey
                    ))?
                }

                //check slave's sig
                let public_key_bytes = slave_device_sig.pubkey.as_bytes();
                let public_key = ed25519_dalek::PublicKey::from_bytes(public_key_bytes).unwrap();
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
        Promise::new(to).transfer(transfer_mt, amount, fee_mt)
    }
}
