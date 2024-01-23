# PotLock Pot Contract

## Purpose

A Pot contract manages a **funding round**. Quadratic Funding (QF) is the only distribution mechanism currently supported, in which projects apply, patrons contribute to the matching pool, end users donate during the "public round" period, and at the end of the round, projects receive a portion of the matching pool relative to the broadness of their public support base.

**Sybil resistance** is essential for QF, as the intent is to demonstrate a broad base of support and the matching pool payouts are calculated accordingly. A single entity acting as multiple donors can therefore cheat the system and result in a larger payout. This Pot contract enables composable, customizable support for integrating sybil resistance services.

The typical flow / lifetime of a Pot is as follows:

- Pot is **deployed** via PotFactory contract
  - Deployer (e.g. DAO that calls `deploy_pot` on PotFactory) is, by default, the "owner" (superuser) of the Pot contract
- After deployment, Pot **configuration** can be updated by permissioned accounts (owner or admins)
- A **chef** account can be set by Pot owner/admin. This account has permissions to change status of applications (e.g. move from `Pending` to `Approved`), as well as calculate and set payouts. Any action that is permissioned for the chef is also permissioned for owner/admins. The chef cannot update Pot configuration details; its primary purpose is to manage applications for the funding round.
- At any time after deployment until the public round has closed, a **patron** can contribute to the **matching pool**. A minimum amount for matching pool donations can be set by the Pot owner/admin via `min_matching_pool_donation_amount`. A `referrer_id` may be included with a matching pool donation, indicating an account to which a percentage of the donation should be sent as a **referral fee**. This percentage is set by the owner/admin via `referral_fee_matching_pool_basis_points`. No additional fees (e.g. protocol or chef fees) are paid out of matching pool donations.
- During the **application period** (between `application_start_ms` and `application_end_ms`), projects may apply to the funding round. Depending on the registration requirement set by the owner/admin via `registry_provider`, projects may be required to be registered on an external registry contract before they can apply.
- During the **public round** (between `public_round_start_ms` and `public_round_end_ms`), end users may donate to approved projects. A `project_id` must be specified with the donation. Similarly to matching pool donations, a `referrer_id` may be provided; the referral fee percentage for public donations is set by the owner/admin via `referral_fee_public_round_basis_points`. Sybil resistance checks may be implemented for public donations by the Pot owner/admin. If a chef is specified on the contract, they will receive a percentage of the donation as specified by `chef_fee_basis_points`. If a `protocol_config_provider` is specified, a cross-contract (CC) call to this provider will be made to retrieve the percentage and recipient account for the protocol fee, and this amount will also be taken out of the donation. The donation must be large enough to cover its own storage _after_ all fees have been subtracted.
- Once the public round is over, **payouts** may be calculated. This occurs off-chain as it is a computationally-expensive operation due to pairwise square root calculations. This calculation logic, however, will live on-chain in a BOS component. It can currently be found in [`test/utils/quadratics.ts`](../test/utils/quadratics.ts). Its required inputs are the total matching pool amount, and all individual donations, which can be fetched via paginated calls to `get_donations`. Once payouts have been calculated off-chain, they should be set on the Pot contract by the chef (or owner/admin). During this process, an error will occur if the total payout amount is not consistent with the matching pool balance.
- Once payouts are set, a **cooldown period** starts (currently hardcoded to one week). The end of the cooldown period is specified by `cooldown_end_ms`, and this can be updated by owner/admin. The intention of the cooldown period is to allow a public audit of the payouts and allow challenges. Once the cooldown period is complete, payouts can be processed and payments will be made from the matching pool to individual projects.
- Once payouts have all been processed and paid out, without errors, `all_paid_out` is set to `true` and this is considered the end of life for the Pot.

## Contract Types / Structure

### Contract / Config

