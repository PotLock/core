# PotLock Donation Contract

## Purpose

Provide a way to donate NEAR or FTs to any account, with a protocol fee taken out

## Contract Structure

### General Types

```rs
type DonationId = u64;
type TimestampMs = u64;
```

### Contract

```rs
pub struct Contract {
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    owner: AccountId,
    protocol_fee_basis_points: u32,
    referral_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    donations_by_id: UnorderedMap<DonationId, VersionedDonation>,
    donation_ids_by_recipient_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    donation_ids_by_donor_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    donation_ids_by_ft_id: LookupMap<AccountId, UnorderedSet<DonationId>>,
    total_donations_amount: Balance, // Added total_donations_amount to track total donations amount without iterating through all donations
    net_donations_amount: Balance,   // Added net_donations_amount to track net donations amount (after fees) without iterating through all donations
    total_protocol_fees: Balance,    // Added total_protocol_fees to track total protocol fees without iterating through all donations
    total_referrer_fees: Balance,    // Added total_referrer_fees to track total referral fees without iterating through all donations
}

/// NOT stored in contract storage; only used for get_config response
pub struct Config {
    pub owner: AccountId,
    pub protocol_fee_basis_points: u32,
    pub referral_fee_basis_points: u32,
    pub protocol_fee_recipient_account: AccountId,
    pub total_donations_amount: U128,
    pub net_donations_amount: U128,
    pub total_donations_count: U64,
    pub total_protocol_fees: U128,
    pub total_referrer_fees: U128,
}
```

### Donations

_NB: Projects are automatically approved by default._

```rs
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor
    pub donor_id: AccountId,
    /// Amount donated
    pub total_amount: U128,
    /// FT id (e.g. "near")
    pub ft_id: AccountId,
    /// Optional message from the donor
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// ID of the account receiving the donation
    pub recipient_id: AccountId,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
}
```

### Storage

The storage-related methods (`storage_deposit`, `storage_withdraw` and `storage_balance_of`) are utilized for fungible token (FT) donations, where the user must prepay storage on this Donation contract - to cover the storage of the Donation data - before calling `ft_transfer_call` on the FT contract.

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
    referral_fee_basis_points: u32,
    protocol_fee_recipient_account: AccountId,
    source_metadata: ContractSourceMetadata,
) -> Self


// DONATIONS

#[payable]
pub fn donate(
    &mut self,
    recipient_id: AccountId,
    message: Option<String>,
    referrer_id: Option<AccountId>,
    bypass_protocol_fee: Option<bool>, // Allows donor to bypass protocol fee if they wish. Defaults to "false".
) -> Donation


// STORAGE

pub fn storage_deposit(&mut self) -> U128

pub fn storage_withdraw(&mut self, amount: Option<U128>) -> U128


// OWNER

#[payable]
pub fn owner_change_owner(&mut self, owner: AccountId)

pub fn owner_set_protocol_fee_basis_points(&mut self, protocol_fee_basis_points: u32)

pub fn owner_set_referral_fee_basis_points(&mut self, referral_fee_basis_points: u32)

pub fn owner_set_protocol_fee_recipient_account(&mut self, protocol_fee_recipient_account: AccountId)


// SOURCE METADATA

pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)

```

### Read Methods

```rs
// CONFIG

pub fn get_config(&self) -> Config


// DONATIONS
pub fn get_donations(&self, from_index: Option<u128>, limit: Option<u64>) -> Vec<Donation>

pub fn get_donation_by_id(&self, donation_id: DonationId) -> Option<Donation>

pub fn get_donations_for_recipient(
        &self,
        recipient_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation>

pub fn get_donations_for_donor(
    &self,
    donor_id: AccountId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<Donation>

pub fn get_donations_for_ft(
    &self,
    ft_id: AccountId,
    from_index: Option<u128>,
    limit: Option<u64>,
) -> Vec<Donation>


// STORAGE

pub fn storage_balance_of(&self, account_id: &AccountId) -> U128


// OWNER

pub fn get_owner(&self) -> AccountId


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>
```

## Events

### `donation`

Indicates that a `Donation` object has been created.

**Example:**

```json
{
  "standard": "potlock",
  "version": "1.0.0",
  "event": "donation",
  "data": [
    {
      "donation": {
        "donated_at_ms": 1698948121940,
        "donor_id": "lachlan.near",
        "ft_id": "near",
        "id": 9,
        "message": "Go go go!",
        "protocol_fee": "7000000000000000000000",
        "recipient_id": "magicbuild.near",
        "referrer_fee": "2000000000000000000000",
        "referrer_id": "plugrel.near",
        "total_amount": "100000000000000000000000"
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
