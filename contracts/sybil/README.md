# PotLock Sybil Contract (V2)

## Purpose

1. Provides registry for sybil resistance providers (e.g. i-am-human, wormhole, others).
2. Allows users to collect stamps indicating their verification with registered providers.
3. Abstracts away individual sybil resistance providers/solutions to provide a single contract to call `is_human` (customizable parameters coming soon)

## Contract Structure

### General Types

```rs
pub type TimestampMs = u64;
```

### Contract

```rs
pub struct Contract {
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    providers_by_id: UnorderedMap<ProviderId, VersionedProvider>,
    pending_provider_ids: UnorderedSet<ProviderId>,
    active_provider_ids: UnorderedSet<ProviderId>,
    deactivated_provider_ids: UnorderedSet<ProviderId>,
    default_provider_ids: UnorderedSet<ProviderId>,
    default_human_threshold: u32,
    next_provider_id: ProviderId,
    next_stamp_id: StampId,
    // MAPPINGS
    // Stores all Stamp records, versioned for easy upgradeability
    stamps_by_id: UnorderedMap<StampId, VersionedStamp>,
    // Enables fetching of all stamps for a user
    // provider_ids_for_user: LookupMap<AccountId, UnorderedSet<ProviderId>>,
    stamp_ids_for_user: LookupMap<AccountId, UnorderedSet<StampId>>,
    // Enables fetching of all users with given stamp (provider ID)
    user_ids_for_provider: LookupMap<ProviderId, UnorderedSet<AccountId>>,
    // Enables fetching of providers that a user has submitted (e.g. if user has submitted one malicious provider, they are likely to submit more and you'll want to be able to fetch these or filter them out of results)
    provider_ids_for_submitter: LookupMap<AccountId, UnorderedSet<ProviderId>>,
    // Maps group name to Group struct
    groups_by_name: UnorderedMap<String, Group>,
    // Mapping of group name to provider IDs
    provider_ids_for_group: UnorderedMap<String, UnorderedSet<ProviderId>>,
    // Blacklisted accounts
    blacklisted_accounts: UnorderedSet<AccountId>,
}

/// Ephemeral-only
pub struct Config {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub default_provider_ids: Vec<ProviderId>,
    pub default_human_threshold: u32,
    pub pending_provider_count: u64,
    pub active_provider_count: u64,
    pub deactivated_provider_count: u64,
}
```

### Providers

```rs
pub type ProviderId = u64;

// Provider struct that is versioned & stored internally
pub struct Provider {
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: AccountId,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub provider_name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
    /// Admin notes, e.g. reason for flagging or marking inactive
    pub admin_notes: Option<String>,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    /// Custom gas amount required
    pub gas: Option<u64>,
    /// Optional tags
    pub tags: Option<Vec<String>>,
    /// Optional icon URL
    pub icon_url: Option<String>,
    /// Optional external URL
    pub external_url: Option<String>,
    /// User who submitted this provider
    pub submitted_by: AccountId,
    /// Timestamp of when this provider was submitted
    pub submitted_at_ms: TimestampMs,
    /// Total number of times this provider has been used successfully
    pub stamp_count: u64,
    /// Milliseconds that stamps from this provider are valid for before they expire
    pub stamp_validity_ms: Option<u64>,
    /// Name of account ID arg, e.g. `"account_id"` or `"accountId"` or `"account"`
    pub account_id_arg_name: String,
    /// Custom args as Base64VecU8
    pub custom_args: Option<Base64VecU8>,
}

pub enum ProviderStatus {
    Pending,
    Active,
    Deactivated,
}

// External-only/ephemeral Provider struct (not stored internally)
pub struct ProviderExternal {
    /// Provider ID
    pub id: ProviderId,
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: AccountId,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Account ID arg name
    pub account_id_arg_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub provider_name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
    /// Admin notes, e.g. reason for flagging or marking inactive
    pub admin_notes: Option<String>,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    /// Custom gas amount required
    pub gas: Option<u64>,
    /// Optional tags
    pub tags: Option<Vec<String>>,
    /// Optional icon URL
    pub icon_url: Option<String>,
    /// Optional external URL
    pub external_url: Option<String>,
    /// User who submitted this provider
    pub submitted_by: AccountId,
    /// Timestamp of when this provider was submitted
    pub submitted_at_ms: TimestampMs,
    /// Total number of times this provider has been used successfully
    pub stamp_count: u64,
    /// Milliseconds that stamps from this provider are valid for before they expire
    pub stamp_validity_ms: Option<u64>,
    /// Custom args as readable JSON
    pub custom_args: Option<JsonValue>, // This will hold the readable JSON
}
```

### Groups

