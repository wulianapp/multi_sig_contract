use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId};

pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

// Validator interface, for cross-contract calls
#[ext_contract(coin)]
trait Coin {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn transfer_from(
        &mut self,
        from_id: AccountId,
        to_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );

    fn transfer_from_nongas(
        &mut self,
        from_id: AccountId,
        to_id: AccountId,
        amount: U128,
        memo: Option<String>,
    );
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
