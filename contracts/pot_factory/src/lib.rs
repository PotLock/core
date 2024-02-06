use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, TreeMap, UnorderedSet};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json, serde_json::json, AccountId, Balance,
    BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError,
};

type TimestampMs = u64;

pub mod admin;
pub mod constants;
pub mod events;
pub mod internal;
pub mod pot;
pub mod source;
pub mod utils;
pub mod validation;
pub use crate::admin::*;
pub use crate::constants::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::pot::*;
pub use crate::source::*;
pub use crate::utils::*;
pub use crate::validation::*;

/// Pot Factory Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Contract superuser (should be a DAO, but no restrictions made at the contract level on this matter)
    owner: AccountId,
    /// Admins, which can be added/removed by the owner
    admins: UnorderedSet<AccountId>,
    /// Records of all Pots deployed by this Factory, indexed at their account ID, versioned for easy upgradeability
    pots_by_id: TreeMap<PotId, VersionedPot>,
    /// Config for protocol fees (% * 100)
    protocol_fee_basis_points: u32,
    /// Config for protocol fees recipient
    protocol_fee_recipient_account: AccountId,
    /// Default chef fee (% * 100)
    default_chef_fee_basis_points: u32,
    /// Accounts that are allowed to deploy Pots
    whitelisted_deployers: UnorderedSet<AccountId>,
    /// Specifies whether a Pot deployer is required to be whitelisted
    require_whitelist: bool,
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

/// Ephemeral-only (used in views)
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
    SourceMetadata,
    WhitelistedDeployers,
}

/// Ephemeral-only (used in views) - intended as the result type for Pots querying for protocol fees configuration
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProtocolConfig {
    pub basis_points: u32,
    pub account_id: AccountId,
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

pub const PROVIDER_ID_DELIMITER: &str = ":"; // separates contract_id and method_name in ProviderId

impl ProviderId {
    /// Generate ProviderId (`"{CONTRACT_ADDRESS}:{METHOD_NAME}"`) from contract_id and method_name
    fn new(contract_id: String, method_name: String) -> Self {
        ProviderId(format!(
            "{}{}{}",
            contract_id, PROVIDER_ID_DELIMITER, method_name
        ))
    }

    /// Decompose ProviderId into contract_id and method_name
    pub fn decompose(&self) -> (String, String) {
        let parts: Vec<&str> = self.0.split(PROVIDER_ID_DELIMITER).collect();
        if parts.len() != 2 {
            panic!("Invalid provider ID format. Expected 'contract_id:method_name'.");
        }
        (parts[0].to_string(), parts[1].to_string())
    }

    /// Validate (individual elements cannot be empty, cannot contain PROVIDER_ID_DELIMITER)
    pub fn validate(&self) {
        let (contract_id, method_name) = self.decompose();
        assert!(!contract_id.is_empty(), "Contract ID cannot be empty");
        assert!(!method_name.is_empty(), "Method name cannot be empty");
        assert!(
            !contract_id.contains(PROVIDER_ID_DELIMITER),
            "Contract ID cannot contain delimiter ('{}')",
            PROVIDER_ID_DELIMITER
        );
        assert!(
            !method_name.contains(PROVIDER_ID_DELIMITER),
            "Method name cannot contain delimiter ('{}')",
            PROVIDER_ID_DELIMITER
        );
    }
}

/// Weighting for a given CustomSybilCheck
type SybilProviderWeight = u32;

/// Ephemeral-only (used in custom_sybil_checks for setting on Pot deployment, but not stored in this contract; rather, stored in Pot contract)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CustomSybilCheck {
    contract_id: AccountId,
    method_name: String,
    weight: SybilProviderWeight,
}

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
        source_metadata: ContractSourceMetadata,
    ) -> Self {
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
            pots_by_id: TreeMap::new(StorageKey::PotsById),
            protocol_fee_basis_points,
            protocol_fee_recipient_account,
            default_chef_fee_basis_points,
            whitelisted_deployers: whitelisted_deployers_set,
            require_whitelist,
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
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

    /// Method intended for use by Pot contract querying for protocol fee configuration
    pub fn get_protocol_config(&self) -> ProtocolConfig {
        ProtocolConfig {
            basis_points: self.protocol_fee_basis_points,
            account_id: self.protocol_fee_recipient_account.clone(),
        }
    }
}

// impl Default for Contract {
//     fn default() -> Self {
//         Self {
//             owner: AccountId::new_unchecked("".to_string()),
//             admins: UnorderedSet::new(StorageKey::Admins),
//             pots_by_id: TreeMap::new(StorageKey::PotsById),
//             protocol_fee_basis_points: 0,
//             protocol_fee_recipient_account: AccountId::new_unchecked("".to_string()),
//             default_chef_fee_basis_points: 0,
//             whitelisted_deployers: UnorderedSet::new(StorageKey::WhitelistedDeployers),
//             require_whitelist: false,
//             contract_source_metadata: LazyOption::new(StorageKey::SourceMetadata, None),
//         }
//     }
// }
