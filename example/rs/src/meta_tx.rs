use std::str::FromStr;
//use ed25519_dalek::Signer;
use chainless_jsonrpc_client::methods;
use chainless_jsonrpc_client::JsonRpcClient;
use ed25519_dalek::Signer as DalekSigner;
use hex::ToHex;
use lazy_static::lazy_static;
use near_crypto::InMemorySigner;
use near_crypto::{SecretKey, Signature, Signer};
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::TransactionInfo;
use near_primitives::action::delegate::{DelegateAction, SignedDelegateAction};
use near_primitives::borsh::BorshSerialize;
use near_primitives::signable_message::{SignableMessage, SignableMessageType};
use near_primitives::transaction::{Action, FunctionCallAction, SignedTransaction, Transaction};
use near_primitives::types::AccountId;
use near_primitives::types::{BlockReference, Finality, FunctionArgs};
use near_primitives::views::{
    ExecutionOutcomeWithIdView, FinalExecutionOutcomeView, FinalExecutionStatus, QueryRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

const RELAYER_URL: &str = "http://120.232.251.101:29163/send_meta_tx";

//todo: 本地meta签

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct ExecuteMetTxOutcome {
    pub status: FinalExecutionStatus,
    /// Signed Transaction
    pub message: String,
    /// The execution outcome of the signed transaction.
    #[serde(rename = "Transaction Outcome")]
    pub transaction_outcome: ExecutionOutcomeWithIdView,
    #[serde(rename = "Receipts Outcome")]
    pub receipts_outcome: Vec<ExecutionOutcomeWithIdView>,
}

pub async fn gen_meta_transaction(
    signer: &InMemorySigner,
    actions: Vec<Action>,
    receiver_id: &AccountId,
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
        receiver_id: receiver_id.to_owned(),
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

pub async fn send_meta_tx(delete_action: SignedDelegateAction) -> Result<String, String> {
    let meta_tx_json = serde_json::to_string(&delete_action).unwrap();
    let res_text = reqwest::Client::new()
        .post(RELAYER_URL)
        .header("Content-Type", "application/json")
        .body(meta_tx_json)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    //println!("res_text {}",res_text);
    let exe_outcome = serde_json::from_str::<ExecuteMetTxOutcome>(&res_text).unwrap();
    if let FinalExecutionStatus::Failure(error) = exe_outcome.status {
        Err(error.to_string())?;
    }
    let tx_id = exe_outcome.transaction_outcome.id.to_string();
    Ok(tx_id)
}

pub async fn meta_call(
    signer: &InMemorySigner,
    actions: Vec<Action>,
    receiver_id: &AccountId,
) -> Result<String, String> {
    let meta_tx = gen_meta_transaction(&signer, actions, receiver_id).await?;
    let tx_id = send_meta_tx(meta_tx).await.unwrap();
    Ok(tx_id)
}
