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
pub struct Contract {}

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

#[near_bindgen]
impl Contract {
    // #[init]
    // pub fn new(
    //     source_metadata: Option<ContractSourceMetadata>,
    //     owner: AccountId,
    //     admins: Option<Vec<AccountId>>,
    // ) -> Self {
    //     assert!(!env::state_exists(), "Already initialized");
    //     let versioned_metadata = source_metadata.map(VersionedContractSourceMetadata::Current);
    //     Self {
    //         contract_source_metadata: LazyOption::new(
    //             StorageKey::SourceMetadata,
    //             versioned_metadata.as_ref(),
    //         ),
    //         owner,
    //         admins: account_vec_to_set(
    //             if admins.is_some() {
    //                 admins.unwrap()
    //             } else {
    //                 vec![]
    //             },
    //             StorageKey::Admins,
    //         ),
    //         providers_by_id: UnorderedMap::new(StorageKey::ProvidersById),
    //     }
    // }

    pub fn return_true(&self, account_id: AccountId) -> bool {
        true
    }

    pub fn return_false(&self, account_id: AccountId) -> bool {
        false
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {}
    }
}