```rs
/// Contract state as stored on-chain
pub struct Contract {
    // PERMISSIONED ACCOUNTS
    /// Owner of the contract
    owner: AccountId,
    /// Admins of the contract (Owner, which should in most cases be DAO, might want to delegate admin rights to other accounts)
    admins: UnorderedSet<AccountId>,
    /// Address (ID) of Pot manager ("chef"). This account is responsible for managing the Pot, e.g. reviewing applications, setting payouts, etc.
    /// Optional because it may be set after deployment.
    chef: LazyOption<AccountId>,

    // POT CONFIG
    /// User-facing name for this Pot
    pot_name: String,
    /// User-facing description for this Pot
    pot_description: String,
    /// Maximum number of projects that can be approved for the round. Considerations include gas limits for payouts, etc.
    max_projects: u32,
    /// Base currency for the round
    /// * NB: currently only `"near"` is supported
    base_currency: AccountId,
    /// MS Timestamp when applications can be submitted from
    application_start_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted until
    application_end_ms: TimestampMs,
    /// MS Timestamp when the public round starts
    public_round_start_ms: TimestampMs,
    /// MS Timestamp when the round ends
    public_round_end_ms: TimestampMs,
    /// Account ID that deployed this Pot contract (set at deployment, cannot be updated)
    deployed_by: AccountId,
    /// Contract ID + method name of registry provider that should be queried when projects apply to round. Method specified must receive "account_id" and return bool indicating registration status.
    /// * Optional because not all Pots will require registration, and those that do might set after deployment.
    registry_provider: LazyOption<ProviderId>,
    /// Minimum amount that can be donated to the matching pool
    min_matching_pool_donation_amount: U128,

    // SYBIL RESISTANCE
    /// Sybil contract address & method name that will be called to verify humanness. If `None`, no checks will be made.
    sybil_wrapper_provider: LazyOption<ProviderId>,
    /// Sybil checks (if using custom sybil config)
    custom_sybil_checks: LazyOption<HashMap<ProviderId, SybilProviderWeight>>,
    /// Minimum threshold score for Sybil checks (if using custom sybil config)
    custom_min_threshold_score: LazyOption<u32>,

    // FEES
    /// Basis points (1/100 of a percent) that should be paid to an account that refers a Patron (paid at the point when the matching pool donation comes in)
    referral_fee_matching_pool_basis_points: u32,
    /// Basis points (1/100 of a percent) that should be paid to an account that refers a donor (paid at the point when the donation comes in)
    referral_fee_public_round_basis_points: u32,
    /// Chef's fee for managing the round. Gets taken out of each donation as they come in and are paid out
    chef_fee_basis_points: u32,

    // FUNDS & BALANCES
    /// Total matching pool donations
    total_matching_pool_donations: U128,
    /// Amount of matching funds available (not yet paid out)
    matching_pool_balance: U128,
    /// Total public donations
    total_public_donations: U128,

    // PAYOUTS
    /// Cooldown period starts when Chef sets payouts
    cooldown_end_ms: LazyOption<TimestampMs>,
    /// Indicates whether all projects been paid out (this would be considered the "end-of-lifecycle" for the Pot)
    all_paid_out: bool,

    // MAPPINGS
    /// All application records, versioned for easy upgradeability
    applications_by_id: UnorderedMap<ApplicationId, VersionedApplication>,
    /// Approved application IDs
    approved_application_ids: UnorderedSet<ApplicationId>,
    /// All donation records, versioned for easy upgradeability
    donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
    /// IDs of public round donations (made by donors who are not Patrons, during public round)
    public_round_donation_ids: UnorderedSet<DonationId>,
    /// IDs of matching pool donations (made by Patrons)
    matching_pool_donation_ids: UnorderedSet<DonationId>,
    /// IDs of donations made to a given project
    donation_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<DonationId>>,
    /// IDs of donations made by a given donor (user)
    donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    /// All payout records, versioned for easy upgradeability
    payouts_by_id: UnorderedMap<PayoutId, VersionedPayout>,
    payout_ids_by_project_id: LookupMap<ProjectId, UnorderedSet<PayoutId>>,

    // OTHER
    /// contract ID + method name of protocol config provider that should be queried for protocol fee basis points and protocol fee recipient account.
    /// Method specified must receive no requried args and return struct containing protocol_fee_basis_points and protocol_fee_recipient_account.
    /// Set by deployer and cannot be changed by Pot owner/admins.
    protocol_config_provider: LazyOption<ProviderId>,
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
}

/// Ephemeral-only external struct (used in views)
pub struct PotConfig {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub chef: Option<AccountId>,
    pub pot_name: String,
    pub pot_description: String,
    pub max_projects: u32,
    pub base_currency: AccountId,
    pub application_start_ms: TimestampMs,
    pub application_end_ms: TimestampMs,
    pub public_round_start_ms: TimestampMs,
    pub public_round_end_ms: TimestampMs,
    pub deployed_by: AccountId,
    pub registry_provider: Option<ProviderId>,
    pub min_matching_pool_donation_amount: U128,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<HashMap<ProviderId, SybilProviderWeight>>,
    pub custom_min_threshold_score: Option<u32>,
    pub referral_fee_matching_pool_basis_points: u32,
    pub referral_fee_public_round_basis_points: u32,
    pub chef_fee_basis_points: u32,
    pub matching_pool_balance: U128,
    pub total_public_donations: U128,
    pub cooldown_end_ms: Option<TimestampMs>,
    pub all_paid_out: bool,
    pub protocol_config_provider: Option<ProviderId>,
}

/// Ephemeral-only
pub struct UpdatePotArgs {
    pub owner: Option<AccountId>,
    pub admins: Option<Vec<AccountId>>,
    pub chef: Option<AccountId>,
    pub pot_name: Option<String>,
    pub pot_description: Option<String>,
    pub max_projects: Option<u32>,
    pub application_start_ms: Option<TimestampMs>,
    pub application_end_ms: Option<TimestampMs>,
    pub public_round_start_ms: Option<TimestampMs>,
    pub public_round_end_ms: Option<TimestampMs>,
    pub registry_provider: Option<ProviderId>,
    pub min_matching_pool_donation_amount: Option<U128>,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    pub custom_min_threshold_score: Option<u32>,
    pub referral_fee_matching_pool_basis_points: Option<u32>,
    pub referral_fee_public_round_basis_points: Option<u32>,
    pub chef_fee_basis_points: Option<u32>,
}

/// Result expected from protocol_config_provider when querying for protocol fee configuration
pub struct ProtocolConfigProviderResult {
    pub basis_points: u32,
    pub account_id: AccountId,
}
```

