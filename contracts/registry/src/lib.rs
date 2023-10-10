use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey};

pub mod admins;
pub mod internal;
pub mod projects;
pub mod utils;
pub use crate::admins::*;
pub use crate::internal::*;
pub use crate::projects::*;
pub use crate::utils::*;

type ProjectId = AccountId;
type TimestampMs = u64;

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    project_ids: UnorderedSet<ProjectId>,
    projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    project_team_members_by_project_id: LookupMap<ProjectId, UnorderedSet<AccountId>>,
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
    ProjectTeamMembersByProjectId,
    ProjectTeamMembersByProjectIdInner { project_id: ProjectId },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, admins: Vec<AccountId>) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner,
            admins: account_vec_to_set(admins, StorageKey::Admins),
            project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            projects_by_id: LookupMap::new(StorageKey::ProjectsById),
            project_team_members_by_project_id: LookupMap::new(
                StorageKey::ProjectTeamMembersByProjectId,
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
            project_team_members_by_project_id: LookupMap::new(
                StorageKey::ProjectTeamMembersByProjectId,
            ),
        }
    }
}
