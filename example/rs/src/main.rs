pub mod meta_tx;

use chainless_jsonrpc_client::methods;
use chainless_jsonrpc_client::JsonRpcClient;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use lazy_static::lazy_static;
use meta_tx::{meta_call, send_meta_tx};
use near_crypto::InMemorySigner;
use near_crypto::{SecretKey, Signature, Signer};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use near_primitives::action::delegate::{DelegateAction, SignedDelegateAction};
use near_primitives::borsh::BorshSerialize;
use near_primitives::signable_message::{SignableMessage, SignableMessageType};
use near_primitives::token_balance::Deposit;
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, Transaction};
use near_primitives::types::AccountId;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{
    ExecutionOutcomeWithIdView, FinalExecutionOutcomeView, FinalExecutionStatus, QueryRequest,
    TokenBalanceList,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::str::FromStr;

lazy_static! {
    static ref CHAIN_CLIENT: JsonRpcClient = JsonRpcClient::connect("http://120.232.251.101:29162");
    static ref MULTI_SIG_CID: AccountId =
        AccountId::from_str("test.multiwallet.chainless").unwrap();
}
const RELAYER_URL: &str = "http://120.232.251.101:29163/send_meta_tx";

#[derive(Deserialize, Serialize, Clone)]
pub struct MtTransfer {
    pub to: AccountId,
    pub transfer_mt: String,
    pub fee_mt: String,
    pub amount: u128,
    pub memo: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PubkeySignInfo {
    pub pubkey: PublicKey,
    pub signature: String,
}

type Strategy = (Vec<u128>, Vec<u8>, Vec<PublicKey>, u128);

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

pub async fn gen_meta_transaction(
    signer: &InMemorySigner,
    actions: Vec<Action>,
    receiver_id: AccountId,
) -> Result<SignedDelegateAction, String> {
    let key_state = crate::CHAIN_CLIENT
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await
        .unwrap();

    let current_nonce = match key_state.kind {
        QueryResponseKind::AccessKey(access_key) => access_key.nonce,
        _ => Err("failed to extract current nonce").unwrap(),
    };

    let actions = actions
        .into_iter()
        .map(near_primitives::action::delegate::NonDelegateAction::try_from)
        .collect::<Result<_, _>>()
        .map_err(|_e| "Internal error: can not convert the action to non delegate action (delegate action can not be delegated again)".to_string())?;

    let delegate_action = DelegateAction {
        sender_id: signer.account_id.clone(),
        receiver_id,
        actions,
        nonce: current_nonce + 1,
        max_block_height: key_state.block_height + 1000,
        public_key: signer.public_key.clone(),
    };

    let signable = SignableMessage::new(&delegate_action, SignableMessageType::DelegateAction);
    let signature = signable.sign(signer);

    let meta_tx = SignedDelegateAction {
        delegate_action,
        signature,
    };

    Ok(meta_tx)
}

async fn get_balance(account: &AccountId, _symbol: &str) -> TokenBalanceList {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::None),
        request: QueryRequest::ViewTokenBalanceList {
            account_id: account.to_owned(),
        },
    };
    let rep = CHAIN_CLIENT.call(request).await.unwrap();
    println!("get_balance {:?}", rep);
    if let QueryResponseKind::TokenBalanceList(list) = rep.kind {
        list
    } else {
        unreachable!()
    }
}

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
            max_eq: 1000000000,
            sig_num: 0,
        },
        MultiSigRank {
            min: 1000000000,
            max_eq: 2000000000,
            sig_num: 1,
        },
        MultiSigRank {
            min: 2000000000,
            max_eq: 3000000000,
            sig_num: 2,
        },
    ]
}

async fn set_strategy(
    signer: InMemorySigner,
    servant_pubkeys: Vec<String>,
    mount_points: Vec<u128>,
    slave_needs: Vec<u8>,
) -> Result<String, String> {
    let set_strategy_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "set_strategy".to_string(),
        args: json!({
        "servant_pubkeys": servant_pubkeys,
        "rank_arr": rank_arr
        })
        .to_string()
        .into_bytes(),
        gas: 300000000000000, // 100 TeraGas
        deposit: None,        //Some(Deposit{ deposit: 0, symbol: None, fee: None }),
    }))];

    meta_call(&signer, set_strategy_actions, &MULTI_SIG_CID).await
}

