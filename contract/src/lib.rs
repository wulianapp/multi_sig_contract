#![feature(exclusive_range_pattern)]

mod external_coin;
pub use crate::external_coin::*;


use std::cmp::max;
use std::collections::HashMap;
use std::fmt::format;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, env, AccountId, require, Gas, Promise, PromiseError, serde_json};
use near_sdk::json_types::U128;
use near_sdk::serde::{Serialize, Deserialize};
use ed25519_dalek::Verifier;
use uint::hex;


// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner: AccountId,
    user_strategy: HashMap<AccountId, StrategyData>,
}

//delete it
impl Default for Contract {
    // The default trait with which to initialize the contract
    fn default() -> Self {
        Self {
            owner: AccountId::from_str("node0").unwrap(),
            user_strategy: HashMap::new(),
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

    fn get_user_strategy(&self,account_id:&AccountId) -> StrategyData {
        match self.user_strategy.get(account_id) {
            Some(data) => data.to_owned(),
            None => {
                env::panic_str(format!("{} haven't register multi_sig account", &account_id).as_str());
            }
        }
    }

    fn call_chainless_transfer_from(sender_id:&AccountId,coin_id:&AccountId,receiver_id:&AccountId,amount: U128,memo:Option<String>) -> Promise{
        log!("start transfer {}(coin_id) {}(sender_id) {}(receiver_id) {}(amount)",
                     coin_id.to_string(),
                     sender_id.to_string(),
                     receiver_id.to_string(),
                     amount.0
        );
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
    pub fn call_chainless_transfer_from_callback(&self, #[callback_result] call_result: Result<(), PromiseError>) -> bool {
        // Return whether or not the promise succeeded using the method outlined in external_coin
        //fixme: get return info
        if call_result.is_err() {
            env::log_str("ft_transfer failed...");
            return false;
        } else {
            env::log_str("ft_transfer was successful!");
            return true;
        }
    }

    #[init]
    #[private]
    pub fn new() -> Self{
        let caller = env::signer_account_id();
        Contract{
            owner: caller,
            user_strategy: HashMap::new(),
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
        self.user_strategy.insert(user_account_id.clone(), strategy);
        log!("set {}'s strategy successfully",user_account_id.to_string());
    }


    pub fn set_strategy2(&mut self,
                        user_account_id: AccountId,
                        main_device_pubkey: String,
                        servant_device_pubkey: Vec<String>,
                        rank_arr: Vec<MultiSigRank>,
                        loop_time: u32

    ) {

        //todo: span must be serial
        //todo: must be called by owner
        //let multi_sig_ranks = rank_arr.iter().map(|&x| x.into()).collect();
        let multi_sig_ranks = rank_arr;
        let strategy = StrategyData {
            multi_sig_ranks,
            main_device_pubkey,
            servant_device_pubkey,
        };
        let range :Vec<u32> = (0..loop_time).collect();
        for x in range {
            let x = format!("{}.node0",x);
            let user_account_id = AccountId::from_str(&x).unwrap();
            self.user_strategy.insert(user_account_id.clone(), strategy.clone());
        }
        log!("set {}'s strategy successfully",user_account_id.to_string());
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
        self.user_strategy.get(&user_account_id).as_ref().map(|&data| (*data).clone())
    }

    pub fn send_money(&self,
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

            if env::block_timestamp_ms() > expire_at {
                Err("signature have been expired")?
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
        Self::call_chainless_transfer_from(&caller,&coin_id,&to,amount.into(),memo)
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
