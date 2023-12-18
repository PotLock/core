# PotLock PotFactory Contract

## Purpose

The PotFactory contract allows any end user (or a whitelisted user, depending on configuration) to deploy a Pot as a subaccount of the PotFactory.

The PotFactory also serves as the provider of protocol configurations, namely the basis points and recipient account for the protocol fee, to be queried by individual Pot contracts.

## Contract Types / Structure

### Contract

```rs
/// Contract state as stored on-chain
pub struct Contract {
    /// Contract superuser (should be a DAO, but no restrictions made at the contract level on this matter)
    owner: AccountId,
    /// Admins, which can be added/removed by the owner
    admins: UnorderedSet<AccountId>,
    /// Records of all Pots deployed by this Factory, indexed at their account ID, versioned for easy upgradeability
    pots_by_id: UnorderedMap<PotId, VersionedPot>,
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

/// Ephemeral-only external struct (used in views)
pub struct ContractConfigExternal {
    owner: AccountId,
    admins: Vec<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_chef_fee_basis_points: u32,
    whitelisted_deployers: Vec<AccountId>,
    require_whitelist: bool,
}
```

### Protocol Config
```rs
/// Ephemeral-only (used in views) - intended as the result type for Pots querying for protocol fees configuration
pub struct ProtocolConfig {
    pub basis_points: u32,
    pub account_id: AccountId,
}
```

### Providers

A "Provider" is a contract address + method name combination that "provides" some information or service, such as a `RegistryProvider` (which provides information on whether an account is on a registry), a `SybilProvider` (which provides information on whether an account is considered "human"), or a `ProtocolConfigProvider` (which provides information on protocol fee and recipient account).

```rs
pub struct ProviderId(pub String);

pub const PROVIDER_ID_DELIMITER: &str = ":"; // separates contract_id and method_name in ProviderId

impl ProviderId {
    /// Generate ProviderId ("`{CONTRACT_ADDRESS}:{METHOD_NAME}`") from contract_id and method_name
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
}
```

### Sybil configuration

A Sybil Provider can be used to enhance sybil resistance in the Pot contracts. This provider acts as a wrapper around individual sybil resistance providers (e.g. I-Am-Human, Wormhole, etc) and can be either used in its default configuration, or customized by providing `CustomSybilCheck`s. (These can also be added or updated in the Pot contract, after deployment.)

```rs
/// Weighting for a given CustomSybilCheck
type SybilProviderWeight = u32;

/// Ephemeral-only (used in custom_sybil_checks for setting on Pot deployment, but not stored in this contract; rather, stored in Pot contract)
pub struct CustomSybilCheck {
    contract_id: AccountId,
    method_name: String,
    weight: SybilProviderWeight,
}
```

### Pots

```rs
/// The address of a deployed Pot contract
pub type PotId = AccountId;

/// Internal record of a Pot deployed by the Factory (indexed at PotId)
pub struct Pot {
    pub deployed_by: AccountId,
    pub deployed_at_ms: TimestampMs,
}

/// Ephemeral-only Pot struct (used for views; not stored in contract)
pub struct PotExternal {
    id: PotId,
    deployed_by: AccountId,
    deployed_at_ms: TimestampMs,
}

/// Arguments that must be provided to deploy a new Pot; these must be kept up-to-date with the Pot contract
pub struct PotArgs {
    owner: Option<AccountId>,
    admins: Option<Vec<AccountId>>,
    chef: Option<AccountId>,
    pot_name: String,
    pot_description: String,
    max_projects: u32,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    public_round_start_ms: TimestampMs,
    public_round_end_ms: TimestampMs,
    registry_provider: Option<ProviderId>,
    sybil_wrapper_provider: Option<ProviderId>,
    custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    custom_min_threshold_score: Option<u32>,
    patron_referral_fee_basis_points: u32,
    public_round_referral_fee_basis_points: u32,
    chef_fee_basis_points: u32,
    protocol_config_provider: Option<ProviderId>,
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

### Write Methods

```rs
// INIT

pub fn new(
    owner: AccountId,
    admins: Vec<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_chef_fee_basis_points: u32,
    whitelisted_deployers: Vec<AccountId>,
    require_whitelist: bool,
    source_metadata: ContractSourceMetadata,
) -> Self


// POTS

/// Deploy a new Pot. A `None` response indicates an unsuccessful deployment.
#[payable]
pub fn deploy_pot(&mut self, mut pot_args: PotArgs) -> Option<PotExternal>


// OWNER / ADMIN

#[payable]
pub fn owner_change_owner(&mut self, owner: AccountId) -> ()

#[payable]
pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) -> ()

#[payable]
pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) -> ()

#[payable]
pub fn owner_set_admins(&mut self, account_ids: Vec<AccountId>) -> ()

#[payable]
pub fn owner_clear_admins(&mut self) -> ()

#[payable]
pub fn admin_set_protocol_fee_basis_points(&mut self, protocol_fee_basis_points: u32) -> ()

#[payable]
pub fn admin_set_protocol_fee_recipient_account(&mut self, protocol_fee_recipient_account: AccountId) -> ()

#[payable]
pub fn admin_set_protocol_config(&mut self, protocol_fee_basis_points: u32, protocol_fee_recipient_account: AccountId) -> ()

#[payable]
pub fn admin_set_default_chef_fee_basis_points(&mut self, default_chef_fee_basis_points: u32) -> ()

#[payable]
pub fn admin_add_whitelisted_deployers(&mut self, whitelisted_deployers: Vec<AccountId>) -> ()

#[payable]
pub fn admin_remove_whitelisted_deployers(&mut self, whitelisted_deployers: Vec<AccountId>) -> ()

#[payable]
pub fn admin_set_require_whitelist(&mut self, require_whitelist: bool) -> ()


// SOURCE METADATA

pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)
```

### Read Methods

```rs
// CONTRACT CONFIG

pub fn get_config(&self) -> ContractConfigExternal


// POTS

pub fn get_pots(&self) -> Vec<PotExternal>

pub fn get_min_deployment_deposit(&self, args: &PotArgs) -> u128

/// Method intended for use by Pot contract querying for protocol fee configuration
pub fn get_protocol_config(&self) -> ProtocolConfig


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>
```