async fn get_strategy(user_account_id: &AccountId) -> Option<Strategy> {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: (*MULTI_SIG_CID).clone(),
            method_name: "get_strategy".to_string(),
            args: FunctionArgs::from(
                json!({
                    "user_account_id":user_account_id.to_string()
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };
    println!("query_res1 {:?}", request);
    let rep = CHAIN_CLIENT.call(request).await.unwrap();
    println!("query_res1 {:?}", rep);

    if let QueryResponseKind::CallResult(result) = rep.kind {
        let amount_str: String = String::from_utf8(result.result).unwrap();
        println!("query_res1 {}", amount_str);
        serde_json::from_str::<Option<Strategy>>(&amount_str).unwrap()
    } else {
        panic!("")
    }
}

async fn send_money(
    signer: InMemorySigner,
    servant_device_sigs: Vec<PubkeySignInfo>,
    coin_tx: MtTransfer,
) -> Result<String, String> {
    let MtTransfer {
        transfer_mt,
        fee_mt,
        amount,
        ..
    } = coin_tx.clone();
    let set_strategy_actions = vec![Action::FunctionCall(Box::new(FunctionCallAction {
        method_name: "send_money".to_string(),
        args: json!({
            "servant_device_sigs": servant_device_sigs,
            "coin_tx": coin_tx,
        })
        .to_string()
        .into_bytes(),
        gas: 300000000000000, // 100 TeraGas
        deposit: Some(Deposit {
            deposit: amount,
            symbol: Some(transfer_mt),
            fee: Some(fee_mt),
        }),
    }))];
    meta_call(&signer, set_strategy_actions, &MULTI_SIG_CID).await
}

fn get_pubkey(pri_key_str: &str) -> String {
    let secret_key = near_crypto::SecretKey::from_str(pri_key_str).unwrap();
    let pubkey = secret_key.public_key().unwrap_as_ed25519().0.to_vec();
    pubkey.encode_hex()
}

fn ed25519_sign_data(prikey_bytes: &[u8], data: &str) -> String {
    let secret_key = ed25519_dalek::Keypair::from_bytes(&prikey_bytes).unwrap();
    secret_key.sign(data.as_bytes()).to_string()
}

#[tokio::main]
async fn main() {
    //eddy.chainless
    let pri_key = "ed25519:YDqZJcyWYeWN3pw6JBLwZtpkjASs5Q9rYUj3tKQyU719SErbrE75rZiXiWL75MhkF67T9wQZDBQHtCZioTZg1Vz";
    let secret_key: SecretKey = pri_key.parse().unwrap();
    let secret_key_bytes = secret_key.unwrap_as_ed25519().0.as_slice();
    //6a7a4df96a60c225f25394fd0195eb938eb1162de944d2c331dccef324372f45
    let main_device_pubkey = get_pubkey(&pri_key);
    println!("main_device_pubkey_{}", main_device_pubkey);
    let signer_account_id = AccountId::from_str("eddy.chainless").unwrap();
    let receiver_account_id = AccountId::from_str("test2.eddy.chainless").unwrap();
    let signer =
        near_crypto::InMemorySigner::from_secret_key(signer_account_id.to_owned(), secret_key);

    let strategy_str = get_strategy(&signer_account_id).await;
    println!("strategy_str2 {:#?}", strategy_str);

    let call_res = set_strategy(signer.clone(), Default::default(), dummy_ranks())
        .await
        .unwrap();

    println!("set_strategy_res {}", call_res);

    let strategy_str = get_strategy(&signer_account_id).await;
    println!("strategy_str {:#?}", strategy_str);

    let balance = get_balance(&signer_account_id, "").await;
    println!("account {}: balance {:?}", signer_account_id, balance);

    let strategy_str = get_strategy(&signer_account_id).await;
    println!("strategy_str {:#?}", strategy_str);

    let sender_balances = get_balance(&signer_account_id, "USDT").await;
    let receiver_balances = get_balance(&receiver_account_id, "USDT").await;
    println!(
        "sender_balances {:?},receiver_balances {:?}",
        sender_balances, receiver_balances
    );

    let transfer_amount = 1;
    let coin_tx_info = MtTransfer {
        to: receiver_account_id.clone(),
        transfer_mt: "USDT".to_string(),
        fee_mt: "USDT".to_string(),
        amount: transfer_amount,
        memo: None,
    };
    let serverns_sig = Default::default();
    let send_money_txid = send_money(signer, serverns_sig, coin_tx_info)
        .await
        .unwrap();

    println!("send_money_txid {}", send_money_txid);

    let sender_balances = get_balance(&signer_account_id, "USDT").await;
    let receiver_balances = get_balance(&receiver_account_id, "USDT").await;
    println!(
        "sender_balances {:?},receiver_balances {:?}",
        sender_balances, receiver_balances
    );
}
