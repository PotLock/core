use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey,
    Gas, Promise, PromiseError,
};

type TimestampMs = u64;

pub mod admin;
pub mod constants;
pub mod internal;
pub mod pot;
pub mod utils;
pub use crate::admin::*;
pub use crate::constants::*;
pub use crate::internal::*;
pub use crate::pot::*;
pub use crate::utils::*;

pub const TGAS: u64 = 1_000_000_000_000; // 1 TGAS
pub const XCC_GAS: Gas = Gas(TGAS * 5);
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

/// Pot Factory Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Contract superuser
    owner: AccountId,
    /// Admins, which can be added/removed by the owner
    admins: UnorderedSet<AccountId>,
    /// All Pot records
    pots_by_id: UnorderedMap<PotId, Pot>,
    /// Config for protocol fees (% * 100)
    protocol_fee_basis_points: u32,
    /// Config for protocol fee recipient
    protocol_fee_recipient_account: AccountId,
    /// Default chef fee (% * 100)
    default_chef_fee_basis_points: u32,
    /// Accounts that are allowed to deploy pots
    whitelisted_deployers: UnorderedSet<AccountId>,
    /// Specifies whether a pot deployer is required to be whitelisted
    require_whitelist: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractConfigExternal {
    owner: AccountId,
    admins: Vec<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_chef_fee_basis_points: u32,
    whitelisted_deployers: Vec<AccountId>,
    require_whitelist: bool,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Admins,
    PotsById,
    WhitelistedDeployers,
}

pub struct ProtocolConfig {
    pub protocol_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
}

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

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotArgs {
    owner: Option<AccountId>,
    admins: Option<Vec<AccountId>>,
    chef: AccountId,
    pot_name: String,
    pot_description: String,
    max_projects: u32,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    public_round_start_ms: TimestampMs,
    public_round_end_ms: TimestampMs,
    registry_provider: Option<ProviderId>,
    sybil_wrapper_provider: Option<ProviderId>,
    custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    custom_min_threshold_score: Option<u32>,
    patron_referral_fee_basis_points: u32,
    public_round_referral_fee_basis_points: u32,
    chef_fee_basis_points: u32,
}

// /// `PotArgs` + `deployed_by`
// #[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
// #[serde(crate = "near_sdk::serde")]
// pub struct PotArgsInternal {
//     chef: AccountId,
//     pot_name: String,
//     pot_description: String,
//     public_round_start_ms: TimestampMs,
//     public_round_end_ms: TimestampMs,
//     application_start_ms: TimestampMs,
//     application_end_ms: TimestampMs,
//     max_projects: u32,
//     base_currency: AccountId,
//     deployed_by: AccountId,
//     // milestone_threshold: U64,
//     // basis_points_paid_upfront: u32,
//     donation_requirement: Option<SBTRequirement>,
//     referral_fee_basis_points: u32,
//     chef_fee_basis_points: u32,
//     protocol_fee_basis_points: u32,
// }

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        admins: Vec<AccountId>,
        protocol_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
        default_chef_fee_basis_points: u32,
        whitelisted_deployers: Vec<AccountId>,
        require_whitelist: bool,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut admins_set = UnorderedSet::new(StorageKey::Admins);
        for admin in admins.iter() {
            admins_set.insert(admin);
        }
        let mut whitelisted_deployers_set = UnorderedSet::new(StorageKey::WhitelistedDeployers);
        for whitelisted_deployer in whitelisted_deployers.iter() {
            whitelisted_deployers_set.insert(whitelisted_deployer);
        }
        Self {
            owner,
            admins: admins_set,
            pots_by_id: UnorderedMap::new(StorageKey::PotsById),
            protocol_fee_basis_points,
            protocol_fee_recipient_account,
            default_chef_fee_basis_points,
            whitelisted_deployers: whitelisted_deployers_set,
            require_whitelist,
        }
    }

    pub fn get_config(&self) -> ContractConfigExternal {
        ContractConfigExternal {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
            default_chef_fee_basis_points: self.default_chef_fee_basis_points,
            whitelisted_deployers: self.whitelisted_deployers.to_vec(),
            require_whitelist: self.require_whitelist,
        }
    }

    pub fn get_protocol_config(&self) -> ProtocolConfig {
        ProtocolConfig {
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
        }
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: AccountId::new_unchecked("".to_string()),
            admins: UnorderedSet::new(StorageKey::Admins),
            pots_by_id: UnorderedMap::new(StorageKey::PotsById),
            protocol_fee_basis_points: 0,
            protocol_fee_recipient_account: AccountId::new_unchecked("".to_string()),
            default_chef_fee_basis_points: 0,
            whitelisted_deployers: UnorderedSet::new(StorageKey::WhitelistedDeployers),
            require_whitelist: false,
        }
    }
}
