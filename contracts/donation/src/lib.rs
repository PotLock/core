use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};

pub mod constants;
pub mod donations;
pub mod events;
pub mod internal;
pub mod owner;
pub mod source;
pub mod storage;
pub mod utils;
pub use crate::constants::*;
pub use crate::donations::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::source::*;
pub use crate::storage::*;
pub use crate::utils::*;

type DonationId = u64;
type TimestampMs = u64;

/// DEPRECATED (V1) Donation Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractV1 {
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

/// DEPRECATED (V2) Donation Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ContractV2 {
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
    total_donations_amount: Balance, // Add total_donations_amount to track total donations amount without iterating through all donations
    net_donations_amount: Balance, // Add net_donations_amount to track net donations amount (after fees) without iterating through all donations
    total_protocol_fees: Balance, // Add total_protocol_fees to track total protocol fees without iterating through all donations
    total_referrer_fees: Balance, // Add total_referrer_fees to track total referral fees without iterating through all donations
}

/// CURRENT Donation Contract
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
    total_donations_amount: Balance,
    net_donations_amount: Balance,
    total_protocol_fees: Balance,
    total_referrer_fees: Balance,
    next_donation_id: DonationId, // Add next_donation_id to track next donation id and handle failed donations without accidental overwrites
    storage_deposits: UnorderedMap<AccountId, Balance>, // Add storage_deposits to track storage deposits for FTs
}
// #[derive(BorshSerialize, BorshDeserialize)]
// pub enum VersionedContract {
//     Current(Contract),
//     V1(ContractV1),
// }

// /// Convert VersionedContract to Contract
// impl From<VersionedContract> for Contract {
//     fn from(contract: VersionedContract) -> Self {
//         match contract {
//             VersionedContract::Current(current) => current,
//             VersionedContract::V1(v1) => Contract {
//                 contract_source_metadata: v1.contract_source_metadata,
//                 owner: v1.owner,
//                 protocol_fee_basis_points: v1.protocol_fee_basis_points,
//                 referral_fee_basis_points: v1.referral_fee_basis_points,
//                 protocol_fee_recipient_account: v1.protocol_fee_recipient_account,
//                 donations_by_id: v1.donations_by_id,
//                 donation_ids_by_recipient_id: v1.donation_ids_by_recipient_id,
//                 donation_ids_by_donor_id: v1.donation_ids_by_donor_id,
//                 donation_ids_by_ft_id: v1.donation_ids_by_ft_id,
//                 next_donation_id: 0,
//                 storage_deposits: v1.storage_deposits,
//             },
//         }
//     }
// }

/// NOT stored in contract storage; only used for get_config response
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub owner: AccountId,
    pub protocol_fee_basis_points: u32,
    pub referral_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
    pub total_donations_amount: U128,
    pub net_donations_amount: U128,
    pub total_donations_count: U64,
    pub total_protocol_fees: U128,
    pub total_referrer_fees: U128,
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
    StorageDeposits,
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
            total_donations_amount: 0,
            net_donations_amount: 0,
            total_protocol_fees: 0,
            total_referrer_fees: 0,
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
            next_donation_id: 1,
            storage_deposits: UnorderedMap::new(StorageKey::StorageDeposits),
        }
    }

    pub fn get_config(&self) -> Config {
        Config {
            owner: self.owner.clone(),
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            referral_fee_basis_points: self.referral_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
            total_donations_amount: self.total_donations_amount.into(),
            net_donations_amount: self.net_donations_amount.into(),
            total_donations_count: self.donations_by_id.len().into(),
            total_protocol_fees: self.total_protocol_fees.into(),
            total_referrer_fees: self.total_referrer_fees.into(),
        }
    }

    // LEAVING FOR REFERENCE - this is function used to migrate data in upgrade from v1.0.0 to v2.0.0
    // #[private]
    // pub fn migrate_chunk_temp(&mut self, donation_ids: Vec<DonationId>) {
    //     for donation_id in donation_ids {
    //         log!("Migrating donation {}", donation_id);
    //         let donation = Donation::from(self
    //             .donations_by_id
    //             .get(&donation_id)
    //             .expect(format!("Donation {} not found", donation_id).as_str()));
    //         self.total_donations_amount += donation.total_amount.0;
    //         let mut net_amount = donation.total_amount.0 - donation.protocol_fee.0;
    //         self.total_protocol_fees += donation.protocol_fee.0;
    //         if let Some(referral_fee) = donation.referrer_fee {
    //             net_amount -= referral_fee.0;
    //             self.total_referrer_fees += referral_fee.0;
    //         }
    //         self.net_donations_amount += net_amount;
    //     }
    // }

    // LEAVING FOR REFERENCE - this is the initFunction used in upgrade from v1.0.0 to v2.0.0
    // #[private]
    // #[init(ignore_state)]
    // pub fn migrate() -> Self {
    //     let old_state: ContractV1 = env::state_read().expect("state read failed");
    //     Self {
    //         owner: old_state.owner,
    //         protocol_fee_basis_points: old_state.protocol_fee_basis_points,
    //         referral_fee_basis_points: old_state.referral_fee_basis_points,
    //         protocol_fee_recipient_account: old_state.protocol_fee_recipient_account,
    //         donations_by_id: old_state.donations_by_id,
    //         donation_ids_by_recipient_id: old_state.donation_ids_by_recipient_id,
    //         donation_ids_by_donor_id: old_state.donation_ids_by_donor_id,
    //         donation_ids_by_ft_id: old_state.donation_ids_by_ft_id,
    //         total_donations_amount: 0,
    //         net_donations_amount: 0,
    //         total_protocol_fees: 0,
    //         total_referrer_fees: 0,
    //         contract_source_metadata: old_state.contract_source_metadata,
    //     }
    // }

    // LEAVING FOR REFERENCE - this is the initFunction used in upgrade from v2.0.0 to v3.0.0
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let old_state: ContractV2 = env::state_read().expect("state read failed");
        let num_donations = old_state.donations_by_id.len();
        Self {
            owner: old_state.owner,
            protocol_fee_basis_points: old_state.protocol_fee_basis_points,
            referral_fee_basis_points: old_state.referral_fee_basis_points,
            protocol_fee_recipient_account: old_state.protocol_fee_recipient_account,
            donations_by_id: old_state.donations_by_id,
            donation_ids_by_recipient_id: old_state.donation_ids_by_recipient_id,
            donation_ids_by_donor_id: old_state.donation_ids_by_donor_id,
            donation_ids_by_ft_id: old_state.donation_ids_by_ft_id,
            total_donations_amount: old_state.total_donations_amount,
            net_donations_amount: old_state.net_donations_amount,
            total_protocol_fees: old_state.total_protocol_fees,
            total_referrer_fees: old_state.total_referrer_fees,
            contract_source_metadata: old_state.contract_source_metadata,
            next_donation_id: num_donations as u64 + 1,
            storage_deposits: UnorderedMap::new(StorageKey::StorageDeposits),
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
            total_donations_amount: 0,
            net_donations_amount: 0,
            total_protocol_fees: 0,
            total_referrer_fees: 0,
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
            next_donation_id: 1,
            storage_deposits: UnorderedMap::new(StorageKey::StorageDeposits),
        }
    }
}
