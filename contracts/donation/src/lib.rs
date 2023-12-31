use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey,
    PanicOnDefault, Promise,
};

pub mod constants;
pub mod donations;
pub mod events;
pub mod internal;
pub mod owner;
pub mod source;
pub mod utils;
pub use crate::constants::*;
pub use crate::donations::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::source::*;
pub use crate::utils::*;

type DonationId = u64;
type TimestampMs = u64;

/// Registry Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    protocol_fee_basis_points: u32,
    referral_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
    donation_ids_by_recipient_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    donation_ids_by_ft_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
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

/// NOT stored in contract storage; only used for get_config response
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub owner: AccountId,
    pub protocol_fee_basis_points: u32,
    pub referral_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    DonationsById,
    DonationIdsByRecipientId,
    DonationIdsByRecipientIdInner { recipient_id: AccountId },
    DonationIdsByDonorId,
    DonationIdsByDonorIdInner { donor_id: AccountId },
    DonationIdsByFtId,
    DonationIdsByFtIdInner { ft_id: AccountId },
    SourceMetadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        protocol_fee_basis_points: u32,
        referral_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
        source_metadata: ContractSourceMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner,
            protocol_fee_basis_points,
            referral_fee_basis_points,
            protocol_fee_recipient_account,
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            donation_ids_by_recipient_id: LookupMap::new(StorageKey::DonationIdsByRecipientId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            donation_ids_by_ft_id: LookupMap::new(StorageKey::DonationIdsByFtId),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
        }
    }

    pub fn get_config(&self) -> Config {
        Config {
            owner: self.owner.clone(),
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            referral_fee_basis_points: self.referral_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
        }
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: AccountId::new_unchecked("".to_string()),
            protocol_fee_basis_points: 0,
            referral_fee_basis_points: 0,
            protocol_fee_recipient_account: AccountId::new_unchecked("".to_string()),
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            donation_ids_by_recipient_id: LookupMap::new(StorageKey::DonationIdsByRecipientId),
            donation_ids_by_donor_id: LookupMap::new(StorageKey::DonationIdsByDonorId),
            donation_ids_by_ft_id: LookupMap::new(StorageKey::DonationIdsByFtId),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(
                    ContractSourceMetadata {
                        version: "1.0.0".to_string(),
                        commit_hash: "12345".to_string(),
                        link: "www.example.com".to_string(),
                    },
                )),
            ),
        }
    }
}
