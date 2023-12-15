use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise,
};

pub mod admins;
pub mod constants;
pub mod events;
pub mod internal;
pub mod owner;
pub mod projects;
pub mod source;
pub mod utils;
pub use crate::admins::*;
pub use crate::constants::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::projects::*;
pub use crate::source::*;
pub use crate::utils::*;

type ProjectId = AccountId;
type TimestampMs = u64;

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Contract superuser
    owner: AccountId,
    /// Contract admins (can be added/removed by owner)
    admins: UnorderedSet<AccountId>,
    /// Records of all Projects deployed by this Registry, indexed at their account ID, versioned for easy upgradeability
    project_ids: UnorderedSet<ProjectId>, // NB: this is unnecessary, but retained for now as it is implemented in v0
    projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
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
    Admins,
    ProjectIds,
    ProjectsById,
    SourceMetadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        admins: Vec<AccountId>,
        source_metadata: ContractSourceMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner,
            admins: account_vec_to_set(admins, StorageKey::Admins),
            project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            projects_by_id: LookupMap::new(StorageKey::ProjectsById),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
        }
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: AccountId::new_unchecked("".to_string()),
            admins: UnorderedSet::new(StorageKey::Admins),
            project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            projects_by_id: LookupMap::new(StorageKey::ProjectsById),
            contract_source_metadata: LazyOption::new(StorageKey::SourceMetadata, None),
        }
    }
}
