use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, BorshStorageKey, Gas, Promise,
    PromiseError,
};

type TimestampMs = u64;
type ProjectId = u64; // TODO: change to AccountId?
type ApplicationId = u64;
type DonationId = u64; // TODO: change to Sring formatted as `"application_id:donation_id"`
type PayoutId = u64;

pub mod donations;
pub mod external;
pub mod internal;
pub mod sbt;
pub use crate::donations::*;
pub use crate::external::*;
pub use crate::internal::*;
pub use crate::sbt::*;

/// Pot Contract (funding round)
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Address (ID) of round manager ("chef"), essentially the contract owner
    // TODO: return error if a specified round manager is not a "chef" role in the refi.sputnik-dao.near contract
    pub chef_id: AccountId,
    /// Friendly & descriptive round name
    pub round_name: String,
    /// Friendly & descriptive round description
    pub round_description: String,
    /// Timestamp when the round starts
    pub start_time: TimestampMs,
    /// Timestamp when the round ends
    pub end_time: TimestampMs,
    /// Timestamp when applications can be submitted from
    pub application_start_ms: TimestampMs,
    /// Timestamp when applications can be submitted until
    pub application_end_ms: TimestampMs,
    /// Maximum number of projects that can be approved for the round
    pub max_projects: u32,
    /// Base currency for the round
    pub base_currency: AccountId,
    /// Account ID that deployed this Pot contract
    pub created_by: AccountId,
    /// If project raises less than this amount in donations, milestone submissions aren't required
    pub milestone_threshold: U64, // TODO: is this practical to implement?
    pub basis_points_paid_upfront: u32, // TODO: what does this mean? how will it be paid upfront if there are no donations yet?
    /// SBTs required to submit an application
    pub application_requirement: Option<SBTRequirement>,
    /// SBTs required to donate to a project
    pub donation_requirement: Option<SBTRequirement>,
    // payment_per_milestone: u32,
    pub patron_referral_fee_basis_points: u32,
    /// Max amount that can be paid to an account that referred a Patron
    pub max_patron_referral_fee: U128, // TODO: consider whether this is necessary
    /// Chef's fee for managing the round
    pub round_manager_fee_basis_points: u32, // TODO: should this be basis points or a fixed amount?
    /// Protocol fee
    pub protocol_fee_basis_points: u32, // e.g. 700 (7%)
    /// Amount of matching funds available
    pub matching_pool_balance: U128,

    // PROJECT MAPPINGS // TODO: update this
    /// All project records
    // pub projects_by_id: LookupMap<ProjectId, Project>,
    /// IDs of all projects
    pub project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that have been approved
    pub approved_project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that have been rejected
    pub rejected_project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that are pending approval
    pub pending_project_ids: UnorderedSet<ProjectId>,

    // DONATION MAPPINGS
    /// All donation records
    pub donations_by_id: LookupMap<DonationId, Donation>,
    /// IDs of donations made to a given project
    pub donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of donations made by a given donor (user)
    pub donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    // TODO: add records for matching pool donations
    /// IDs of matching pool donations
    pub patron_donation_ids: UnorderedSet<DonationId>,

    // PAYOUT MAPPINGS
    pub payouts_by_id: LookupMap<PayoutId, Payout>,
    pub payout_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<PayoutId>>,
}

// PAYOUTS

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// ID of the project receiving the payout
    pub project_id: ProjectId,
    /// Amount paid out
    pub amount: U128,
    /// Timestamp when the payout was made
    pub paid_at: TimestampMs,
}

