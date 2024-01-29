# PotLock SybilProviderSimulator Contract

## Purpose

Provides an example of an extremely 3rd party Sybil resistance provider that can integrate with Nada.Bot and be used for testing purposes. (Obviously leaves out the actual Sybil resistance part; used instead to simulate API)

## Deployed Contracts

- **Staging (Mainnet): `sybilprovidersimulator-1.staging.nadabot.near`**
- **Testnet: `sybilprovidersimulator-1.nadabot.testnet`**

## Contract Structure

### General Types

```rs
```

### Contract

```rs
pub struct Contract {
    account_ids_to_bool: UnorderedMap<AccountId, bool>,
}
```

## Methods

### Write Methods


```rs
// INIT

pub fn new() -> Self


// CHECKS

#[payable]
pub fn get_check(&mut self) // Simulates getting a sybil-resistant check (e.g. connecting twitter, passing face scan, etc)

pub fn remove_check(&mut self)

```

### Read Methods

```rs
// CHECKS

pub fn has_check(&self, account_id: AccountId) -> bool // Simulates the primary method signature that must be implemented for integration with Nada.Bot

```