```rs
pub enum Rule {
    Highest,                 // Take the highest score from the group
    Lowest,                  // Take the lowest score from the group
    Sum(Option<u32>),        // Sum all scores with optional max value
    DiminishingReturns(u32), // Sum with diminishing returns, factor in percentage (e.g., 10 for 10% reduction each)
    IncreasingReturns(u32), // Sum with increasing returns, factor in percentage (e.g., 10 for 10% increase each)
}

// Group record stored internally
pub struct Group {
    pub rule: Rule,
}

// Ephemeral struct used for view methods
pub struct GroupExternal {
    pub name: String,
    pub providers: Vec<ProviderId>,
    pub rule: Rule,
}
```

### Stamps

A **stamp** is the verification of a user against a given sybil provider.

```rs
pub type StampId = u64;

/// Stamp record that is stored on the contract
pub struct Stamp {
    pub user_id: AccountId,
    pub provider_id: ProviderId,
    pub validated_at_ms: TimestampMs,
}

/// Ephermal stamp data returned to user (not stored in contract)
pub struct StampExternal {
    pub id: StampId,
    pub user_id: AccountId,
    pub provider: ProviderExternal,
    pub validated_at_ms: TimestampMs,
}
```

### Constants & Input Validation

```rs
pub const PROVIDER_DEFAULT_WEIGHT: u32 = 100;
pub const MAX_PROVIDER_NAME_LENGTH: usize = 64;
pub const MAX_PROVIDER_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_PROVIDER_EXTERNAL_URL_LENGTH: usize = 256;
pub const MAX_PROVIDER_ICON_URL_LENGTH: usize = 256;
pub const MAX_TAGS_PER_PROVIDER: usize = 10;
pub const MAX_TAG_LENGTH: usize = 32;
pub const MAX_GAS: u64 = 100_000_000_000_000;
```

### Contract Source Metadata

_NB: Below implemented as per NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`_

```rs
pub struct ContractSourceMetadata {
    /// Version of source code, e.g. "v1.0.0", could correspond to Git tag
    pub version: String,
    /// Git commit hash of currently deployed contract code
    pub commit_hash: String,
    /// GitHub repo url for currently deployed contract code
    pub link: String,
}
```

## Methods

### Write Methods

**NB: ALL privileged write methods (those beginning with `admin_*` or `owner_*`) require an attached deposit of at least one yoctoNEAR, for security purposes.**

```rs
// INIT

pub fn new(
    source_metadata: Option<ContractSourceMetadata>,
    owner: AccountId,
    admins: Option<Vec<AccountId>>,
) -> Self


// PROVIDERS

#[payable]
pub fn register_provider(
    &mut self,
    contract_id: AccountId,
    method_name: String,
    account_id_arg_name: Option<String>, // defaults to "account_id" if None
    provider_name: String,
    description: Option<String>,
    gas: Option<u64>,
    tags: Option<Vec<String>>,
    icon_url: Option<String>,
    external_url: Option<String>,
    stamp_validity_ms: Option<u64>,
    custom_args: Option<Base64VecU8>,
    default_weight: Option<u32>, // owner/admin-only
) -> ProviderExternal // NB: anyone can call this method to register a provider.
// emits add_or_update_provider event

/// NB: this method can only be called by the provider's original submitter, or sybil contract owner/admin.
#[payable]
pub fn update_provider(
    &mut self,
    provider_id: ProviderId,
    // TODO: allow update of contract_id and method_name (should go back to pending status)
    account_id_arg_name: Option<String>,
    provider_name: Option<String>,
    description: Option<String>,
    gas: Option<u64>,
    tags: Option<Vec<String>>,
    icon_url: Option<String>,
    external_url: Option<String>,
    stamp_validity_ms: Option<u64>,
    custom_args: Option<Base64VecU8>,
    default_weight: Option<u32>,    // owner/admin-only
    status: Option<ProviderStatus>, // owner/admin-only
    admin_notes: Option<String>,    // owner/admin-only
) -> ProviderExternal
// emits add_or_update_provider event


// STAMPS

#[payable]
pub fn add_stamp(&mut self, provider_id: ProviderId) -> Option<StampExternal> // None response indicates that user is not verified on target provider
// emits add_stamp event

pub fn delete_stamp(&mut self, provider_id: ProviderId) -> ()
// emits delete_stamp event


// GROUPS

#[payable]
pub fn add_or_update_group(
    &mut self,
    group_name: String,
    providers: Vec<ProviderId>,
    rule: Rule,
) -> GroupExternal
// emits add_or_update_group event

#[payable]
pub fn delete_group(&mut self, group_name: String)
// emits delete_group event


// BLACKLIST

#[payable]
pub fn blacklist_accounts(&mut self, accounts: Vec<AccountId>, reason: Option<String>)
// emits blacklist_accounts event

#[payable]
pub fn unblacklist_accounts(&mut self, accounts: Vec<AccountId>)
// emits unblacklist_accounts event


// SOURCE METADATA

#[payable]
pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)
// emits set_source_metadata event


// OWNER/ADMINS

#[payable]
pub fn owner_change_owner(&mut self, new_owner: AccountId)
// emits transfer_owner event

#[payable]
pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>)
// emits update_admins_event

#[payable]
pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>)
// emits update_admins_event

