# Campaign Contract

## Purpose

Provide a way to raise funds, for yourself as an organization, or on behalf of an organization, through donations, raising on behalf of organizations has an approval process to indicate whether the campaign is "official".
Contract can also function as an escrow with minimum target amounts, refunding donors if the target is not met. Campaigns can be time-based.

The typical flow / lifetime of a camapign is as follows:

- Campaign is **created** via Campaign contract's `create_campaign` function call
  - Creator (e.g. user that calls `create_campaign` on contract) is, by default, the "owner" of the campaign
- After creation, some Campaign details can be updated by owner.
- A **recipient** account will be set during campaign creation, it defines who/what the campaign is for
- During the **campaign**(between `campaign.start_ms` and `campaign.end_ms`), end users may donate to the campaign. A `campaign_id` must be specified during donations.
- Donations are either held in escrow(until minimum target is reached) or transfered to the recipient.
- once rthe campaign is over, donations are processed in batch, sent to recipient and fees sent to the appropriate channels

## Contract Structure

### General Types

```rs
type CampaignId = u64;
type DonationId = u64;
type TimestampMs = u64;
type ReferrerPayouts = HashMap<AccountId, Balance>;
```

### Contract

```rs
pub struct Contract {
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    admins: IterableSet<AccountId>,
    protocol_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    default_referral_fee_basis_points: u32,
    default_creator_fee_basis_points: u32,
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
    storage_deposits: IterableMap<AccountId, Balance>,
}

/// NOT stored in contract storage; only used for get_config response
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
```

### Campaigns

_NB: Campaigns can be created on behalf of others, hence the recipient field.

```rs
pub struct Campaign {
    pub owner: AccountId,
    pub name: String,
    pub description: Option<String>,
    pub cover_image_url: Option<String>,
    pub recipient: AccountId,
    pub start_ms: TimestampMs,
    pub end_ms: Option<TimestampMs>,
    pub created_ms: TimestampMs,
    pub ft_id: Option<AccountId>,
    pub target_amount: Balance,
    pub min_amount: Option<Balance>,
    pub max_amount: Option<Balance>,
    pub total_raised_amount: Balance,
    pub net_raised_amount: Balance,
    pub escrow_balance: Balance,
    pub referral_fee_basis_points: u32,
    pub creator_fee_basis_points: u32,
    pub allow_fee_avoidance: bool,
}
```

### Storage

The storage-related methods (`storage_deposit`, `storage_withdraw` and `storage_balance_of`) are utilized for fungible token (FT) donations, where the user must prepay storage on this Campaign contract - to cover the storage of the Donation data - before calling `ft_transfer_call` on the FT contract.

This is a simplified version of the [Storage Management standard](https://nomicon.io/Standards/StorageManagement).

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
        owner: AccountId,
        protocol_fee_basis_points: u32,
        protocol_fee_recipient_account: AccountId,
        default_referral_fee_basis_points: u32,
        default_creator_fee_basis_points: u32,
        source_metadata: ContractSourceMetadata,
    ) -> Self {


// Campaign

#[payable]
pub fn create_campaign(
        &mut self,
        name: String,
        description: Option<String>,
        cover_image_url: Option<String>,
        recipient: AccountId,
        start_ms: TimestampMs,
        end_ms: Option<TimestampMs>,
        ft_id: Option<AccountId>,
        target_amount: U128,
        min_amount: Option<U128>,
        max_amount: Option<U128>,
        referral_fee_basis_points: Option<u32>,
        creator_fee_basis_points: Option<u32>,
        allow_fee_avoidance: Option<bool>,
    ) -> CampaignExternal
    

#[payable]
pub fn update_campaign(
   &mut self,
   campaign_id: CampaignId,
   name: Option<String>,
   description: Option<String>,
   cover_image_url: Option<String>,
   start_ms: Option<TimestampMs>,
   end_ms: Option<TimestampMs>,
   ft_id: Option<AccountId>,
   target_amount: Option<Balance>,
   max_amount: Option<Balance>,
   min_amount: Option<U128>, // Can only be provided if campaign has not started yet
   allow_fee_avoidance: Option<bool>,
   // NB: recipient cannot be updated. If incorrect recipient is specified, campaign should be deleted and recreated
) -> CampaignExternal {


pub fn delete_campaign(&mut self, campaign_id: CampaignId)

// Donation 
#[payable]
pub fn donate(
        &mut self,
        campaign_id: CampaignId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        bypass_protocol_fee: Option<bool>,
        bypass_creator_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {


// STORAGE

pub fn storage_deposit(&mut self) -> U128

pub fn storage_withdraw(&mut self, amount: Option<U128>) -> U128


// OWNER

#[payable]
pub fn owner_change_owner(&mut self, owner: AccountId)

pub fn owner_add_admins(&mut self, admins: Vec<AccountId>)
pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>)
pub fn owner_clear_admins(&mut self)

// SOURCE METADATA

pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)

```

### Read Methods

```rs
// CONFIG

pub fn get_config(&self) -> Config

// CAMPAIGNS
get_campaign(&self, campaign_id: CampaignId) -> CampaignExternal

get_campaigns(
        &self,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal>
    

pub fn get_campaigns_by_owner(
        &self,
        owner_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal>

pub fn get_campaigns_by_recipient(
        &self,
        recipient_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u128>,
    ) -> Vec<CampaignExternal>



// DONATIONS
pub fn get_donations(&self, from_index: Option<u128>, limit: Option<u64>) -> Vec<DonationExternal>

pub fn get_donation_by_id(&self, donation_id: DonationId) -> Option<DonationExternal>

pub fn get_donations_for_campaign(
        &self,
        campaign_id: CampaignId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal>

pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal>


// STORAGE

pub fn storage_balance_of(&self, account_id: &AccountId) -> U128


// OWNER

pub fn get_owner(&self) -> AccountId


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>
```

## Events

### `campaign`

Indicates that a `Campaign` object has been created.

**Example:**

```json
{
  "standard": "potlock",
  "version": "1.0.0",
  "event": "campaign_create",
  "data": [
    {
      "campaign": {
        "owner": ""
      }
    }
  ]
}
```

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
        "commit_hash": "ec02294253b22c2d4c50a75331df23ada9eb04db",
        "link": "https://github.com/PotLock/core",
        "version": "0.1.0"
      }
    }
  ]
}
```
