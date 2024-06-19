use std::str::FromStr;
//use ed25519_dalek::Signer;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use near_crypto::{SecretKey, Signature, Signer};
use near_jsonrpc_client::methods;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, Transaction};
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{FinalExecutionStatus, QueryRequest};
use serde_json::json;
use near_jsonrpc_client::{JsonRpcClient};
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
//use near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo;
use near_crypto::InMemorySigner;
use near_primitives::types::AccountId;
use lazy_static::lazy_static;
use near_primitives::borsh::BorshSerialize;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MultiSigRank {
    min: u128,
    max_eq: u128,
    sig_num: u8,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SubAccConf {
    pubkey:String,
    hold_value_limit: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StrategyData {
    multi_sig_ranks: Vec<MultiSigRank>,
    main_device_pubkey: String,
    servant_device_pubkey: Vec<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SignInfo {
    pubkey: String,
    signature: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoinTx {
    from: AccountId,
    to: AccountId,
    coin_id:AccountId,
    amount:u128,
    memo:Option<String>,
    expire_at: u64,
}

lazy_static! {
    static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://120.232.251.101:29162");
    static ref MULTI_SIG_CID: AccountId = AccountId::from_str("test.multiwallet.chainless").unwrap();
    static ref DW20_CID: AccountId = AccountId::from_str("dw20.node0").unwrap();
}

pub async fn gen_transaction(signer: &InMemorySigner, contract_addr: &str) -> Transaction {
    println!("___{}__{}_", signer.account_id, signer.public_key);
    let access_key_query_response = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await
        .unwrap();

    let current_nonce = match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce").unwrap(),
    };

    Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce: current_nonce + 1,
        receiver_id: contract_addr.parse().unwrap(),
        block_hash: access_key_query_response.block_hash,
        actions: vec![],
    }
}

/*** 
async fn get_balance(account: &AccountId) -> u128 {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: (*DW20_CID).clone(),
            method_name: "ft_balance_of".to_string(),
            args: FunctionArgs::from(json!({
                "account_id":account.to_string()
            }).to_string().into_bytes()),
        },
    };
    let rep = CHAIN_CLIENT.call(request).await.unwrap();

    if let QueryResponseKind::CallResult(result) = rep.kind {
        let amount_str: String = String::from_utf8(result.result).unwrap().split("\"").collect();
        u128::from_str(&amount_str).unwrap()
    } else {
        unreachable!()
    }
}
**/


fn servant_keys() -> Vec<String> {
    vec![
        "ed25519:s1sw1PXCkHrbyE9Rmg6j18PoUxnhCNZ2CxSPUvvE7dZK9UCEkpTWC1Zy6ZKWvBcAdK8MoRUSdMsduMFRJrRtuGq".to_string(),
        "ed25519:5cNJ9mg3b3VZoiTyimwz3YZhimF5KTDuV1DMU6TMhR1RR3NtXtArxFizDRoRo4kgUJQdQzM1urNxCKbftNhLNN5v".to_string(),
        "ed25519:4D2nFZNxfpCmTBPZhgEGJs2rFeLEe9MhBVNzZyr5XiYL92PnSbYBUbAmPnx4qhi6WQkrFGasNjTdLMNDrj3vRjQU".to_string(),
        "ed25519:vUxMDvDoFVT9qxNZWDpc7TLjK4W8MLGnL6UvardxbcptYtm2VJxaiFq9rZ6LMfxxzs5NVQKpr5UYHaq8Gw9LPZA".to_string(),
        "ed25519:5E398aXyuB2rHmAgGSKVunaEFnvRDJA8v9WjBGv84sxXLSEHAphfo99xbRGmvghnx1befSyLNkiYVbu4M8aaSg8m".to_string(),
        "ed25519:3rZKJGN6qQDWqEKge3gFm4KqqmNWJ7B8VXSz9f5wEFjgwVU81U6nF4iFF75DvReKaqoRxncBTi5HL5n8UPx9n9g4".to_string(),
        "ed25519:3TYRq9LstrATmGetoT2daaK7LCuCtnoP6Vt6JfGe2GBT49iqQLGnj8g8AVDeUStvSbCjwVEhwYnvyCoAyrmGD1sp".to_string(),
        "ed25519:cM3cWYumPkSTn56ELfn2mTTYdf9xzJMJjLQnCFq8dgbJ3x97hw7ezkrcnbk4nedPLPMga3dCGZB51TxWaGuPtwE".to_string(),
    ]
}

fn dummy_ranks() -> Vec<MultiSigRank> {
    vec![
        MultiSigRank {
            min: 0,
            max_eq: 100,
            sig_num: 1,
        },
        MultiSigRank {
            min: 100,
            max_eq: 10000,
            sig_num: 2,
        },
        MultiSigRank {
            min: 10000,
            max_eq: 999999999999,
            sig_num: 3,
        },
    ]
}


async fn set_strategy(
                      signer: InMemorySigner,
                      master_pubkey: String,  
                      user_account_id: &AccountId,
                      servant_pubkeys: Vec<String>,
                      sub_confs: BTreeMap<AccountId,SubAccConf>,
                      rank_arr: Vec<MultiSigRank>
) -> Result<String, String> {
    let set_strategy_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "set_strategy2".to_string(),
        args: json!({
                "master_pubkey": master_pubkey,
                "user_account_id": user_account_id,
                "servant_pubkeys": servant_pubkeys,
                "sub_confs": sub_confs,
                "rank_arr": rank_arr
            })
            .to_string()
            .into_bytes(),
        gas: 300000000000000, // 100 TeraGas
        deposit: None,
    }))];

    let mut transaction = gen_transaction(&signer, &MULTI_SIG_CID.to_string()).await;
    transaction.actions = set_strategy_actions;
    let signature = signer.sign(transaction.get_hash_and_size().0.as_ref());


    let tx = SignedTransaction::new(signature, transaction);
    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: tx.clone(),
    };

    println!("call set strategy txid {}",&tx.get_hash().to_string());

    let rep = CHAIN_CLIENT.call(request).await.unwrap();
    if let FinalExecutionStatus::Failure(error) = rep.status {
        Err(error.to_string())?;
    }
    let tx_id = rep.transaction.hash.to_string();
    Ok(tx_id)
}