#[payable]
pub fn admin_activate_provider(&mut self, provider_id: ProviderId) -> Provider
// emits update_provider_event

#[payable]
pub fn admin_deactivate_provider(&mut self, provider_id: ProviderId) -> Provider
// emits update_provider_event

pub fn admin_update_provider_status( // NB: this can also be done via update_provider method
    &mut self,
    provider_id: ProviderId,
    status: ProviderStatus,
) -> Provider
// emits update_provider_event

#[payable]
pub fn admin_set_default_providers(&mut self, provider_ids: Vec<ProviderId>)
// emits update_default_providers event

#[payable]
pub fn admin_add_default_providers(&mut self, provider_ids: Vec<ProviderId>)
// emits update_default_providers event

#[payable]
pub fn admin_remove_default_providers(&mut self, provider_ids: Vec<ProviderId>)
// emits update_default_providers event

#[payable]
pub fn admin_clear_default_providers(&mut self)
// emits update_default_providers event

#[payable]
pub fn admin_set_default_human_threshold(&mut self, default_human_threshold: u32)
// emits update_default_human_threshold event

```

### Read Methods

```rs
// CONFIG

pub fn get_config(&self) -> Config

// PROVIDERS
pub fn get_provider(&self, contract_id: String, method_name: String) -> Option<ProviderJson>

pub fn get_providers(
    &self,
    status: Option<ProviderStatus>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<ProviderExternal>


// STAMPS

pub fn get_stamps_for_account_id(
    &self,
    account_id: AccountId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<StampExternal>

pub fn get_users_for_stamp(
    &self,
    provider_id: ProviderId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<AccountId>

pub fn get_providers_submitted_by_user(
    &self,
    account_id: AccountId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<ProviderExternal>


// GROUPS

pub fn get_groups(&self) -> Vec<GroupExternal>

pub fn get_group(&self, group_name: String) -> Option<GroupExternal>


// BLACKLIST

pub fn get_blacklisted_accounts(&self) -> Vec<AccountId>


// IS-HUMAN

pub struct HumanScoreResponse {
    pub is_human: bool,
    pub score: u32,
}

pub fn get_human_score(&self, account_id: AccountId) -> HumanScoreResponse

pub fn is_human(&self, account_id: AccountId) -> bool // TODO: add option for caller to specify providers (with weights) + min_human_threshold


// OWNER/ADMINS

pub fn get_owner(&self) -> AccountId

pub fn get_admins(&self) -> Vec<AccountId>


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>
```

## Events

```rs
/// source metadata update
pub(crate) fn log_set_source_metadata_event(source_metadata: &ContractSourceMetadata) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "set_source_metadata",
                "data": [
                    {
                        "source_metadata": source_metadata,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// transfer owner
pub(crate) fn log_transfer_owner_event(new_owner: &AccountId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "transfer_owner",
                "data": [
                    {
                        "new_owner": new_owner,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update admins
pub(crate) fn log_update_admins_event(admins: &Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_admins",
                "data": [
                    {
                        "admins": admins,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add or update provider
pub(crate) fn log_add_or_update_provider_event(provider: &ProviderExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_or_update_provider",
                "data": [
                    {
                        "provider": provider,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add stamp
pub(crate) fn log_add_stamp_event(stamp_id: &StampId, stamp: &Stamp) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_stamp",
                "data": [
                    {
                        "stamp_id": stamp_id,
                        "stamp": stamp,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete stamp
pub(crate) fn log_delete_stamp_event(stamp_id: &StampId) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_stamp",
                "data": [
                    {
                        "stamp_id": stamp_id,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// add or update group
pub(crate) fn log_add_or_update_group_event(group: &GroupExternal) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "add_or_update_group",
                "data": [
                    {
                        "group": group,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// delete group
pub(crate) fn log_delete_group_event(group_name: &String) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "delete_group",
                "data": [
                    {
                        "group_name": group_name,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update default providers
pub(crate) fn log_update_default_providers_event(default_providers: Vec<ProviderId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_default_providers",
                "data": [
                    {
                        "default_providers": default_providers,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// update default human threshold
pub(crate) fn log_update_default_human_threshold_event(default_human_threshold: u32) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "update_default_human_threshold",
                "data": [
                    {
                        "default_human_threshold": default_human_threshold,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// blacklist account
pub(crate) fn log_blacklist_accounts_event(accounts: &Vec<AccountId>, reason: &Option<String>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "blacklist_account",
                "data": [
                    {
                        "accounts": accounts,
                        "reason": reason,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

/// unblacklist account
pub(crate) fn log_unblacklist_accounts_event(accounts: &Vec<AccountId>) {
    env::log_str(
        format!(
            "{}{}",
            EVENT_JSON_PREFIX,
            json!({
                "standard": "potlock",
                "version": "1.0.0",
                "event": "unblacklist_account",
                "data": [
                    {
                        "accounts": accounts,
                    }
                ]
            })
        )
        .as_ref(),
    );
}

```
