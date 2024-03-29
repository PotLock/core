# Potlock Contracts

Welcome to the home of public goods funding on NEAR! ✨🫕 Read more on our mission and roadmap [here](https://potlock.io).

## Introduction

## Overview

The Potlock stack contains 5 main contracts:

### [Pot Factory](pot_factory)

A Factory contract that deploys Pots.

### [Pot](pot)

A configurable, flexible yet secure contract that manages a funding round.

### [Sybil](sybil)
   
A registry for sybil resistance providers, allowing users to collect stamps indicating their verification with registered providers. Additionally, abstracts away individual sybil resistance providers/solutions to provide a single contract to call `is_human`.

### [Registry](registry)

Projects that wish to apply for a Pot (funding round) may be required to be registered on a project Registry. Flexibility is provided to use a 3rd-party registry; this contract is the registry that Potlock uses by default. Each Pot contract that implements a registry requirement will verify the project against the specified Registry when a project applies for the Pot.

### [Donation](donation)

Provides a means to donate NEAR or FTs (soon) to any account.


### [Sybil Provider Simulator](sybil_provider_simulator)

Not technically a part of the PotLock stack, this contract simulates a 3rd-party Sybil Resistance Provider.


## Tests

Integration tests for the earliest implementations of these contracts were written using near-api-js and can be found in the [`/test` directory](test). However, **these tests are no longer up-to-date and are not being maintained.**

Before the public use of these contracts, integration tests should be added using [near-workspaces-rs](https://github.com/near/near-workspaces-rs) (check out additional resources [here](https://docs.near.org/develop/testing/introduction) and [here](https://docs.near.org/sdk/rust/testing/integration-tests)), as well as native unit tests where appropriate.

In the meantime, these contracts have been thoroughly tested manually and via the original tests, and should reliably function as expected.

## Known Issues

- Some TODO's need to be addressed
- FTs not yet supported
- Milestones not yet supported (Pot)
- Additional funding mechanisms other than Quadratic Funding not yet supported (Pot)
- Sybil contract `is_human` method not yet customizable

