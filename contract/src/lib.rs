#![feature(exclusive_range_pattern)]

mod external_coin;
pub use crate::external_coin::*;


use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::format;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, env, AccountId, require, Gas, Promise, PromiseError, serde_json};
use near_sdk::json_types::U128;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::{LookupMap,LookupSet};
use ed25519_dalek::Verifier;
use uint::hex;

type Index = u64;

//考虑到跨合约调用无法原子操作，以及合约本身一些条件不满足的报错，且链上交易监控工作量大
//因此在合约记录成功交易的，给应用层进行判断

/***
impl fmt::Display for TxStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::Created => "Created",
            Self::Failed => "Failed",
            Self::Successful => "Successful",
        };
        write!(f, "{}", description)
    }
}

 */

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner: AccountId,
    user_strategy: LookupMap<AccountId, StrategyData>,
    success_tx: LookupSet<Index>,
    success_tx2: LookupSet<Index>,
    TestBool:bool,
}

//delete it
impl Default for Contract {
    // The default trait with which to initialize the contract
    fn default() -> Self {
        Self {
            owner: AccountId::from_str("node0").unwrap(),
            user_strategy: LookupMap::new(b"m"),
            success_tx: LookupSet::new(b"m"),
            success_tx2: LookupSet::new(b"m"),
            TestBool: false,
        }
    }
}

/// calculate transfer_value, get number of needing servant's sig
fn get_servant_need(strategy: &Vec<MultiSigRank>, coin_account_id: &AccountId, amount: u128) -> Option<u8> {
    //todo: get price by oracle
    //let coin_price = get_coin_price(coin_account_id);
    let coin_price = 1;
    let transfer_value = amount * coin_price;
    strategy
        .iter()
        .find(|&rank|
            transfer_value >= rank.min && transfer_value < rank.max_eq
        )
        .map(|rank| rank.sig_num)
}


#[derive(Serialize, Deserialize,BorshDeserialize, BorshSerialize,Clone)]
pub struct StrategyData {
    multi_sig_ranks: Vec<MultiSigRank>,
    //maybe  user_account_id unequal to main pub key
    main_device_pubkey: String,
    servant_device_pubkey: Vec<String>,
}


#[derive(Serialize, Deserialize,BorshDeserialize, BorshSerialize,Clone)]
pub struct CoinTx {
    from: AccountId,
    to: AccountId,
    coin_id:AccountId,
    amount:u128,
    expire_at: u64,
    memo:Option<String>
}

//min,max,sig_num
//pub type MultiSigRank = (u128, u128, u8);

#[derive(Serialize, Deserialize,BorshDeserialize, BorshSerialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MultiSigRank {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

#[derive(Serialize, Deserialize,BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SignInfo {
    pubkey: String,
    signature: String,
}

#[near_bindgen]
impl Contract {

    pub fn get_user_strategy(&self,account_id:&AccountId) -> StrategyData {
        match self.user_strategy.get(account_id) {
            Some(data) => data.to_owned(),
            None => {
                env::panic_str(format!("{} haven't register multi_sig account", &account_id).as_str());
            }
        }
    }

    pub fn get_txs_state(&self, txs_index:Vec<Index>) -> Vec<(Index, bool)> {
        let values:Vec<bool> = txs_index.iter().map(|index| self.success_tx.contains(index)).collect();
        txs_index.into_iter().zip(values.into_iter()).collect()
    }

