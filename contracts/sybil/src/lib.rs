use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseError, PromiseResult,
};

pub mod admin;
pub mod constants;
pub mod events;
pub mod human;
pub mod internal;
pub mod owner;
pub mod providers;
pub mod source;
pub mod stamps;
pub mod utils;
pub use crate::admin::*;
pub use crate::constants::*;
pub use crate::events::*;
pub use crate::human::*;
pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::providers::*;
pub use crate::source::*;
pub use crate::stamps::*;
pub use crate::utils::*;

/// log prefix constant
pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";
pub type TimestampMs = u64;

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    providers_by_id: UnorderedMap<ProviderId, VersionedProvider>,
    // TODO: add active providers count, or sets of active provider IDs for easier targeted fetching
    default_provider_ids: UnorderedSet<ProviderId>,
    default_human_threshold: u32,
    // MAPPINGS
    // Stores all Stamp records, versioned for easy upgradeability
    stamps_by_id: UnorderedMap<StampId, VersionedStamp>,
    // Enables fetching of all stamps for a user
    provider_ids_for_user: LookupMap<AccountId, UnorderedSet<ProviderId>>,
    // Enables fetching of all users with given stamp (provider ID)
    user_ids_for_provider: LookupMap<ProviderId, UnorderedSet<AccountId>>,
    // Enables fetching of providers that a user has submitted (e.g. if user has submitted one malicious provider, they are likely to submit more and you'll want to be able to fetch these or filter them out of results)
    provider_ids_for_submitter: LookupMap<AccountId, UnorderedSet<ProviderId>>,
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

/// Ephemeral-only
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub default_provider_ids: Vec<ProviderId>,
    pub default_human_threshold: u32,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    SourceMetadata,
    Admins,
    ProvidersById,
    DefaultProviderIds,
    StampsById,
    ProviderIdsForUser,
    ProviderIdsForUserInner { user_id: AccountId },
    UserIdsForProvider,
    UserIdsForProviderInner { provider_id: ProviderId },
    SubmitterIdsForProvider,
    SubmitterIdsForProviderInner { provider_id: ProviderId },
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
            stamps_by_id: UnorderedMap::new(StorageKey::StampsById),
            provider_ids_for_user: LookupMap::new(StorageKey::ProviderIdsForUser),
            user_ids_for_provider: LookupMap::new(StorageKey::UserIdsForProvider),
            provider_ids_for_submitter: LookupMap::new(StorageKey::SubmitterIdsForProvider),
        }
    }

    pub fn get_config(&self) -> Config {
        Config {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            default_provider_ids: self.default_provider_ids.to_vec(),
            default_human_threshold: self.default_human_threshold,
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
            stamps_by_id: UnorderedMap::new(StorageKey::StampsById),
            provider_ids_for_user: LookupMap::new(StorageKey::ProviderIdsForUser),
            user_ids_for_provider: LookupMap::new(StorageKey::UserIdsForProvider),
            provider_ids_for_submitter: LookupMap::new(StorageKey::SubmitterIdsForProvider),
        }
    }
}
