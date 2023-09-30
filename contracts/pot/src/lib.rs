use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, AccountId, BorshStorageKey, Gas, Promise,
    PromiseError,
};

type TimestampMs = u64;
type ProjectId = AccountId;
type ApplicationId = u64;
type DonationId = u64; // TODO: change to Sring formatted as `"application_id:donation_id"`

pub mod admin;
pub mod applications;
pub mod config;
pub mod constants;
pub mod donations;
pub mod external;
pub mod internal;
pub mod payouts;
pub mod sbt;
pub use crate::admin::*;
pub use crate::applications::*;
pub use crate::config::*;
pub use crate::constants::*;
pub use crate::donations::*;
pub use crate::external::*;
pub use crate::internal::*;
pub use crate::payouts::*;
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
    /// MS Timestamp when the round starts
    pub round_start_ms: TimestampMs,
    /// MS Timestamp when the round ends
    pub round_end_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted from
    pub application_start_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted until
    pub application_end_ms: TimestampMs,
    /// Maximum number of projects that can be approved for the round
    pub max_projects: u32,
    /// Base currency for the round
    pub base_currency: AccountId, // TODO: add FT support
    /// Account ID that deployed this Pot contract
    pub created_by: AccountId,
    /// Account ID of registry contract that should be queried when projects apply to round
    pub registry_contract_id: AccountId,
    /// Account ID of pot deployer contract (singleton) that is queried to verify admin status
    pub pot_deployer_contract_id: AccountId,
    // /// If project raises less than this amount in donations, milestone submissions aren't required
    // pub milestone_threshold: U64, // TODO: is this practical to implement?
    // pub basis_points_paid_upfront: u32, // TODO: what does this mean? how will it be paid upfront if there are no donations yet?
    // /// SBTs required to submit an application
    // pub application_requirement: Option<SBTRequirement>,
    /// SBTs required to donate to a project
    pub donation_requirement: Option<SBTRequirement>,
    // payment_per_milestone: u32,
    pub patron_referral_fee_basis_points: u32, // TODO: implement referral fees
    /// Max amount that can be paid to an account that referred a Patron
    pub max_patron_referral_fee: U128, // TODO: consider whether this is necessary
    /// Chef's fee for managing the round
    pub chef_fee_basis_points: u32, // TODO: should this be basis points or a fixed amount?
    /// Protocol fee
    pub protocol_fee_basis_points: u32, // e.g. 700 (7%)
    /// Account that protocol fee should be transferred to
    pub protocol_fee_recipient_account: AccountId,
    /// Amount of matching funds available
    pub matching_pool_balance: U128, // TODO: may want to change this to U128?
    /// Amount of donated funds available
    pub donations_balance: U128, // TODO: may want to change this to U128?
    /// Cooldown period starts when Chef sets payouts
    pub cooldown_end_ms: Option<TimestampMs>,
    /// Have all projects been paid out?
    pub paid_out: bool,

    // APPLICATION MAPPINGS
    // /// All application records
    // pub applications_by_id: UnorderedMap<ApplicationId, Application>,
    // /// IDs of all applications
    // pub application_ids: UnorderedSet<ApplicationId>,
    // /// ID of applications by their `project_id`
    // pub application_id_by_project_id: LookupMap<ProjectId, ApplicationId>,
    pub applications_by_project_id: UnorderedMap<ProjectId, Application>,

    // DONATION MAPPINGS (end-user)
    /// All donation records
    pub donations_by_id: UnorderedMap<DonationId, Donation>,
    /// IDs of donations made to a given project
    pub donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of donations made by a given donor (user)
    pub donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,

    // MATCHING POOL DONATION MAPPINGS (patron)
    /// All matching pool donation records
    pub patron_donations_by_id: UnorderedMap<DonationId, PatronDonation>,
    /// IDs of matching pool donations
    pub patron_donation_ids: UnorderedSet<DonationId>,

    // PAYOUT MAPPINGS
    pub payouts_by_id: UnorderedMap<PayoutId, Payout>, // can iterate over this to get all payouts
    pub payout_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<PayoutId>>, // TODO: change to project_id
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    // ApplicationIds,
    // ApplicationsById,
    // ApplicationIdByProjectId,
    ApplicationsByProjectId,
    ApprovedApplicationIds,
    RejectedApplicationIds,
    PendingApplicationIds,
    DonationsById,
    DonationIdsByProjectId,
    DonationIdsByProjectIdInner { project_id: ProjectId },
    DonationIdsByDonorId,
    DonationIdsByDonorIdInner { donor_id: AccountId },
    PatronDonationsById,
    PatronDonationIds,
    PayoutsById,
    PayoutIdsByProjectId,
    PayoutIdsByProjectIdInner { project_id: ProjectId },
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
        round_start_ms: TimestampMs,
        round_end_ms: TimestampMs,
        application_start_ms: TimestampMs,
        application_end_ms: TimestampMs,
        max_projects: u32,
        base_currency: AccountId,
        created_by: AccountId,
        registry_contract_id: AccountId,
        pot_deployer_contract_id: AccountId,
        // milestone_threshold: U64,
        // basis_points_paid_upfront: u32,
        // application_requirement: Option<SBTRequirement>,
        donation_requirement: Option<SBTRequirement>,
        patron_referral_fee_basis_points: u32,
        max_patron_referral_fee: U128,
        chef_fee_basis_points: u32,
        protocol_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            chef_id,
            round_name,
            round_description,
            round_start_ms, // TODO: change to "public_round_start_ms" & update occurrences & related vars
            round_end_ms,
            application_start_ms,
            application_end_ms,
            max_projects,
            base_currency,
            created_by,
            registry_contract_id,
            pot_deployer_contract_id,
            // milestone_threshold,
            // basis_points_paid_upfront,
            // application_requirement,
            donation_requirement,
            patron_referral_fee_basis_points,
            max_patron_referral_fee,
            chef_fee_basis_points,
            protocol_fee_basis_points,
            protocol_fee_recipient_account,
            matching_pool_balance: U128::from(0),
            donations_balance: U128::from(0),
            cooldown_end_ms: None,
            paid_out: false,
            // application_ids: UnorderedSet::new(StorageKey::ApplicationIds),
            // applications_by_id: UnorderedMap::new(StorageKey::ApplicationsById),
            // application_id_by_project_id: LookupMap::new(StorageKey::ApplicationIdByProjectId),
            applications_by_project_id: UnorderedMap::new(StorageKey::ApplicationsByProjectId),
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            patron_donations_by_id: UnorderedMap::new(StorageKey::PatronDonationsById),
            patron_donation_ids: UnorderedSet::new(StorageKey::PatronDonationIds),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutIdsByProjectId),
            payouts_by_id: UnorderedMap::new(StorageKey::PayoutsById),
        }
    }

    pub fn is_round_active(&self) -> bool {
        let block_timestamp_ms = env::block_timestamp_ms();
        block_timestamp_ms >= self.round_start_ms && block_timestamp_ms < self.round_end_ms
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            chef_id: AccountId::new_unchecked("".to_string()),
            round_name: "".to_string(),
            round_description: "".to_string(),
            round_start_ms: 0,
            round_end_ms: 0,
            application_start_ms: 0,
            application_end_ms: 0,
            max_projects: 0,
            base_currency: AccountId::new_unchecked("".to_string()),
            created_by: AccountId::new_unchecked("".to_string()),
            registry_contract_id: AccountId::new_unchecked("".to_string()),
            pot_deployer_contract_id: AccountId::new_unchecked("".to_string()),
            // milestone_threshold: U64(0),
            // basis_points_paid_upfront: 0,
            // application_requirement: None,
            donation_requirement: None,
            patron_referral_fee_basis_points: 0,
            max_patron_referral_fee: U128(0),
            chef_fee_basis_points: 0,
            protocol_fee_basis_points: 0,
            protocol_fee_recipient_account: AccountId::new_unchecked("".to_string()),
            matching_pool_balance: U128::from(0),
            donations_balance: U128::from(0),
            cooldown_end_ms: None,
            paid_out: false,
            // application_ids: UnorderedSet::new(StorageKey::ApplicationIds),
            // applications_by_id: UnorderedMap::new(StorageKey::ApplicationsById),
            // application_id_by_project_id: LookupMap::new(StorageKey::ApplicationIdByProjectId),
            applications_by_project_id: UnorderedMap::new(StorageKey::ApplicationsByProjectId),
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            patron_donations_by_id: UnorderedMap::new(StorageKey::PatronDonationsById),
            patron_donation_ids: UnorderedSet::new(StorageKey::PatronDonationIds),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutIdsByProjectId),
            payouts_by_id: UnorderedMap::new(StorageKey::PayoutsById),
        }
    }
}
