use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseError, PromiseOrValue,
};
use std::collections::HashMap;

pub mod admin;
pub mod campaigns;
pub mod constants;
pub mod donations;
pub mod escrow;
pub mod events;
pub mod internal;
pub mod owner;
pub mod source;
pub mod storage;
pub mod transfer;
pub mod utils;
pub mod validation;
pub use crate::admin::*;
pub use crate::campaigns::*;
pub use crate::constants::*;
pub use crate::donations::*;
pub use crate::escrow::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::owner::*;
pub use crate::source::*;
pub use crate::storage::*;
pub use crate::transfer::*;
pub use crate::utils::*;
pub use crate::validation::*;

type DonationId = u64;
type TimestampMs = u64;

/// CURRENT Campaigns Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_referral_fee_basis_points: u32,
    default_creator_fee_basis_points: u32,
    // TODO: add batch_size rather than storing as constant, so that we can adjust dynamically after contract is locked
    next_campaign_id: CampaignId,
    campaigns_by_id: UnorderedMap<CampaignId, VersionedCampaign>,
    campaign_ids_by_owner: UnorderedMap<AccountId, UnorderedSet<CampaignId>>,
    campaign_ids_by_recipient: UnorderedMap<AccountId, UnorderedSet<CampaignId>>,
    next_donation_id: DonationId,
    donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
    escrowed_donation_ids_by_campaign_id: UnorderedMap<CampaignId, UnorderedSet<DonationId>>,
    unescrowed_donation_ids_by_campaign_id: UnorderedMap<CampaignId, UnorderedSet<DonationId>>,
    returned_donation_ids_by_campaign_id: UnorderedMap<CampaignId, UnorderedSet<DonationId>>,
    donation_ids_by_donor_id: UnorderedMap<AccountId, UnorderedSet<DonationId>>,
    storage_deposits: UnorderedMap<AccountId, Balance>, // Add storage_deposits to track storage deposits for FTs
}

/// NOT stored in contract storage; only used for get_config response
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub protocol_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
    pub default_referral_fee_basis_points: u32,
    pub default_creator_fee_basis_points: u32,
    pub total_campaigns_count: u64,
    pub total_donations_count: u64,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Admins,
    CampaignsById,
    CampaignIdsByOwner,
    CampaignIdsByOwnerInner { owner_id: AccountId },
    CampaignIdsByRecipient,
    CampaignIdsByRecipientInner { recipient_id: AccountId },
    DonationsById,
    EscrowedDonationIdsByCampaignId,
    EscrowedDonationIdsByCampaignIdInner { campaign_id: CampaignId },
    UnescrowedDonationIdsByCampaignId,
    UnescrowedDonationIdsByCampaignIdInner { campaign_id: CampaignId },
    ReturnedDonationIdsByCampaignId,
    ReturnedDonationIdsByCampaignIdInner { campaign_id: CampaignId },
    DonationIdsByDonorId,
    DonationIdsByDonorIdInner { donor_id: AccountId },
    SourceMetadata,
    StorageDeposits,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId,
        protocol_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
        default_referral_fee_basis_points: u32,
        default_creator_fee_basis_points: u32,
        source_metadata: ContractSourceMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner,
            admins: UnorderedSet::new(StorageKey::Admins),
            protocol_fee_basis_points,
            protocol_fee_recipient_account,
            default_referral_fee_basis_points,
            default_creator_fee_basis_points,
            next_campaign_id: 1,
            campaigns_by_id: UnorderedMap::new(StorageKey::CampaignsById),
            campaign_ids_by_owner: UnorderedMap::new(StorageKey::CampaignIdsByOwner),
            campaign_ids_by_recipient: UnorderedMap::new(StorageKey::CampaignIdsByRecipient),
            next_donation_id: 1,
            donations_by_id: UnorderedMap::new(StorageKey::DonationsById),
            // donation_ids_by_campaign_id: UnorderedMap::new(StorageKey::DonationIdsByCampaignId),
            escrowed_donation_ids_by_campaign_id: UnorderedMap::new(
                StorageKey::EscrowedDonationIdsByCampaignId,
            ),
            unescrowed_donation_ids_by_campaign_id: UnorderedMap::new(
                StorageKey::UnescrowedDonationIdsByCampaignId,
            ),
            returned_donation_ids_by_campaign_id: UnorderedMap::new(
                StorageKey::ReturnedDonationIdsByCampaignId,
            ),
            donation_ids_by_donor_id: UnorderedMap::new(StorageKey::DonationIdsByDonorId),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
            storage_deposits: UnorderedMap::new(StorageKey::StorageDeposits),
        }
    }

    pub fn get_config(&self) -> Config {
        Config {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
            default_referral_fee_basis_points: self.default_referral_fee_basis_points,
            default_creator_fee_basis_points: self.default_creator_fee_basis_points,
            total_campaigns_count: self.campaigns_by_id.len(),
            total_donations_count: self.donations_by_id.len(),
        }
    }
}
