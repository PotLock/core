use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey,
    Gas, Promise, PromiseError,
};

pub mod utils;
pub use crate::utils::*;

type ProjectId = AccountId;
type TimestampMs = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatus {
    Draft,
    Submitted,
    InReview,
    Approved,
    Rejected,
}

pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub description: String,
    pub status: ProjectStatus,
    pub owner: AccountId,
    // team members by project stored on top level of Contract
    pub submitted_at: TimestampMs,
    pub updated_at: TimestampMs,
    pub review_notes: Option<String>,
}

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    projects_by_id: UnorderedMap<ProjectId, Project>,
    project_team_members_by_project_id: LookupMap<ProjectId, UnorderedSet<AccountId>>,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Admins,
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
            projects_by_id: UnorderedMap::new(StorageKey::ProjectsById),
            project_team_members_by_project_id: LookupMap::new(
                StorageKey::ProjectTeamMembersByProjectId,
            ),
        }
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: AccountId::new_unchecked("".to_string()),
            admins: UnorderedSet::new(StorageKey::Admins),
            projects_by_id: UnorderedMap::new(StorageKey::ProjectsById),
            project_team_members_by_project_id: LookupMap::new(
                StorageKey::ProjectTeamMembersByProjectId,
            ),
        }
    }
}
