use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
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

/// OLD (v1) Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractV1 {
    /// Contract superuser
    owner: AccountId,
    /// Contract admins (can be added/removed by owner)
    admins: UnorderedSet<AccountId>,
    /// Records of all Projects deployed by this Registry, indexed at their account ID, versioned for easy upgradeability
    project_ids: UnorderedSet<ProjectId>, // NB: this is unnecessary, but retained for now as it is implemented in v0
    projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    // /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    // contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
}

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Contract superuser
    owner: AccountId,
    /// Contract admins (can be added/removed by owner)
    admins: UnorderedSet<AccountId>,
    /// Old set (deprecated but empty set must be retained in state or serialization will break)
    _deprecated_project_ids: UnorderedSet<ProjectId>,
    /// Old map (deprecated but empty map must be retained in state or serialization will break)
    _deprecated_projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    /// Records of all Projects deployed by this Registry, indexed at their account ID, versioned for easy upgradeability
    projects_by_id: UnorderedMap<ProjectId, VersionedProjectInternal>,
    /// Projects pending approval
    pending_project_ids: UnorderedSet<ProjectId>,
    /// Projects approved
    approved_project_ids: UnorderedSet<ProjectId>,
    /// Projects rejected
    rejected_project_ids: UnorderedSet<ProjectId>,
    /// Projects graylisted
    graylisted_project_ids: UnorderedSet<ProjectId>,
    /// Projects blacklisted
    blacklisted_project_ids: UnorderedSet<ProjectId>,
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    /// Default status when project registers
    default_project_status: ProjectStatus,
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
    ProjectIds,   // deprecated but must not delete or change
    ProjectsById, // deprecated but must not delete or change
    ProjectsById2,
    PendingProjectIds,
    ApprovedProjectIds,
    RejectedProjectIds,
    GraylistedProjectIds,
    BlacklistedProjectIds,
    SourceMetadata,
}