async fn get_strategy(user_account_id: &AccountId) -> Option<StrategyData> {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: (*MULTI_SIG_CID).clone(),
            method_name: "get_strategy".to_string(),
            args: FunctionArgs::from(json!({
                "user_account_id":user_account_id.to_string()
            }).to_string().into_bytes()),
        },
    };
    let rep = CHAIN_CLIENT.call(request).await;
    println!("0001_ {:?}",rep);
    return None;
    /***
    if let QueryResponseKind::CallResult(result) = rep.kind {
        let amount_str: String = String::from_utf8(result.result).unwrap();
        println!("strategy_str {}", amount_str);
        Some(serde_json::from_str::<StrategyData>(&amount_str).unwrap())
    } else {
        None
    }
    */
}


async fn send_money(
    signer: InMemorySigner,
    servant_device_sigs: Vec<SignInfo>,
    coin_tx: CoinTx,
) -> Result<String, String>{
    //let CoinTx{from,to,coin_id,amount,memo} = coin_tx;
    let set_strategy_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "send_money".to_string(),
        args: json!({
                "servant_device_sigs": servant_device_sigs,
                "coin_tx": coin_tx,
            })
            .to_string()
            .into_bytes(),
        gas: 300000000000000, // 100 TeraGas
        deposit: None,
    }))];

    let mut transaction = gen_transaction(&signer, &MULTI_SIG_CID.to_string()).await;
    transaction.actions = set_strategy_actions;
    let signature = signer.sign(transaction.get_hash_and_size().0.as_ref());

    let tx = SignedTransaction::new(signature, transaction);
    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: tx.clone(),
    };

    println!("call set strategy txid {}",&tx.get_hash().to_string());

    let rep = CHAIN_CLIENT.call(request).await.unwrap();
    if let FinalExecutionStatus::Failure(error) = rep.status {
        Err(error.to_string())?;
    }
    let tx_id = rep.transaction.hash.to_string();
    Ok(tx_id)
}

fn get_pubkey(pri_key_str:&str) -> String{
    let secret_key = near_crypto::SecretKey::from_str(pri_key_str).unwrap();
    let pubkey = secret_key.public_key().unwrap_as_ed25519().0.to_vec();
    pubkey.encode_hex()
}

/***

pub fn ed25519_sign_data2(prikey_bytes_hex: &str, data_hex: &str) -> String {
   let prikey_bytes = hex::decode(prikey_bytes_hex).unwrap();
    let data = hex::decode(data_hex).unwrap();

    println!("ed25519_secret {:?}", prikey_bytes);
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    let sig = secret_key.sign(&data);
    sig.to_string()
}
*/
fn ed25519_sign_data(prikey_bytes:&[u8], data:&str) -> String{
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    secret_key.sign(data.as_bytes()).to_string()
}

