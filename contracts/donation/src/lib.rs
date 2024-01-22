use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseError,
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
    next_donation_id: DonationId,
    storage_deposits: UnorderedMap<AccountId, Balance>,
}

// /// OLD - Registry Contract V1
// #[near_bindgen]
// #[derive(BorshDeserialize, BorshSerialize)]
// pub struct ContractV1 {
//     /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
//     contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
//     owner: AccountId,
//     protocol_fee_basis_points: u32,
//     referral_fee_basis_points: u32,
//     protocol_fee_recipient_account: AccountId,
//     donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
//     donation_ids_by_recipient_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
//     donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
//     donation_ids_by_ft_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
//     storage_deposits: UnorderedMap<AccountId, Balance>,
// }

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
        }
    }

    // #[private]
    // #[init(ignore_state)]
    // pub fn migrate_to_v2() -> Self {
    //     let old_state = env::state_read::<ContractV1>().expect("Old state doesn't exist");
    //     let next_donation_id = old_state.donations_by_id.len() as u64 + 1;
    //     Self {
    //         owner: old_state.owner,
    //         protocol_fee_basis_points: old_state.protocol_fee_basis_points,
    //         referral_fee_basis_points: old_state.referral_fee_basis_points,
    //         protocol_fee_recipient_account: old_state.protocol_fee_recipient_account,
    //         donations_by_id: old_state.donations_by_id,
    //         donation_ids_by_recipient_id: old_state.donation_ids_by_recipient_id,
    //         donation_ids_by_donor_id: old_state.donation_ids_by_donor_id,
    //         donation_ids_by_ft_id: old_state.donation_ids_by_ft_id,
    //         contract_source_metadata: old_state.contract_source_metadata,
    //         next_donation_id,
    //         storage_deposits: old_state.storage_deposits,
    //     }
    // }
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
            next_donation_id: 1,
            storage_deposits: UnorderedMap::new(StorageKey::StorageDeposits),
        }
    }
}