// PROJECTS

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatus {
    Pending,
    Accepted,
    Rejected,
    InReview,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Application {
    /// Unique identifier for the application, increments from 1
    pub id: ApplicationId,
    /// ID of the individual or group that submitted the application. TODO: MUST be on the Potlock Registry (registry.potluck.[NETWORK])
    pub creator_id: AccountId,
    /// Name of the individual or group that submitted the application
    pub creator_name: Option<String>, // TODO: consider whether this should be required (currently optional)
    /// Account ID that should receive payout funds  
    pub payout_to: AccountId, // TODO: consider whether this should be updateable. Possibly shouldn't even exist (payouts should be made to the creator_id?)
    /// Status of the project application (Pending, Accepted, Rejected, InReview)
    pub status: ProjectStatus,
    /// Timestamp for when the application was submitted
    pub submitted_at: TimestampMs,
    /// Timestamp for when the application was reviewed (if applicable)
    pub reviewed_at: Option<TimestampMs>,
    /// Timestamp for when the project was updated
    // TODO: should only be updateable before it is approved
    pub updated_at: Option<TimestampMs>,
}

// REQUIREMENTS

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SBTRequirement {
    registry_id: AccountId,
    issuer_id: AccountId,
    class_id: u64,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ProjectIds,
    ApprovedProjectIds,
    RejectedProjectIds,
    PendingProjectIds,
    DonationsById,
    DonationIdsByProjectId,
    DonationIdsByProjectIdInner { project_id: ProjectId },
    DonationIdsByDonorId,
    DonationIdsByDonorIdInner { donor_id: AccountId },
    PatronDonationIds,
    PayoutsById,
    PayoutIdsByProjectId,
    ApplicationRequirements,
    DonationRequirements,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        chef_id: AccountId,
        round_name: String,
        round_description: String,
        start_time: TimestampMs,
        end_time: TimestampMs,
        application_start_ms: TimestampMs,
        application_end_ms: TimestampMs,
        max_projects: u32,
        base_currency: AccountId,
        created_by: AccountId,
        milestone_threshold: U64,
        basis_points_paid_upfront: u32,
        application_requirement: Option<SBTRequirement>,
        donation_requirement: Option<SBTRequirement>,
        // application_requirements: UnorderedSet<SBTRequirement>,
        // donation_requirements: UnorderedSet<SBTRequirement>,
        patron_referral_fee_basis_points: u32,
        max_patron_referral_fee: U128,
        round_manager_fee_basis_points: u32,
        protocol_fee_basis_points: u32,
        matching_pool_balance: U128,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            chef_id,
            round_name,
            round_description,
            start_time,
            end_time,
            application_start_ms,
            application_end_ms,
            max_projects,
            base_currency,
            created_by,
            milestone_threshold,
            basis_points_paid_upfront,
            application_requirement,
            donation_requirement,
            patron_referral_fee_basis_points,
            max_patron_referral_fee,
            round_manager_fee_basis_points,
            protocol_fee_basis_points,
            matching_pool_balance,
            project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            approved_project_ids: UnorderedSet::new(StorageKey::ApprovedProjectIds),
            rejected_project_ids: UnorderedSet::new(StorageKey::RejectedProjectIds),
            pending_project_ids: UnorderedSet::new(StorageKey::PendingProjectIds),
            donations_by_id: LookupMap::new(StorageKey::DonationsById),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            patron_donation_ids: UnorderedSet::new(StorageKey::PatronDonationIds),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutsById),
            payouts_by_id: LookupMap::new(StorageKey::PayoutsById),
        }
    }

    pub fn add_donation(&mut self, donation: Donation, application_id: AccountId) {
        // TODO: Implementation
    }

    pub fn approve_applicant(&mut self, applicant: Application, reason: String) {
        // TODO: Implementation
    }

    pub fn approve_multi_applicants(&mut self, applicants: Vec<Application>, reasons: Vec<String>) {
        // TODO: Implementation
    }

    pub fn reject_application(&mut self, applicant: Application, reason: String) {
        // TODO: Implementation
    }

    pub fn reject_multi_applicants(&mut self, applicants: Vec<Application>, reasons: Vec<String>) {
        // TODO: Implementation
    }

    pub fn add_to_matched_round(&mut self, amount: u64) {
        // TODO: Implementation
    }

    pub fn add_to_matched_round_referrer(&mut self, amount: u64, referrer_id: AccountId) {
        // TODO: Implementation
    }

    pub fn remove_approved_application(&mut self, application_id: AccountId, reason: String) {
        // TODO: Implementation
    }

    // ...etc
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            chef_id: AccountId::new_unchecked("".to_string()),
            round_name: "".to_string(),
            round_description: "".to_string(),
            start_time: 0,
            end_time: 0,
            application_start_ms: 0,
            application_end_ms: 0,
            max_projects: 0,
            base_currency: AccountId::new_unchecked("".to_string()),
            created_by: AccountId::new_unchecked("".to_string()),
            milestone_threshold: U64(0),
            basis_points_paid_upfront: 0,
            application_requirement: None,
            donation_requirement: None,
            patron_referral_fee_basis_points: 0,
            max_patron_referral_fee: U128(0),
            round_manager_fee_basis_points: 0,
            protocol_fee_basis_points: 0,
            matching_pool_balance: U128(0),
            project_ids: UnorderedSet::new(StorageKey::ProjectIds),
            approved_project_ids: UnorderedSet::new(StorageKey::ApprovedProjectIds),
            rejected_project_ids: UnorderedSet::new(StorageKey::RejectedProjectIds),
            pending_project_ids: UnorderedSet::new(StorageKey::PendingProjectIds),
            donations_by_id: LookupMap::new(StorageKey::DonationsById),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            patron_donation_ids: UnorderedSet::new(StorageKey::PatronDonationIds),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutsById),
            payouts_by_id: LookupMap::new(StorageKey::PayoutsById),
        }
    }
}
