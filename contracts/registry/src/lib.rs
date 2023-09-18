use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey,
    Gas, Promise, PromiseError,
};

pub mod internal;
pub mod utils;
pub use crate::internal::*;
pub use crate::utils::*;

type ProjectId = AccountId;
type TimestampMs = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatus {
    Submitted,
    InReview,
    Approved,
    Rejected,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectInternal {
    pub id: ProjectId,
    pub name: String,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectExternal {
    pub id: ProjectId,
    pub name: String,
    pub team_members: Vec<AccountId>,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    project_ids: UnorderedSet<ProjectId>,
    projects_by_id: LookupMap<ProjectId, ProjectInternal>,
    project_team_members_by_project_id: LookupMap<ProjectId, UnorderedSet<AccountId>>,
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

    #[payable]
    pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        for admin in admins {
            self.admins.insert(&admin);
        }
    }

    pub fn get_admins(&self) -> Vec<AccountId> {
        self.admins.to_vec()
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        for admin in admins {
            self.admins.remove(&admin);
        }
    }

    #[payable]
    pub fn register(
        &mut self,
        name: String,
        team_members: Vec<AccountId>,
        _project_id: Option<AccountId>,
    ) -> ProjectExternal {
        // TODO: require enough funds to cover storage deposit
        // _project_id can only be specified by admin; otherwise, it is the caller
        let project_id = if let Some(_project_id) = _project_id {
            self.assert_admin();
            _project_id
        } else {
            env::predecessor_account_id()
        };
        self.assert_project_does_not_exist(&project_id);
        let project_internal = ProjectInternal {
            id: project_id.clone(),
            name,
            status: ProjectStatus::Approved, // approved by default - TODO: double-check that this is desired functionality
            submitted_ms: env::block_timestamp_ms(),
            updated_ms: env::block_timestamp_ms(),
            review_notes: None,
        };
        self.project_ids.insert(&project_id);
        self.projects_by_id.insert(&project_id, &project_internal);
        let mut team_members_set =
            UnorderedSet::new(StorageKey::ProjectTeamMembersByProjectIdInner {
                project_id: project_id.clone(),
            });
        for team_member in team_members {
            team_members_set.insert(&team_member);
        }
        self.project_team_members_by_project_id
            .insert(&project_id, &team_members_set);
        self.format_project(project_internal)
    }

    pub fn get_projects(&self) -> Vec<ProjectExternal> {
        self.project_ids
            .iter()
            .map(|project_id| {
                self.format_project(self.projects_by_id.get(&project_id).expect("No project"))
            })
            .collect()
    }

    pub fn get_project_by_id(&self, project_id: ProjectId) -> ProjectExternal {
        self.format_project(self.projects_by_id.get(&project_id).expect("No project"))
    }

    pub(crate) fn format_project(&self, project_internal: ProjectInternal) -> ProjectExternal {
        ProjectExternal {
            id: project_internal.id.clone(),
            name: project_internal.name,
            team_members: self
                .project_team_members_by_project_id
                .get(&project_internal.id)
                .unwrap()
                .to_vec(),
            status: project_internal.status,
            submitted_ms: project_internal.submitted_ms,
            updated_ms: project_internal.updated_ms,
            review_notes: project_internal.review_notes,
        }
    }

    #[payable]
    pub fn admin_set_project_status(
        &mut self,
        project_id: ProjectId,
        status: ProjectStatus,
        review_notes: Option<String>,
    ) {
        self.assert_admin();
        self.assert_project_exists(&project_id);
        let mut project = self.projects_by_id.get(&project_id).unwrap();
        project.status = status;
        project.review_notes = review_notes;
        project.updated_ms = env::block_timestamp_ms();
        self.projects_by_id.insert(&project_id, &project);
    }
}

// TODO: not sure why this is necessary
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
