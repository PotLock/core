use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, serde_json::json, AccountId, Balance,
    BorshStorageKey, Gas, Promise, PromiseError,
};
use std::collections::HashMap;

type TimestampMs = u64;
type ProjectId = AccountId;
type ApplicationId = ProjectId; // Applications are indexed by ProjectId
type DonationId = u64; // TODO: change to Sring formatted as `"application_id:donation_id"`

pub mod admin;
pub mod applications;
pub mod config;
pub mod constants;
pub mod donations;
pub mod internal;
pub mod payouts;
pub mod utils;
pub use crate::admin::*;
pub use crate::applications::*;
pub use crate::config::*;
pub use crate::constants::*;
pub use crate::donations::*;
pub use crate::internal::*;
pub use crate::payouts::*;
pub use crate::utils::*;

// TODO: move Provider stuff elsewhere?
#[derive(
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderId(pub String);

pub const PROVIDER_ID_DELIMITER: &str = ":"; // separates contract_id and method_name in ProviderId // TODO: move to constants.rs?

// Generate ProviderId ("{CONTRACT_ADDRESS}:{METHOD_NAME}") from contract_id and method_name
impl ProviderId {
    fn new(contract_id: String, method_name: String) -> Self {
        ProviderId(format!(
            "{}{}{}",
            contract_id, PROVIDER_ID_DELIMITER, method_name
        ))
    }

    pub fn decompose(&self) -> (String, String) {
        let parts: Vec<&str> = self.0.split(PROVIDER_ID_DELIMITER).collect();
        if parts.len() != 2 {
            panic!("Invalid provider ID format. Expected 'contract_id:method_name'.");
        }
        (parts[0].to_string(), parts[1].to_string())
    }
}

const MAX_PROTOCOL_FEE_BASIS_POINTS: u32 = 1000; // 10% max protocol fee

// #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
// #[serde(crate = "near_sdk::serde")]
// pub struct SybilProvider {
//     // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
//     /// Weight for this provider, e.g. 100
//     pub default_weight: u32,
//     // TODO: consider adding optional `gas`, `type`/`description` (e.g. "face scan", "twitter", "captcha", etc.)
// }

// #[derive(BorshSerialize, BorshDeserialize)]
// pub enum VersionedProvider {
//     Current(Provider),
// }

// impl From<VersionedProvider> for Provider {
//     fn from(provider: VersionedProvider) -> Self {
//         match provider {
//             VersionedProvider::Current(current) => current,
//         }
//     }
// }

// // TODO: move this elsewhere
// #[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
// #[serde(crate = "near_sdk::serde")]
// pub struct SybilConfig

/// Sybil provider weight
type SybilProviderWeight = u32;

// Ephemeral-only (used in custom_sybil_checks for setting and viewing)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CustomSybilCheck {
    contract_id: AccountId,
    method_name: String,
    weight: SybilProviderWeight,
}

// impl CustomSybilCheck {
//     pub fn to_stored(contract_id: AccountId, method_name: String, weight: SybilProviderWeight) -> Self {
//         Self {
//             contract_id,
//             method_name,
//             weight,
//         }
//     }
// }

