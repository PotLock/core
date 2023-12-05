use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseResult,
};

pub mod events;
pub mod internal;
pub mod providers;
pub mod source;
pub mod utils;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::providers::*;
pub use crate::source::*;
pub use crate::utils::*;

type TimestampMs = u64;

/// log prefix constant
pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    providers_by_id: UnorderedMap<ProviderId, VersionedProvider>,
    default_provider_ids: UnorderedSet<ProviderId>,
    default_human_threshold: u32,
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
    SourceMetadata,
    Admins,
    ProvidersById,
    DefaultProviderIds,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        source_metadata: Option<ContractSourceMetadata>,
        owner: AccountId,
        admins: Option<Vec<AccountId>>,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let versioned_metadata = source_metadata.map(VersionedContractSourceMetadata::Current);
        Self {
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                versioned_metadata.as_ref(),
            ),
            owner,
            admins: account_vec_to_set(
                if admins.is_some() {
                    admins.unwrap()
                } else {
                    vec![]
                },
                StorageKey::Admins,
            ),
            providers_by_id: UnorderedMap::new(StorageKey::ProvidersById),
            default_provider_ids: UnorderedSet::new(StorageKey::DefaultProviderIds),
            default_human_threshold: 0,
        }
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(
                    ContractSourceMetadata {
                        version: "1.0.0".to_string(),
                        commit_hash: "12345".to_string(),
                        link: "www.example.com".to_string(),
                    },
                )),
            ),
            owner: AccountId::new_unchecked("".to_string()),
            admins: account_vec_to_set(vec![], StorageKey::Admins),
            providers_by_id: UnorderedMap::new(StorageKey::ProvidersById),
            default_provider_ids: UnorderedSet::new(StorageKey::DefaultProviderIds),
            default_human_threshold: 0,
        }
    }
}