### Applications
```rs
pub type ProjectId = AccountId;
pub type ApplicationId = ProjectId; // Applications are indexed by ProjectId

pub struct Application {
    /// functions as unique identifier for application, since projects can only apply once per round
    // NB: Don't technically need this, since we use the project_id as the key in the applications_by_id mapping, but it's possible that we'll want to change that in the future, so keeping this for now
    pub project_id: ProjectId,
    /// Status of the project application (Pending, Accepted, Rejected, InReview)
    pub status: ApplicationStatus,
    /// Timestamp for when the application was submitted
    pub submitted_at: TimestampMs,
    /// Timestamp for when the application was last updated (e.g. status changed)
    pub updated_at: Option<TimestampMs>,
    /// Notes to be added by Chef when reviewing the application
    pub review_notes: Option<String>,
}

pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
    InReview,
}
```

### Donations
```rs
pub type DonationId = u64; // auto-incrementing ID for donations

pub struct Donation {
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation, if applicable (matching pool donations will contain `None`)
    pub project_id: Option<ProjectId>,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Protocol fee
    pub protocol_fee: U128,
    // TODO: add chef fee? chef ID? this is getting pretty hefty though for something intended to be small (could cost 0.01N just to store the Donation)
}

/// Ephemeral-only (used in views)
pub struct DonationExternal {
    /// ID of the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation, if applicable (matching pool donations will contain `None`)
    pub project_id: Option<ProjectId>,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Indicates whether this is matching pool donation
    pub matching_pool: bool,
    // TODO: add chef fee?
}

pub const DONATION_ID_DELIMETER: &str = ":";

```

