use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, require, serde_json::json, AccountId, Balance, BorshStorageKey, Gas,
    PanicOnDefault, Promise, PromiseError,
};
use std::collections::HashMap;

pub mod admins;
pub mod constants;
pub mod events;
pub mod internal;
pub mod lists;
pub mod refunds;
pub mod registrations;
pub mod source;
pub mod utils;
pub use crate::admins::*;
pub use crate::constants::*;
pub use crate::events::*;
pub use crate::internal::*;
pub use crate::lists::*;
pub use crate::refunds::*;
pub use crate::registrations::*;
pub use crate::source::*;
pub use crate::utils::*;

type RegistrantId = AccountId;
type RegistrationId = u64;
type ListId = u64;
type TimestampMs = u64;

/// CURRENT Lists Contract (v1.0.0)
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Incrementing ID to assign to new lists
    next_list_id: ListId,
    /// Incrementing ID to assign to new registrations
    next_registration_id: RegistrationId,
    /// Lists indexed by List ID
    lists_by_id: UnorderedMap<ListId, VersionedList>,
    /// Lookup from owner account ID to List IDs they own
    list_ids_by_owner: UnorderedMap<AccountId, UnorderedSet<ListId>>,
    /// Lookup from registrant ID to List IDs it belongs to
    list_ids_by_registrant: UnorderedMap<RegistrantId, UnorderedSet<ListId>>,
    /// List admins by List ID
    list_admins_by_list_id: LookupMap<ListId, UnorderedSet<AccountId>>,
    /// List registrants by ID
    // NB: list_id is stored on registration
    registrations_by_id: UnorderedMap<RegistrationId, VersionedRegistrationInternal>,
    /// Lookup from List ID to registration IDs
    registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    /// Lookup from Registrant ID to registration IDs
    registration_ids_by_registrant_id: UnorderedMap<RegistrantId, UnorderedSet<RegistrationId>>,
    /// Lookup from List ID to upvotes (account IDs)
    upvotes_by_list_id: LookupMap<ListId, UnorderedSet<AccountId>>,
    /// Lookup from Registrant ID to storage claims (to refund registrants if a list owner deletes a list or removes their registration)
    refund_claims_by_registrant_id: UnorderedMap<RegistrantId, Balance>,
    // // TODO: might want to add a lookup from list ID to registration IDs e.g. all_registrations_by_list_id, so don't have to iterate through all registrations sets & synthesize data
    // /// Pending registrations by List ID
    // pending_registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    // /// Approved registrations by List ID
    // approved_registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    // /// Rejected registration_ids by List ID
    // rejected_registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    // /// Graylisted registration_ids by List ID
    // graylisted_registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    // /// Blacklisted registration_ids by List ID
    // blacklisted_registration_ids_by_list_id: UnorderedMap<ListId, UnorderedSet<RegistrationId>>,
    /// Contract "source" metadata
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ListsById,
    ListIdsByOwner,
    ListIdsByOwnerInner { owner: AccountId },
    ListIdsByRegistrant,
    ListIdsByRegistrantInner { registrant: AccountId },
    ListAdminsByListId,
    ListAdminsByListIdInner { list_id: ListId },
    RegistrationsById,
    RegistrationIdsByListId,
    RegistrationIdsByListIdInner { list_id: ListId },
    RegistrationIdsByRegistrantId,
    RegistrationIdsByRegistrantIdInner { registrant_id: AccountId },
    UpvotesByListId,
    UpvotesByListIdInner { list_id: ListId },
    RefundClaimsByRegistrantId,
    // PendingRegistrantsByListId,
    // ApprovedRegistrantsByListId,
    // RejectedRegistrantsByListId,
    // GraylistedRegistrantsByListId,
    // BlacklistedRegistrantsByListId,
    SourceMetadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(source_metadata: ContractSourceMetadata) -> Self {
        Self {
            next_list_id: 1,
            next_registration_id: 1,
            lists_by_id: UnorderedMap::new(StorageKey::ListsById),
            list_ids_by_owner: UnorderedMap::new(StorageKey::ListIdsByOwner),
            list_ids_by_registrant: UnorderedMap::new(StorageKey::ListIdsByRegistrant),
            list_admins_by_list_id: LookupMap::new(StorageKey::ListAdminsByListId),
            registrations_by_id: UnorderedMap::new(StorageKey::RegistrationsById),
            registration_ids_by_list_id: UnorderedMap::new(StorageKey::RegistrationIdsByListId),
            registration_ids_by_registrant_id: UnorderedMap::new(
                StorageKey::RegistrationIdsByRegistrantId,
            ),
            upvotes_by_list_id: LookupMap::new(StorageKey::UpvotesByListId),
            refund_claims_by_registrant_id: UnorderedMap::new(
                StorageKey::RefundClaimsByRegistrantId,
            ),
            contract_source_metadata: LazyOption::new(
                StorageKey::SourceMetadata,
                Some(&VersionedContractSourceMetadata::Current(source_metadata)),
            ),
        }
    }
}