/// Contract configuration
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractConfig {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub default_project_status: ProjectStatus,
    pub pending_project_count: u64,
    pub approved_project_count: u64,
    pub rejected_project_count: u64,
    pub graylisted_project_count: u64,
    pub blacklisted_project_count: u64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        admins: Vec<AccountId>,
        source_metadata: ContractSourceMetadata,
    ) -> Self {
        Self {
            owner,
            admins: account_vec_to_set(admins, StorageKey::Admins),
            _deprecated_project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            _deprecated_projects_by_id: LookupMap::new(StorageKey::ProjectsById),
            projects_by_id: UnorderedMap::new(StorageKey::ProjectsById2),
            pending_project_ids: UnorderedSet::new(StorageKey::PendingProjectIds),
            approved_project_ids: UnorderedSet::new(StorageKey::ApprovedProjectIds),
            rejected_project_ids: UnorderedSet::new(StorageKey::RejectedProjectIds),
            graylisted_project_ids: UnorderedSet::new(StorageKey::GraylistedProjectIds),
            blacklisted_project_ids: UnorderedSet::new(StorageKey::BlacklistedProjectIds),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
            default_project_status: ProjectStatus::Approved,
        }
    }

    pub fn get_config(&self) -> ContractConfig {
        ContractConfig {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            default_project_status: self.default_project_status.clone(),
            pending_project_count: self.pending_project_ids.len(),
            approved_project_count: self.approved_project_ids.len(),
            rejected_project_count: self.rejected_project_ids.len(),
            graylisted_project_count: self.graylisted_project_ids.len(),
            blacklisted_project_count: self.blacklisted_project_ids.len(),
        }
    }

    // for project_id in old_state.project_ids.iter() {
    //     let project_internal =
    //         ProjectInternal::from(old_state.projects_by_id.get(&project_id).unwrap());
    //     // add to projects_by_id
    //     projects_by_id.insert(
    //         &project_id,
    //         &VersionedProjectInternal::Current(project_internal.clone()),
    //     );
    //     log!(
    //         "Migrated project {} with status {:?}",
    //         project_id,
    //         project_internal.status
    //     );
    //     match project_internal.status {
    //         ProjectStatus::Pending => {
    //             pending_project_ids.insert(&project_id);
    //         }
    //         ProjectStatus::Approved => {
    //             approved_project_ids.insert(&project_id);
    //         }
    //         ProjectStatus::Rejected => {
    //             rejected_project_ids.insert(&project_id);
    //         }
    //         ProjectStatus::Graylisted => {
    //             graylisted_project_ids.insert(&project_id);
    //         }
    //         ProjectStatus::Blacklisted => {
    //             blacklisted_project_ids.insert(&project_id);
    //         }
    //     }
    // }

    #[private]
    pub fn migrate_chunk_temp(&mut self, project_ids: Vec<ProjectId>) {
        for project_id in project_ids {
            let project = self._deprecated_projects_by_id.get(&project_id);
            if let Some(project) = project {
                let project_internal = ProjectInternal::from(project);
                // if let Some(project_internal) = project_internal.clone() {
                // add to projects_by_id
                self.projects_by_id.insert(
                    &project_id,
                    &VersionedProjectInternal::Current(project_internal.clone()),
                );
                log!("Migrated project {}", project_id);
                match project_internal.status {
                    ProjectStatus::Pending => {
                        self.pending_project_ids.insert(&project_id);
                    }
                    ProjectStatus::Approved => {
                        self.approved_project_ids.insert(&project_id);
                    }
                    ProjectStatus::Rejected => {
                        self.rejected_project_ids.insert(&project_id);
                    }
                    ProjectStatus::Graylisted => {
                        self.graylisted_project_ids.insert(&project_id);
                    }
                    ProjectStatus::Blacklisted => {
                        self.blacklisted_project_ids.insert(&project_id);
                    }
                }
                self._deprecated_project_ids.remove(&project_id);
                self._deprecated_projects_by_id.remove(&project_id);
            } else {
                log!(format!(
                    "project ID {} not found in deprectaed projects by ID",
                    project_id
                ));
            }
        }
    }

    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: ContractV1 = env::state_read().expect("state read failed");
        // populate new maps/sets
        let projects_by_id = UnorderedMap::new(StorageKey::ProjectsById2);
        let pending_project_ids = UnorderedSet::new(StorageKey::PendingProjectIds);
        let approved_project_ids = UnorderedSet::new(StorageKey::ApprovedProjectIds);
        let rejected_project_ids = UnorderedSet::new(StorageKey::RejectedProjectIds);
        let graylisted_project_ids = UnorderedSet::new(StorageKey::GraylistedProjectIds);
        let blacklisted_project_ids = UnorderedSet::new(StorageKey::BlacklistedProjectIds);
        // old_state.project_ids.clear(); // don't need these anymore, but still need to keep mapping at StorageKey::ProjectIds
        Self {
            owner: old_state.owner,
            admins: old_state.admins,
            _deprecated_project_ids: old_state.project_ids,
            _deprecated_projects_by_id: old_state.projects_by_id,
            projects_by_id,
            pending_project_ids,
            approved_project_ids,
            rejected_project_ids,
            graylisted_project_ids,
            blacklisted_project_ids,
            contract_source_metadata: LazyOption::new(StorageKey::SourceMetadata, None),
            default_project_status: ProjectStatus::Approved,
        }
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: AccountId::new_unchecked("".to_string()),
            admins: UnorderedSet::new(StorageKey::Admins),
            _deprecated_project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            _deprecated_projects_by_id: LookupMap::new(StorageKey::ProjectsById),
            projects_by_id: UnorderedMap::new(StorageKey::ProjectsById2),
            pending_project_ids: UnorderedSet::new(StorageKey::PendingProjectIds),
            approved_project_ids: UnorderedSet::new(StorageKey::ApprovedProjectIds),
            rejected_project_ids: UnorderedSet::new(StorageKey::RejectedProjectIds),
            graylisted_project_ids: UnorderedSet::new(StorageKey::GraylistedProjectIds),
            blacklisted_project_ids: UnorderedSet::new(StorageKey::BlacklistedProjectIds),
            contract_source_metadata: LazyOption::new(StorageKey::SourceMetadata, None),
            default_project_status: ProjectStatus::Approved,
        }
    }
}
