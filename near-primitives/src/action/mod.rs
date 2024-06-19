pub mod delegate;

use borsh::{BorshDeserialize, BorshSerialize};
use near_crypto::PublicKey;
use near_primitives_core::{
    account::AccessKey,
    serialize::dec_format,
    types::{AccountId, Balance, Gas},
};
use serde_with::base64::Base64;
use serde_with::serde_as;
use std::fmt;

use crate::types::{BalanceRatio, Fee, Role};

fn base64(s: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(s)
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct AddKeyAction {
    /// A public key which will be associated with an access_key
    pub public_key: PublicKey,
    /// An access key with the permission
    pub access_key: AccessKey,
}

/// Create account action
#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct CreateAccountAction {}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct DeleteAccountAction {
    pub beneficiary_id: AccountId,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct DeleteKeyAction {
    /// A public key associated with the access_key to be deleted.
    pub public_key: PublicKey,
}

/// Deploy contract action
#[serde_as]
#[derive(
    BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone,
)]
pub struct DeployContractAction {
    /// WebAssembly binary
    #[serde_as(as = "Base64")]
    pub code: Vec<u8>,
}

impl fmt::Debug for DeployContractAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeployContractAction")
            .field("code", &format_args!("{}", base64(&self.code)))
            .finish()
    }
}

#[serde_as]
#[derive(
    BorshSerialize, BorshDeserialize, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone,
)]
pub struct FunctionCallAction {
    pub method_name: String,
    #[serde_as(as = "Base64")]
    pub args: Vec<u8>,
    pub gas: Gas,
    pub deposit: Option<Deposit>,
}

impl fmt::Debug for FunctionCallAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("FunctionCallAction");
        debug
            .field("method_name", &format_args!("{}", &self.method_name))
            .field("args", &format_args!("{}", base64(&self.args)))
            .field("gas", &format_args!("{}", &self.gas));

        if let Some(deposit) = self.deposit.clone() {
            debug.field("deposit", &format_args!("{}", deposit));
        } else {
            debug.field("deposit", &format_args!("{}", 0));
        }

        debug.finish()
    }
}

/// An action which stakes signer_id tokens and setup's validator public key
#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct StakeAction {
    /// Amount of tokens to stake.
    #[serde(with = "dec_format")]
    pub stake: Balance,
    /// Validator key which will be used to sign transactions on behalf of signer_id
    pub public_key: PublicKey,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Deposit {
    #[serde(with = "dec_format")]
    pub deposit: Balance,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub fee: Option<String>,
}

impl Deposit {
    pub fn from_balance(balance: Balance) -> Self {
        Self {
            deposit: balance,
            ..Default::default()
        }
    }
}

impl fmt::Display for Deposit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct TransferAction {
    pub deposit: Deposit,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SetRoleAction {
    pub role: Role,
    pub enable: bool,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SetFeeAction {
    pub symbol: String,
    pub fee: Option<Fee>,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SetPriceAction {
    pub symbol: String,
    pub price: Option<BalanceRatio>,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MintBurnAction {
    pub symbol: String,
    #[serde(with = "dec_format")]
    pub amount: Balance,
    pub mint: bool,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    PartialEq,
    Eq,
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    strum::AsRefStr,
)]
pub enum Action {
    /// Create an (sub)account using a transaction `receiver_id` as an ID for
    /// a new account ID must pass validation rules described here
    /// <http://nomicon.io/Primitives/Account.html>.
    CreateAccount(CreateAccountAction),
    /// Sets a Wasm code to a receiver_id
    DeployContract(DeployContractAction),
    FunctionCall(Box<FunctionCallAction>),
    Transfer(Box<TransferAction>),
    Stake(Box<StakeAction>),
    AddKey(Box<AddKeyAction>),
    DeleteKey(Box<DeleteKeyAction>),
    DeleteAccount(DeleteAccountAction),
    Delegate(Box<delegate::SignedDelegateAction>),
    SetRole(Box<SetRoleAction>),
    SetFee(Box<SetFeeAction>),
    SetPrice(Box<SetPriceAction>),
    MintBurn(Box<MintBurnAction>),
}

const _: () = assert!(
    // 1 word for tag plus the largest variant `DeployContractAction` which is a 3-word `Vec`.
    // The `<=` check covers platforms that have pointers smaller than 8 bytes as well as random
    // freak nightlies that somehow find a way to pack everything into one less word.
    std::mem::size_of::<Action>() <= 32,
    "Action <= 32 bytes for performance reasons, see #9451"
);

impl Action {
    pub fn get_prepaid_gas(&self) -> Gas {
        match self {
            Action::FunctionCall(a) => a.gas,
            _ => 0,
        }
    }
    pub fn get_deposit(&self) -> Option<Deposit> {
        match self {
            Action::FunctionCall(a) => a.deposit.clone(),
            Action::Transfer(a) => Some(a.deposit.clone()),
            _ => None,
        }
    }
}

impl From<CreateAccountAction> for Action {
    fn from(create_account_action: CreateAccountAction) -> Self {
        Self::CreateAccount(create_account_action)
    }
}

impl From<DeployContractAction> for Action {
    fn from(deploy_contract_action: DeployContractAction) -> Self {
        Self::DeployContract(deploy_contract_action)
    }
}

impl From<FunctionCallAction> for Action {
    fn from(function_call_action: FunctionCallAction) -> Self {
        Self::FunctionCall(Box::new(function_call_action))
    }
}

impl From<TransferAction> for Action {
    fn from(transfer_action: TransferAction) -> Self {
        Self::Transfer(Box::new(transfer_action))
    }
}

impl From<StakeAction> for Action {
    fn from(stake_action: StakeAction) -> Self {
        Self::Stake(Box::new(stake_action))
    }
}

impl From<AddKeyAction> for Action {
    fn from(add_key_action: AddKeyAction) -> Self {
        Self::AddKey(Box::new(add_key_action))
    }
}

impl From<DeleteKeyAction> for Action {
    fn from(delete_key_action: DeleteKeyAction) -> Self {
        Self::DeleteKey(Box::new(delete_key_action))
    }
}

impl From<DeleteAccountAction> for Action {
    fn from(delete_account_action: DeleteAccountAction) -> Self {
        Self::DeleteAccount(delete_account_action)
    }
}

impl From<SetRoleAction> for Action {
    fn from(action: SetRoleAction) -> Self {
        Self::SetRole(Box::new(action))
    }
}

impl From<SetFeeAction> for Action {
    fn from(action: SetFeeAction) -> Self {
        Self::SetFee(Box::new(action))
    }
}

impl From<SetPriceAction> for Action {
    fn from(action: SetPriceAction) -> Self {
        Self::SetPrice(Box::new(action))
    }
}

impl From<MintBurnAction> for Action {
    fn from(action: MintBurnAction) -> Self {
        Self::MintBurn(Box::new(action))
    }
}
