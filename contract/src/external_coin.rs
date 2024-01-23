use near_sdk::{AccountId, ext_contract};
use near_sdk::json_types::U128;

pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

// Validator interface, for cross-contract calls
#[ext_contract(coin)]
trait Coin {
    fn ft_transfer(&mut self,
                   receiver_id: AccountId,
                   amount: U128,
                   memo: Option<String>,
    );
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn chainless_transfer_from(&mut self,
                               sender_id: AccountId,
                               receiver_id: AccountId,
                               amount: U128,
                               memo: Option<String>);
}


/***
   fn ft_transfer(
                &mut self,
                receiver_id: AccountId,
                amount: U128,
                memo: Option<String>,



            fn ft_balance_of(&self, account_id: AccountId) -> U128 {
                self.$token.ft_balance_of(account_id)
            }
*/