    fn call_chainless_transfer_from(&mut self,tx_index:Index,sender_id:&AccountId,coin_id:&AccountId,receiver_id:&AccountId,amount: U128,memo:Option<String>) -> Promise{
        log!("start transfer {}(coin_id) {}(sender_id) {}(receiver_id) {}(amount)",
                     coin_id.to_string(),
                     sender_id.to_string(),
                     receiver_id.to_string(),
                     amount.0
        );
        //todo: move to callback
        log!("index {} ft_transfer was successful2!",tx_index);
        self.success_tx.insert(&tx_index);
        coin::ext(coin_id.to_owned())
            .with_static_gas(Gas(5*TGAS))
            .chainless_transfer_from(sender_id.to_owned(),receiver_id.to_owned(),amount,memo)
            .then( // Create a callback change_greeting_callback
                   Self::ext(env::current_account_id())
                       //todo: how many gas?
                       .with_static_gas(Gas(5*TGAS))
                       .call_chainless_transfer_from_callback()
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn call_chainless_transfer_from_callback(&mut self, #[callback_result] call_result: Result<(), PromiseError>) -> bool {
        // Return whether or not the promise succeeded using the method outlined in external_coin
        //fixme: get return info
        if call_result.is_err() {
            env::log_str("ft_transfer failed...");
            return false;
        } else {
            env::log_str("ft_transfer was successful2!");
            //todo: get tx_index from memo
            //self.success_tx.insert(tx_index);
            return true;
        }
    }

    #[init]
    #[private]
    pub fn new() -> Self{
        let caller = env::signer_account_id();
        Contract{
            owner: caller,
            user_strategy: LookupMap::new(b"m"),
            success_tx: LookupSet::new(b"m"),
            success_tx2: LookupSet::new(b"m"),
            TestBool: true
        }
    }

    pub fn get_owner(){
        unimplemented!()
    }

    #[private]
    pub fn update_owner(){
        unimplemented!()
    }

    //#[private]
    pub fn set_strategy(&mut self,
                        user_account_id: AccountId,
                        main_device_pubkey: String,
                        servant_device_pubkey: Vec<String>,
                        rank_arr: Vec<MultiSigRank>) {

        //todo: span must be serial
        //todo: must be called by owner
        //let multi_sig_ranks = rank_arr.iter().map(|&x| x.into()).collect();
        let multi_sig_ranks = rank_arr;
        let strategy = StrategyData {
            multi_sig_ranks,
            main_device_pubkey,
            servant_device_pubkey,
        };
        self.user_strategy.insert(&user_account_id, &strategy);
        log!("set {}'s strategy successfully",user_account_id.to_string());
    }

    /***
    //todo: cann't iter
    pub fn clear_all(&mut self){
        self.user_strategy.clear();
        self.success_tx.clear();
    }
     */
    pub fn remove_account_strategy(&mut self,acc: AccountId){
        self.user_strategy.remove(&acc);
    }
    pub fn remove_tx_index(&mut self,index:Index){
        self.success_tx.remove(&index);
    }

    #[private]
    pub fn update_rank(){
        unimplemented!()
    }

    #[private]
    pub fn update_main_pubkey(){
        unimplemented!()
    }

    #[private]
    pub fn update_servant_pubkey(){
        unimplemented!()
    }

    pub fn get_strategy(&self,user_account_id: AccountId) -> Option<StrategyData>{
        self.user_strategy.get(&user_account_id).as_ref().map(|data| data.to_owned())
    }

    pub fn send_money(&mut self,
                      tx_index: Index,
                      servant_device_sigs: Vec<SignInfo>,
                      coin_tx: CoinTx,
    ) -> Promise{
        let coin_tx_str = serde_json::to_string(&coin_tx).unwrap();
        let CoinTx{ from,to,coin_id,amount,memo,expire_at} = coin_tx;
        let caller = env::predecessor_account_id();

        require!(caller.eq(&from),"from must be  equal caller");

        let check_inputs = || -> Result<(), String>{
            let my_strategy = self.user_strategy.get(&caller).ok_or(
                format!("{} haven't register multi_sig account!",caller.to_string())
            )?;

            let now = env::block_timestamp_ms();
            if now > expire_at {
                Err(format!("signature have been expired: now {} and expire_at {}",now,expire_at))?
            }

            let servant_need = get_servant_need(&my_strategy.multi_sig_ranks, &coin_id, amount).unwrap();
            if servant_device_sigs.len() < servant_need as usize {
                Err(format!("servant device sigs is insufficient,  need {} at least",servant_need))?
            }

             for servant_device_sig in servant_device_sigs {
                 if !my_strategy.servant_device_pubkey.contains(&servant_device_sig.pubkey){
                     Err(format!("{} is not belong this multi_sig_account",servant_device_sig.pubkey))?
                 }

                 //check servant's sig
                 let public_key_bytes :Vec<u8> = hex::decode(servant_device_sig.pubkey).unwrap();
                 let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes).unwrap();
                 let signature = ed25519_dalek::Signature::from_str(&servant_device_sig.signature).unwrap();
                 if let Err(error) = public_key.verify(coin_tx_str.as_bytes(), &signature) {
                     Err(format!("signature check failed:{}",error.to_string()))?
                 }
             }
            Ok(())
        };
        //as far as possible to chose require rather than  panic_str
        if let Err(error) = check_inputs() {
            require!(false,error)
        }
        self.call_chainless_transfer_from(tx_index,&caller,&coin_id,&to,amount.into(),memo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_strategy() {
        let user_account_id = AccountId::from_str("test1.node0").unwrap();
        let user_account_id = AccountId::from_str("c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0dd6").unwrap();
        let main_device_pubkey = "c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0dd6".to_string();
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

        let rank_arr = vec![rank1,rank2,rank3];
        let mut contract = Contract::new();
        contract.set_strategy(user_account_id,main_device_pubkey,servant_device_pubkey,rank_arr);
    }

    #[test]
    fn test_send_money() {
        let receiver_id = AccountId::from_str(
            "c25c7068ba9a5b74e1fbf051049359b7e98305b5415eed8d697087e1304f0d06"
        ).unwrap();
        let coin_account_id = AccountId::from_str("dw20.node0").unwrap();
        let amount = 200u128;
        let sign_info1 = SignInfo{
            pubkey: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            signature: "11bfe4d0b7705f6c57282a9030b22505ce2641547e9f246561d75a284f5a6e0a10e596fa7e702b6f89\
             7ad19c859ee880d4d1e80e521d91c53cc8827b67550001".to_string(),
        };
        let sign_info2 = SignInfo{
            pubkey: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            signature: "11bfe4d0b7705f6c57282a9030b22505ce2641547e9f246561d75a284f5a6e0a10e596fa7e70\
             2b6f897ad19c859ee880d4d1e80e521d91c53cc8827b67550002".to_string(),
        };
        let servant_device_sigs = vec![sign_info1,sign_info2];
        let mut contract = Contract::new();
        contract.send_money(servant_device_sigs,receiver_id,coin_account_id,amount);
    }
}
