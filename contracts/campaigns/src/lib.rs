use near_sdk::json_types::U128;
use near_sdk::store::{IterableMap, IterableSet, LazyOption};
use near_sdk::{
    env, log, near, require, serde_json::json, AccountId, BorshStorageKey, Gas, NearToken,
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

pub type Balance = u128;

/// CURRENT Campaigns Contract
#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: IterableSet<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_referral_fee_basis_points: u32,
    default_creator_fee_basis_points: u32,
    // TODO: add batch_size rather than storing as constant, so that we can adjust dynamically after contract is locked
    next_campaign_id: CampaignId,
    campaigns_by_id: IterableMap<CampaignId, VersionedCampaign>,
    campaign_ids_by_owner: IterableMap<AccountId, IterableSet<CampaignId>>,
    campaign_ids_by_recipient: IterableMap<AccountId, IterableSet<CampaignId>>,
    next_donation_id: DonationId,
    donations_by_id: IterableMap<DonationId, VersionedDonation>,
    escrowed_donation_ids_by_campaign_id: IterableMap<CampaignId, IterableSet<DonationId>>,
    unescrowed_donation_ids_by_campaign_id: IterableMap<CampaignId, IterableSet<DonationId>>,
    returned_donation_ids_by_campaign_id: IterableMap<CampaignId, IterableSet<DonationId>>,
    donation_ids_by_donor_id: IterableMap<AccountId, IterableSet<DonationId>>,
    storage_deposits: IterableMap<AccountId, Balance>, // Add storage_deposits to track storage deposits for FTs
}

/// NOT stored in contract storage; only used for get_config response
#[near(serializers = [json, borsh])]
#[derive(Clone)]
pub struct Config {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub protocol_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
    pub default_referral_fee_basis_points: u32,
    pub default_creator_fee_basis_points: u32,
    pub total_campaigns_count: u32,
    pub total_donations_count: u32,
}

#[near(serializers = [borsh])]
#[derive(BorshStorageKey)]
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

#[near]
impl Contract {
    /// For testing purposes only
    #[init]
    pub fn new_default_meta(owner: AccountId) -> Self {
        Self::new(
            owner.clone(),
            100,
            owner.clone(),
            100,
            100,
            ContractSourceMetadata {
                version: "0.1.0".to_string(),
                commit_hash: "".to_string(),
                link: "".to_string(),
            },
        )
    }

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
            admins: IterableSet::new(StorageKey::Admins),
            protocol_fee_basis_points,
            protocol_fee_recipient_account,
            default_referral_fee_basis_points,
            default_creator_fee_basis_points,
            next_campaign_id: 1,
            campaigns_by_id: IterableMap::new(StorageKey::CampaignsById),
            campaign_ids_by_owner: IterableMap::new(StorageKey::CampaignIdsByOwner),
            campaign_ids_by_recipient: IterableMap::new(StorageKey::CampaignIdsByRecipient),
            next_donation_id: 1,
            donations_by_id: IterableMap::new(StorageKey::DonationsById),
            // donation_ids_by_campaign_id: IterableMap::new(StorageKey::DonationIdsByCampaignId),
            escrowed_donation_ids_by_campaign_id: IterableMap::new(
                StorageKey::EscrowedDonationIdsByCampaignId,
            ),
            unescrowed_donation_ids_by_campaign_id: IterableMap::new(
                StorageKey::UnescrowedDonationIdsByCampaignId,
            ),
            returned_donation_ids_by_campaign_id: IterableMap::new(
                StorageKey::ReturnedDonationIdsByCampaignId,
            ),
            donation_ids_by_donor_id: IterableMap::new(StorageKey::DonationIdsByDonorId),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(VersionedContractSourceMetadata::Current(source_metadata)),
            ),
            storage_deposits: IterableMap::new(StorageKey::StorageDeposits),
        }
    }

    pub fn get_config(&self) -> Config {
        let nm = self.admins.iter().cloned().collect();
        Config {
            owner: self.owner.clone(),
            admins: nm,
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            protocol_fee_recipient_account: self.protocol_fee_recipient_account.clone(),
            default_referral_fee_basis_points: self.default_referral_fee_basis_points,
            default_creator_fee_basis_points: self.default_creator_fee_basis_points,
            total_campaigns_count: self.campaigns_by_id.len(),
            total_donations_count: self.donations_by_id.len(),
        }
    }
}
