# PotLock Sybil Contract

## Purpose

1. Provides registry for sybil resistance providers (e.g. i-am-human, wormhole, others).
2. Allows users to collect stamps indicating their verification with registered providers.
3. Abstracts away individual sybil resistance providers/solutions to provide a single contract to call `is_human` (customizable parameters coming soon)

## Contract Structure

### General Types

```rs
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
    // MAPPINGS
    // Stores all Stamp records, versioned for easy upgradeability
    stamps_by_id: UnorderedMap<StampId, VersionedStamp>,
    // Enables fetching of all stamps for a user
    provider_ids_for_user: LookupMap<AccountId, UnorderedSet<ProviderId>>,
    // Enables fetching of all users with given stamp (provider ID)
    user_ids_for_provider: LookupMap<ProviderId, UnorderedSet<AccountId>>,
    // Enables fetching of providers that a user has submitted (e.g. if user has submitted one malicious provider, they are likely to submit more and you'll want to be able to fetch these or filter them out of results)
    provider_ids_for_submitter: LookupMap<AccountId, UnorderedSet<ProviderId>>,
}

/// Ephemeral-only
pub struct Config {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub default_provider_ids: Vec<ProviderId>,
    pub default_human_threshold: u32,
    pub pending_provider_count: u64, // may want to change these to U64 (string) to avoid JSON overflow, but this is highly unlikely. Easy to change later since this is ephemeral.
    pub active_provider_count: u64,
    pub deactivated_provider_count: u64,
}
```

### Providers

*NB: Providers are stored by their ID, which is a concatenation of the contract ID + method name, e.g. "iamhuman.near:is_human"*

```rs
type ProviderId = String; // NB: this is stored internally as a struct

// Provider struct that is versioned & stored internally
pub struct Provider {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
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
}

pub enum ProviderStatus {
    Pending,
    Active,
    Deactivated,
}

// External-only/ephemeral Provider struct (not stored internally) that contains contract_id and method_name
pub struct ProviderExternal {
    /// Provider ID
    pub provider_id: ProviderId,
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: String,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
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
}
```

### Stamps

A **stamp** is the verification of a user against a given sybil provider.

```rs
pub struct StampId(pub String); // "{USER_ID}#{PROVIDER_ID}"

const STAMP_ID_DELIMITER: &str = "#"; // separates user_id and provider_id in StampId. * NB: should not be the same as PROVIDER_ID_DELIMITER (currently set to ":")

impl StampId {
    // Generate StampId ("{USER_ID}#{PROVIDER_ID}") from user_id and provider_id
    fn new(user_id: AccountId, provider_id: ProviderId) -> Self {
        StampId(format!(
            "{}{}{}",
            user_id, STAMP_ID_DELIMITER, provider_id.0
        ))
    }
}

/// Ephermal stamp data returned to user (not stored in contract)
pub struct StampExternal {
    user_id: AccountId,
    provider: ProviderExternal,
    validated_at_ms: TimestampMs,
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
    contract_id: String,
    method_name: String,
    name: String,
    description: Option<String>,
    gas: Option<u64>,
    tags: Option<Vec<String>>,
    icon_url: Option<String>,
    external_url: Option<String>,
) -> ProviderExternal // NB: anyone can call this method to register a provider.

/// NB: this method can only be called by the provider's original submitter, or sybil contract owner/admin.
#[payable]
pub fn update_provider(
    &mut self,
    provider_id: ProviderId,
    name: Option<String>,
    description: Option<String>,
    gas: Option<u64>,
    tags: Option<Vec<String>>,
    icon_url: Option<String>,
    external_url: Option<String>,
    default_weight: Option<u32>,    // owner/admin-only
    status: Option<ProviderStatus>, // owner/admin-only
    admin_notes: Option<String>,    // owner/admin-only
) -> ProviderExternal

// STAMPS

#[payable]
pub fn add_stamp(&mut self, provider_id: ProviderId) -> Option<StampExternal> // None response indicates that user is not verified on target provider

pub fn delete_stamp(&mut self, provider_id: ProviderId) -> ()


// SOURCE METADATA

#[payable]
pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)


// OWNER/ADMINS

#[payable]
pub fn owner_change_owner(&mut self, new_owner: AccountId)

#[payable]
pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>)

#[payable]
pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>)

#[payable]
pub fn admin_activate_provider(&mut self, provider_id: ProviderId) -> Provider

#[payable]
pub fn admin_deactivate_provider(&mut self, provider_id: ProviderId) -> Provider

pub fn admin_update_provider_status( // NB: this can also be done via update_provider method
    &mut self,
    provider_id: ProviderId,
    status: ProviderStatus,
) -> Provider

#[payable]
pub fn admin_set_default_providers(&mut self, provider_ids: Vec<ProviderId>)

#[payable]
pub fn admin_add_default_providers(&mut self, provider_ids: Vec<ProviderId>)

#[payable]
pub fn admin_remove_default_providers(&mut self, provider_ids: Vec<ProviderId>)

#[payable]
pub fn admin_clear_default_providers(&mut self)

#[payable]
pub fn admin_set_default_human_threshold(&mut self, default_human_threshold: u32)

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

### `set_source_metadata`

Indicates that `ContractSourceMetadata` object has been set/updated.

**Example:**

```json
{
    "standard": "potlock",
    "version": "1.0.0",
    "event": "set_source_metadata",
    "data": [
        {
            "source_metadata": {
                "commit_hash":"ec02294253b22c2d4c50a75331df23ada9eb04db",
                "link":"https://github.com/PotLock/core",
                "version":"0.1.0",
            }
        }
    ]
}
```

### `add_provider`

Indicates that a new provider has been added.

**Example:**

```json
{
    "standard": "potlock",
    "version": "1.0.0",
    "event": "add_provider",
    "data": [
        {
            "provider_id": "provider.near:is_human",
            "provider": {
                "name": "Provider Name",
                "description": "Description of the provider",
                "tags": ["face-scan", "twitter"],
                "icon_url": "https://google.com/myimage.png",
                "external_url": "https://provider.example.com",
                "submitted_by": "user.near",
                "submitted_at_ms": 1706289760834,
                "stamp_count": 0,
                "status": "Pending",
                "default_weight": 100,
                "admin_notes": null,
            }
        }
    ]
}
```

### `update_provider`

Indicates that an existing provider has been updated.

**Example:**

```json
{
    "standard": "potlock",
    "version": "1.0.0",
    "event": "update_provider",
    "data": [
        {
            "provider_id": "provider.near:is_human",
            "provider": {
                "name": "Provider Name",
                "description": "Description of the provider",
                "tags": ["face-scan", "twitter"],
                "icon_url": "https://google.com/myimage.png",
                "external_url": "https://provider.example.com",
                "submitted_by": "user.near",
                "submitted_at_ms": 1706289760834,
                "stamp_count": 0,
                "status": "Active",
                "default_weight": 20,
                "admin_notes": null,
            }
        }
    ]
}
```