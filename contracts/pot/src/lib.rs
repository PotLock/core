use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::{near_bindgen, AccountId, Promise};
use std::collections::HashMap;

type TimestampMs = u64;
type MilestoneId = u64;
type ProjectId = u64;
type DonationId = u64;
type PayoutId = u64;

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
    pub application_requirements: UnorderedSet<SBTRequirement>,
    /// SBTs required to donate to a project
    pub donation_requirements: UnorderedSet<SBTRequirement>,
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

    // PROJECT MAPPINGS
    /// All project records
    pub projects_by_id: LookupMap<ProjectId, Project>,
    /// IDs of all projects
    pub project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that have been approved
    pub approved_project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that have been rejected
    pub rejected_project_ids: UnorderedSet<ProjectId>,
    /// IDs of projects that are pending approval
    pub pending_project_ids: UnorderedSet<ProjectId>,

    // MILESTONE MAPPINGS
    /// All milestone records
    pub milestones_by_id: LookupMap<MilestoneId, Milestone>,
    /// IDs of milestones for a given project
    pub milestone_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<MilestoneId>>,

    // REVIEW MAPPINGS
    /// All review records
    pub reviews_by_id: LookupMap<ReviewId, Review>,
    /// IDs of reviews made on a given milestone
    pub reviews_by_milestone_id: LookupMap<MilestoneId, UnorderedSet<Review>>,

    // DONATION MAPPINGS
    /// All donation records
    pub donations_by_id: LookupMap<DonationId, Donation>,
    /// IDs of donations made to a given project
    pub donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of donations made by a given donor
    pub donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    /// IDs of end-user donations (aka not matching pool donations)
    pub user_donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of matching pool donations
    pub patron_donation_ids: UnorderedSet<DonationId>,

    // PAYOUT MAPPINGS
    pub payouts_by_id: LookupMap<PayoutId, Payout>,
    pub payout_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<PayoutId>>,
}

// PAYOUTS

#[derive(BorshDeserialize, BorshSerialize)]
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

// DONATIONS

/// Could be an end-user donation (must include a project_id in this case) or a matching pool donation (may include a referrer_id in this case)
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation        
    pub project_id: Option<ProjectId>,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
}

// PROJECTS

#[derive(BorshDeserialize, BorshSerialize)]
pub enum ProjectStatus {
    Pending,
    Accepted,
    Rejected,
    InReview,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Project {
    /// Unique identifier for the application, increments from 1
    pub id: ProjectId,
    /// Title of the project
    pub project_title: String,
    /// Detailed text explaining the purpose of the project
    pub project_text: String,
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

// MILESTONES

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MilestoneStatus {
    NotStarted,
    InProgress,
    Submitted,
    Approved,
    Rejected,
    InReview,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Milestone {
    /// Unique identifier for the milestone, increments from 1
    pub id: String,
    /// Milestone title
    pub title: String,
    /// Detailed description of the milestone
    pub description: String,
    /// Amount of funds required for the milestone
    pub payout_amount: U128,
    /// Timestamp when the milestone is due
    pub due_at: Option<TimestampMs>,
    /// Status of the milestone (NotStarted, InProgress, Submitted, Approved, Rejected, InReview)
    pub status: MilestoneStatus,
    /// Timestamp when the milestone was submitted
    pub submitted_at: Option<TimestampMs>,
    /// Timestamp when the milestone was paid
    pub paid_at: Option<TimestampMs>,
}

// REVIEWS

#[derive(BorshDeserialize, BorshSerialize)]
pub enum ReviewStatus {
    Open,
    Resolved,
}

// A Review is a comment or change request on a Milestone
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Review {
    pub milestone_id: Option<MilestoneId>,
    pub comment: String,
    pub status: ReviewStatus,
    pub updated_at: TimestampMs,
}

// REQUIREMENTS

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SBTRequirement {
    issuer_address: AccountId,
    class_id: String, // TODO: is this the right type?
}

#[near_bindgen]
impl Pot {
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
