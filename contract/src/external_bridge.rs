use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

//todo: setup by func
pub const BRIDGE_ADDRESS:&'static str = "cvault0004.chainless";

// Validator interface, for cross-contract calls
#[ext_contract(bridge)]
trait Bridge {
    fn new_order(
        &mut self,
        chain_id: u128,
        account_id: AccountId,
        amount: u128,
        token: AccountId,
    );
}

