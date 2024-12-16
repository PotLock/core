use near_sdk::{ext_contract, AccountId};

pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;
type ListId = u64;
type RegistrantId = AccountId;
#[ext_contract(list_contract)]
trait ListContract {
    fn is_registered(&self, list_id: Option<ListId>, account_id: RegistrantId) -> String;
}