/// Pot Contract (funding round)
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    // PERMISSIONED ACCOUNTS
    /// Owner of the contract
    pub owner: AccountId,
    /// Admins of the contract (Owner, which should in most cases be DAO, might want to delegate admin rights to other accounts)
    pub admins: UnorderedSet<AccountId>,
    /// Address (ID) of Pot manager ("chef"). This account is responsible for managing the Pot, e.g. reviewing applications, setting payouts, etc.
    /// Optional because it may be set after deployment.
    pub chef: LazyOption<AccountId>,

    // POT CONFIG
    /// User-facing name for this Pot
    pub pot_name: String,
    /// User-facing description for this Pot
    pub pot_description: String,
    /// Maximum number of projects that can be approved for the round. Considerations include gas limits for payouts, etc.
    pub max_projects: u32,
    /// Base currency for the round
    /// * NB: currently only `"near"` is supported
    pub base_currency: AccountId,
    /// MS Timestamp when applications can be submitted from
    pub application_start_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted until
    pub application_end_ms: TimestampMs,
    /// MS Timestamp when the public round starts
    pub public_round_start_ms: TimestampMs,
    /// MS Timestamp when the round ends
    pub public_round_end_ms: TimestampMs,
    /// Account ID that deployed this Pot contract (set at deployment, cannot be updated)
    pub deployed_by: AccountId,
    /// Contract ID + method name of registry provider that should be queried when projects apply to round. Method specified must receive "account_id" and return bool indicating registration status.
    /// * Optional because not all Pots will require registration, and those that do might set after deployment.
    pub registry_provider: LazyOption<ProviderId>,

    // SYBIL RESISTANCE
    /// Sybil contract address & method name that will be called to verify humanness. If `None`, no checks will be made.
    pub sybil_wrapper_provider: LazyOption<ProviderId>,
    /// Sybil checks (if using custom sybil config)
    pub custom_sybil_checks: LazyOption<HashMap<ProviderId, SybilProviderWeight>>,
    /// Minimum threshold score for Sybil checks (if using custom sybil config)
    pub custom_min_threshold_score: LazyOption<u32>,

    // FEES
    /// Basis points (1/100 of a percent) that should be paid to an account that refers a Patron (paid at the point when the matching pool donation comes in)
    pub patron_referral_fee_basis_points: u32,
    /// Basis points (1/100 of a percent) that should be paid to an account that refers a donor (paid at the point when the donation comes in)
    pub public_round_referral_fee_basis_points: u32,
    /// Chef's fee for managing the round. Gets taken out of each donation as they come in and are paid out
    pub chef_fee_basis_points: u32,
    // TODO: ADD MAX PROTOCOL FEE BASIS POINTS? or as const so it can't be updated without code deployment?

    // FUNDS & BALANCES
    /// Amount of matching funds available
    pub matching_pool_balance: U128,
    /// Total amount donated
    pub total_donations: U128,

    // PAYOUTS
    /// Cooldown period starts when Chef sets payouts
    pub cooldown_end_ms: LazyOption<TimestampMs>,
    /// Indicates whether all projects been paid out (this would be considered the "end-of-lifecycle" for the Pot)
    pub all_paid_out: bool,

    // MAPPINGS
    /// All application records
    pub applications_by_id: UnorderedMap<ApplicationId, VersionedApplication>,
    /// Approved application IDs
    pub approved_application_ids: UnorderedSet<ApplicationId>,
    /// All donation records
    pub donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
    /// IDs of public round donations (made by donors who are not Patrons, during public round)
    pub public_round_donation_ids: UnorderedSet<DonationId>,
    /// IDs of matching pool donations (made by Patrons)
    pub matching_pool_donation_ids: UnorderedSet<DonationId>,
    /// IDs of donations made to a given project
    pub donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of donations made by a given donor (user)
    pub donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    // payouts
    pub payouts_by_id: UnorderedMap<PayoutId, VersionedPayout>, // can iterate over this to get all payouts
    pub payout_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<PayoutId>>,

    // OTHER
    /// contract ID + method name of protocol config provider that should be queried for protocol fee basis points and protocol fee recipient account.
    /// Method specified must receive no requried args and return struct containing protocol_fee_basis_points and protocol_fee_recipient_account.
    /// Set by deployer and cannot be changed by Pot owner/admins.
    pub protocol_config_provider: LazyOption<ProviderId>,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Admins,
    Chef,
    RegistryProvider,
    SybilContractId,
    CustomSybilChecks,
    CustomMinThresholdScore,
    CooldownEndMs,
    ProtocolConfigProvider,
    ApplicationsById,
    ApprovedApplicationIds,
    DonationsById,
    PublicRoundDonationIds,
    MatchingPoolDonationIds,
    DonationIdsByProjectId,
    DonationIdsByProjectIdInner { project_id: ProjectId },
    DonationIdsByDonorId,
    DonationIdsByDonorIdInner { donor_id: AccountId },
    PayoutsById,
    PayoutIdsByProjectId,
    PayoutIdsByProjectIdInner { project_id: ProjectId },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        // permissioned accounts
        owner: Option<AccountId>, // defaults to signer account if not provided
        admins: Option<Vec<AccountId>>,
        chef: Option<AccountId>,

        // pot config
        pot_name: String,
        pot_description: String,
        max_projects: u32,
        application_start_ms: TimestampMs,
        application_end_ms: TimestampMs,
        public_round_start_ms: TimestampMs,
        public_round_end_ms: TimestampMs,
        registry_provider: Option<ProviderId>, // TODO: may need to change type here

        // sybil resistance
        sybil_wrapper_provider: Option<ProviderId>,
        custom_sybil_checks: Option<HashMap<ProviderId, SybilProviderWeight>>,
        custom_min_threshold_score: Option<u32>,

        // fees
        patron_referral_fee_basis_points: u32, // this could be optional with a default, but better to set explicitly for now
        public_round_referral_fee_basis_points: u32, // this could be optional with a default, but better to set explicitly for now
        chef_fee_basis_points: u32,

        // other
        protocol_config_provider: Option<ProviderId>,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            // permissioned accounts
            owner: owner.unwrap_or(env::signer_account_id()),
            admins: account_vec_to_set(
                if admins.is_some() {
                    admins.unwrap()
                } else {
                    vec![]
                },
                StorageKey::Admins,
            ),
            chef: LazyOption::new(StorageKey::Chef, chef.as_ref()),

            // pot config
            pot_name,
            pot_description,
            max_projects,
            base_currency: AccountId::new_unchecked("near".to_string()),
            application_start_ms,
            application_end_ms,
            public_round_start_ms,
            public_round_end_ms,
            deployed_by: env::signer_account_id(),
            registry_provider: LazyOption::new(
                StorageKey::RegistryProvider,
                registry_provider.as_ref(),
            ),

            // sybil resistance
            sybil_wrapper_provider: LazyOption::new(
                StorageKey::SybilContractId,
                sybil_wrapper_provider.as_ref(),
            ),
            custom_sybil_checks: LazyOption::new(
                StorageKey::CustomSybilChecks,
                custom_sybil_checks.as_ref(),
            ),
            custom_min_threshold_score: LazyOption::new(
                StorageKey::CustomMinThresholdScore,
                custom_min_threshold_score.as_ref(),
            ),

            // fees
            patron_referral_fee_basis_points,
            public_round_referral_fee_basis_points,
            chef_fee_basis_points,

            // funds and balances
            matching_pool_balance: U128(0),
            total_donations: U128(0),

            // payouts
            cooldown_end_ms: LazyOption::new(StorageKey::CooldownEndMs, None),
            all_paid_out: false,

            // mappings
            applications_by_id: UnorderedMap::new(StorageKey::ApplicationsById),
            approved_application_ids: UnorderedSet::new(StorageKey::ApprovedApplicationIds),
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            public_round_donation_ids: UnorderedSet::new(StorageKey::PublicRoundDonationIds),
            matching_pool_donation_ids: UnorderedSet::new(StorageKey::MatchingPoolDonationIds),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutIdsByProjectId),
            payouts_by_id: UnorderedMap::new(StorageKey::PayoutsById),

            // other
            protocol_config_provider: LazyOption::new(
                StorageKey::ProtocolConfigProvider,
                protocol_config_provider.as_ref(),
            ),
        }
    }

    pub fn is_round_active(&self) -> bool {
        let block_timestamp_ms = env::block_timestamp_ms();
        block_timestamp_ms >= self.public_round_start_ms
            && block_timestamp_ms < self.public_round_end_ms
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            chef: LazyOption::new(StorageKey::Chef, None),
            owner: env::signer_account_id(),
            admins: UnorderedSet::new(StorageKey::Admins),
            pot_name: "".to_string(),
            pot_description: "".to_string(),
            max_projects: 0,
            base_currency: AccountId::new_unchecked("near".to_string()),
            application_start_ms: 0,
            application_end_ms: 0,
            public_round_start_ms: 0,
            public_round_end_ms: 0,
            deployed_by: env::signer_account_id(),
            registry_provider: LazyOption::new(StorageKey::RegistryProvider, None),
            sybil_wrapper_provider: LazyOption::new(StorageKey::SybilContractId, None),
            custom_sybil_checks: LazyOption::new(StorageKey::CustomSybilChecks, None),
            custom_min_threshold_score: LazyOption::new(StorageKey::CustomMinThresholdScore, None),
            patron_referral_fee_basis_points: 0,
            public_round_referral_fee_basis_points: 0,
            chef_fee_basis_points: 0,
            matching_pool_balance: U128(0),
            total_donations: U128(0),
            cooldown_end_ms: LazyOption::new(StorageKey::CooldownEndMs, None),
            all_paid_out: false,
            applications_by_id: UnorderedMap::new(StorageKey::ApplicationsById),
            approved_application_ids: UnorderedSet::new(StorageKey::ApprovedApplicationIds),
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            public_round_donation_ids: UnorderedSet::new(StorageKey::PublicRoundDonationIds),
            matching_pool_donation_ids: UnorderedSet::new(StorageKey::MatchingPoolDonationIds),
            donation_ids_by_project_id: LookupMap::new(StorageKey::DonationIdsByProjectId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            payout_ids_by_project_id: LookupMap::new(StorageKey::PayoutIdsByProjectId),
            payouts_by_id: UnorderedMap::new(StorageKey::PayoutsById),
            protocol_config_provider: LazyOption::new(StorageKey::ProtocolConfigProvider, None),
        }
    }
}