/***
fn gen_replace_action(){
    let set_strategy_actions = vec![
        Action::FunctionCall(*Box::new(FunctionCallAction {
        method_name: "send_money".to_string(),
        args: json!({
                "servant_device_sigs": servant_device_sigs,
                "coin_tx": coin_tx,
            })
            .to_string()
            .into_bytes(),
        gas: 300000000000000, // 100 TeraGas
        deposit: 0,
    })),
        
    ];

    let mut transaction = gen_transaction(&signer, &MULTI_SIG_CID.to_string()).await;
    transaction.actions = set_strategy_actions;
    let signature = signer.sign(transaction.get_hash_and_size().0.as_ref());

    let tx = SignedTransaction::new(signature, transaction);
    let request = methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
        signed_transaction: tx.clone(),
    };

    println!("call set strategy txid {}",&tx.get_hash().to_string());

    let rep = CHAIN_CLIENT.call(request).await.unwrap();
    if let FinalExecutionStatus::Failure(error) = rep.status {
        Err(error.to_string())?;
    }
    let tx_id = rep.transaction.hash.to_string();
    Ok(tx_id)
}
***/
#[tokio::main]
async fn main() {
    //eddy.chainless
    let pri_key = "ed25519:YDqZJcyWYeWN3pw6JBLwZtpkjASs5Q9rYUj3tKQyU719SErbrE75rZiXiWL75MhkF67T9wQZDBQHtCZioTZg1Vz";
    let secret_key: SecretKey = pri_key.parse().unwrap();
    let secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
    //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
    let main_device_pubkey = get_pubkey(&pri_key);
    println!("main_device_pubkey_{}",main_device_pubkey);
    let signer_account_id = AccountId::from_str("eddy.chainless").unwrap();
    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id.to_owned(), secret_key);

    let strategy_str = get_strategy(&signer_account_id).await;
    println!("strategy_str2 {:#?}", strategy_str);

    /***
     * 
        signer: InMemorySigner,
                      master_pubkey: String,  
                      user_account_id: &AccountId,
                      servant_pubkeys: Vec<String>,
                      sub_confs: BTreeMap<AccountId,SubAccConf>,
                      rank_arr: Vec<MultiSigRank>
    */

    let set_strategy_res = set_strategy(
            signer,
            main_device_pubkey,
            &signer_account_id,
            Default::default(),
            Default::default(),
            Default::default()
        ).await.unwrap();
        println!("set_strategy_res {:#?}", set_strategy_res);

    return;

    /***
    let servant_pubkey = servant_keys().as_slice()[..3].iter().map(|x| {
        let secret_key = near_crypto::SecretKey::from_str(x).unwrap();
        let pubkey = secret_key.public_key().try_to_vec().unwrap();
        pubkey.as_slice()[1..].to_vec().encode_hex()
    }).collect::<Vec<String>>();

    println!("{:?}",servant_pubkey);

    let ranks = dummy_ranks();

    let balance = get_balance(&signer_account_id).await;
    println!("account {}: balance {}", main_device_pubkey,balance);

    let strategy_str = get_strategy(&signer_account_id).await;
    println!("strategy_str2 {:#?}", strategy_str);
    //let set_strategy_res = set_strategy(&user_account_id,main_device_pubkey,servant_pubkey,signer,ranks).await.unwrap();
    //println!("call set strategy txid {}",set_strategy_res);
    //let strategy_str2 = get_strategy(&user_account_id).await;
    //println!("strategy_str2 {:#?}", strategy_str2);
    //send_money 1 dw20, 1 servant is enough

    let transfer_amount = 1;
    let coin_tx_info = CoinTx {
        from: signer_account_id,
        to: receiver_id,
        coin_id: DW20_CID.to_owned(),
        amount: transfer_amount,
        memo:None,
        expire_at: 1808570727000,
    };
    let coin_tx_str = serde_json::to_string(&coin_tx_info).unwrap();




    let sigs: Vec<SignInfo> = servant_keys().as_slice()[..1].iter().map(|x|
        {
            let prikey: SecretKey = x.parse().unwrap();
            let prikey_byte = prikey.unwrap_as_ed25519().0.as_slice();
            println!("prikey {}",hex::encode(prikey_byte));
            let sig = SignInfo {
                pubkey: get_pubkey(x),
                signature: ed25519_sign_data(prikey_byte, &coin_tx_str),
            };
            println!("{:#?}",sig);
            sig
        }
    ).collect();
    let send_money_txid = send_money(signer,sigs,coin_tx_info).await.unwrap();
    println!("send_money_txid {}", send_money_txid);
    ***/
}
