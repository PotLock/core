use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise,
};

/// SybilProvider Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    account_ids_to_bool: UnorderedMap<AccountId, bool>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedContract {
    Current(Contract),
}

/// Convert VersionedContract to Contract
impl From<VersionedContract> for Contract {
    fn from(contract: VersionedContract) -> Self {
        match contract {
            VersionedContract::Current(current) => current,
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountIdsToBool,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            account_ids_to_bool: UnorderedMap::new(StorageKey::AccountIdsToBool),
        }
    }

    pub fn return_true(&self, account_id: AccountId) -> bool {
        true
    }

    pub fn return_false(&self, account_id: AccountId) -> bool {
        false
    }

    pub fn get_check(&mut self) {
        self.account_ids_to_bool
            .insert(&env::predecessor_account_id(), &true);
    }

    pub fn remove_check(&mut self) {
        self.account_ids_to_bool
            .remove(&env::predecessor_account_id());
    }

    pub fn has_check(&self, account_id: AccountId) -> bool {
        self.account_ids_to_bool.get(&account_id).is_some()
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            account_ids_to_bool: UnorderedMap::new(StorageKey::AccountIdsToBool),
        }
    }
}