### Payouts

```rs
pub const PAYOUT_ID_DELIMITER: &str = ":";
pub type PayoutId = String; // concatenation of application_id + PAYOUT_ID_DELIMITER + incrementing integer per-project

pub struct Payout {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// ID of the application receiving the payout
    pub project_id: ProjectId,
    /// Amount to be paid out
    pub amount: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
}

/// Ephemeral-only; used for setting payouts
pub struct PayoutInput {
    pub amount: U128,
    pub project_id: ProjectId,
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

A Sybil Provider can be used to enhance sybil resistance for public donations. This provider acts as a wrapper around individual sybil resistance providers (e.g. I-Am-Human, Wormhole, etc) and can be either used in its default configuration, or customized by providing `CustomSybilCheck`s.

```rs
/// Weighting for a given CustomSybilCheck
type SybilProviderWeight = u32;

/// Ephemeral-only
pub struct CustomSybilCheck {
    contract_id: AccountId,
    method_name: String,
    weight: SybilProviderWeight,
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

**NB: ALL privileged write methods (those beginning with `chef_*`, `admin_*` or `owner_*`) require an attached deposit of at least one yoctoNEAR, for security purposes.**

```rs
// INIT

pub fn new(
    // permissioned accounts
    owner: Option<AccountId>, // defaults to signer account if not provided
    admins: Option<Vec<AccountId>>,
    chef: Option<AccountId>,

    // pot config
    pot_name: String,
    pot_description: String,
    max_projects: u32,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    public_round_start_ms: TimestampMs,
    public_round_end_ms: TimestampMs,
    registry_provider: Option<ProviderId>,
    min_matching_pool_donation_amount: Option<U128>,

    // sybil resistance
    sybil_wrapper_provider: Option<ProviderId>,
    custom_sybil_checks: Option<HashMap<ProviderId, SybilProviderWeight>>,
    custom_min_threshold_score: Option<u32>,

    // fees
    referral_fee_matching_pool_basis_points: u32, // this could be optional with a default, but better to set explicitly for now
    referral_fee_public_round_basis_points: u32, // this could be optional with a default, but better to set explicitly for now
    chef_fee_basis_points: u32,

    // other
    protocol_config_provider: Option<ProviderId>,
    source_metadata: ContractSourceMetadata,
) -> Self


// APPLICATIONS

/// The calling account should be the project/account that is applying
#[payable]
pub fn apply(&mut self) -> Application

/// Only allowed for projects/applications that are in Pending status
pub fn unapply(&mut self) -> ()

#[payable]
pub fn chef_set_application_status(
    &mut self,
    project_id: ProjectId,
    status: ApplicationStatus,
    notes: String,
) -> Application

// convenience methods that wrap chef_set_application_status (may remove, TBD)

#[payable]
pub fn chef_mark_application_approved(
    &mut self,
    project_id: ProjectId,
    notes: String,
) -> Application

#[payable]
pub fn chef_mark_application_rejected(
    &mut self,
    project_id: ProjectId,
    notes: String,
) -> Application

#[payable]
pub fn chef_mark_application_in_review(
    &mut self,
    project_id: ProjectId,
    notes: String,
) -> Application

#[payable]
pub fn chef_mark_application_pending(
    &mut self,
    project_id: ProjectId,
    notes: String,
) -> Application


// DONATIONS

#[payable]
pub fn donate(
    &mut self,
    project_id: Option<ProjectId>,
    message: Option<String>,
    referrer_id: Option<AccountId>,
    matching_pool: Option<bool>,
    bypass_protocol_fee: Option<bool>, // Allows donor to bypass protocol fee if they wish. Defaults to "false".
) -> DonationExternal


// PAYOUTS

#[payable]
pub fn chef_set_payouts(&mut self, payouts: Vec<PayoutInput>) -> ()

#[payable]
pub fn admin_process_payouts(&mut self) -> ()


// CONFIG

#[payable]
pub fn admin_dangerously_set_pot_config(&mut self, update_args: UpdatePotArgs) -> PotConfig

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
pub fn admin_set_chef(&mut self, chef: AccountId) -> ()

#[payable]
pub fn admin_remove_chef(&mut self) -> ()

#[payable]
pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) -> ()

#[payable]
pub fn admin_set_pot_name(&mut self, pot_name: String) -> ()

#[payable]
pub fn admin_set_pot_description(&mut self, pot_description: String) -> ()

#[payable]
pub fn admin_set_max_projects(&mut self, max_projects: u32) -> ()

#[payable]
pub fn admin_set_base_currency(&mut self, base_currency: AccountId) -> ()

#[payable]
pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) -> ()

#[payable]
pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) -> ()

#[payable]
pub fn admin_set_public_round_start_ms(&mut self, public_round_start_ms: u64) -> ()

#[payable]
pub fn admin_set_public_round_end_ms(&mut self, public_round_end_ms: u64) -> ()

/// Sets `public_round_start_ms` to env::block_timestamp_ms()
#[payable]
pub fn admin_set_public_round_open(&mut self, public_round_end_ms: TimestampMs) -> ()

/// Sets `public_round_end_ms` to env::block_timestamp_ms()
#[payable]
pub fn admin_set_public_round_closed(&mut self) -> ()

#[payable]
pub fn admin_set_registry_provider(&mut self, contract_id: AccountId, method_name: String) -> ()

#[payable]
pub fn admin_remove_registry_provider(&mut self) -> ()

#[payable]
pub fn admin_set_min_matching_pool_donation_amount(&mut self, min_matching_pool_donation_amount: U128) -> ()

#[payable]
pub fn admin_set_sybil_wrapper_provider(
    &mut self,
    contract_id: AccountId,
    method_name: String,
) -> ()

#[payable]
pub fn admin_remove_sybil_wrapper_provider(&mut self) -> ()

#[payable]
pub fn admin_set_custom_sybil_checks(&mut self, custom_sybil_checks: Vec<CustomSybilCheck>) -> ()

#[payable]
pub fn admin_remove_custom_sybil_checks(&mut self) -> ()

#[payable]
pub fn admin_set_custom_min_threshold_score(&mut self, custom_min_threshold_score: u32) -> ()

#[payable]
pub fn admin_remove_custom_min_threshold_score(&mut self) -> ()

#[payable]
pub fn admin_set_referral_fee_matching_pool_basis_points(
    &mut self,
    referral_fee_matching_pool_basis_points: u32,
) -> ()

#[payable]
pub fn admin_set_referral_fee_public_round_basis_points(
    &mut self,
    referral_fee_public_round_basis_points: u32,
) -> ()

#[payable]
pub fn admin_set_cooldown_period_complete(&mut self) -> ()


// SOURCE METADATA

pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)

```

### Read Methods

```rs
// POT CONFIG

pub fn get_config(&self) -> PotConfig


// APPLICATIONS

pub fn get_applications(
    &self,
    from_index: Option<u64>,
    limit: Option<u64>,
    status: Option<ApplicationStatus>,
) -> Vec<Application>

pub fn get_approved_applications(
    &self,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<Application>

pub fn get_application_by_project_id(&self, project_id: ProjectId) -> Application


// DONATIONS

/// Get all donations (both matching pool and public round)
pub fn get_donations(
    &self,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<DonationExternal>

pub fn get_public_round_donations(
    &self,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<DonationExternal>

pub fn get_matching_pool_donations(
    &self,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<DonationExternal>

pub fn get_donations_for_project(
    &self,
    project_id: ProjectId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<DonationExternal>

pub fn get_donations_for_donor(
    &self,
    donor_id: AccountId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<DonationExternal>


// PAYOUTS

pub fn get_payouts(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Payout>


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>

```