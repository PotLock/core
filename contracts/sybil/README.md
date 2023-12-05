# PotLock Sybil Contract

## Purpose

Abstracts away individual sybil resistance providers/solutions to provide a single contract to call.

Provides registry for sybil resistance providers (e.g. i-am-human, wormhole, others).

Provides simple `is_human`` check for provided account ID based on default config (weights + threshold).

Allows weight & threshold overrides to be passed in to is_human call. **(TBC)**

Provides methods to be able to get all providers, single provider, or add/set providers (for owner/admins)

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
    default_provider_ids: UnorderedSet<ProviderId>,
    default_human_threshold: u32,
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
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    // TODO: consider adding optional `gas`, `type`/`description` (e.g. "face scan", "twitter", "captcha", etc.)
}

// External-only/ephemeral Provider struct (not stored internally) that contains contract_id and method_name
pub struct ProviderJson {
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: String,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
}
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

### Write Methods (must be owner or admin)

```rs
// PROVIDERS

#[payable]
pub fn add_provider(
    &mut self,
    contract_id: String,
    method_name: String,
    name: String,
    default_weight: u32,
) -> ProviderId

#[payable]
pub fn set_default_providers(&mut self, provider_ids: Vec<ProviderId>)

#[payable]
pub fn add_default_provider(&mut self, provider_id: ProviderId)

// DEFAULT HUMAN THRESHOLD

#[payable]
pub fn set_default_human_threshold(&mut self, default_human_threshold: u32)

// SOURCE METADATA

#[payable]
pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)

// OWNER/ADMINS

// NB: for all methods below, caller must be owner

#[payable]
pub fn owner_change_owner(&mut self, new_owner: AccountId)

#[payable]
pub fn owner_add_admins(&mut self, account_ids: Vec<AccountId>)

#[payable]
pub fn owner_remove_admins(&mut self, account_ids: Vec<AccountId>)

```

### Read Methods

```rs
// CONFIG

pub fn get_config(&self) -> Config

// PROVIDERS
pub fn get_provider(&self, contract_id: String, method_name: String) -> Option<ProviderJson>

pub fn get_providers(&self) -> Vec<ProviderJson>

// IS-HUMAN

pub fn is_human(&self, account_id: String) -> bool // TODO: add option for caller to specify providers (with weights) + min_human_threshold

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