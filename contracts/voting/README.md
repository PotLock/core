# NEAR Voting Contract

A flexible and secure voting system built on NEAR Protocol that supports multiple election types, voting mechanisms, and voter eligibility verification.

## Purpose

This contract enables the creation and management of various types of elections on the NEAR blockchain, supporting features like:
- Multiple voting types (Simple, Weighted)
- Different election formats (General, Pot-based)
- Configurable voter eligibility (Open, List-based, Token-based)
- Secure vote recording and tallying
- Transparent election results

## Key Features

### Election Types
- **General Elections**: Standard voting process with multiple candidates
- **Pot Elections**: Elections with an associated prize pool

### Voting Mechanisms
- **Simple Voting**: One vote per voter per candidate
- **Weighted Voting**: Voters can assign different weights to their votes (up to a maximum)

### Voter Eligibility
- **Open**: Anyone can vote
- **List-based**: Voters must be on a predefined list on potlock's list contract.
- **Token-based**: Voters must hold a minimum token balance (planned)
- **Custom**: Customizable eligibility criteria through external contracts

## Core Functions

### Election Management

```rust
create_election(
title: String,
description: String,
start_date: U64,
end_date: U64,
votes_per_voter: u32,
voter_eligibility: String,
voting_type: String,
election_type: Value,
candidates: Vec<AccountId>
)
```

- Creates a new election with specified parameters
- Requires deposit for creation

### Candidate Operations

```rust
apply(election_id: u64)
```
- Allows accounts to apply as candidates
- Available during nomination period

### Voting Operations
```rust
vote(election_id: u64, vote: (AccountId, u32))
```
- Casts a vote for a candidate
- Validates voter eligibility and voting rules

### Query Methods
```rust
get_election(election_id: &ElectionId) -> Option<Election>
get_voter_votes(election_id: &ElectionId, voter: &AccountId) -> Option<Vec<Vote>>
get_election_vote_count(election_id: &ElectionId) -> u64
get_candidate_vote_weight(election_id: &ElectionId, candidate_id: &AccountId) -> u64
is_voting_period(election_id: &ElectionId) -> bool
```

## Usage

### Deployment
```bash
near deploy --wasmFile target/near/voting_contract.wasm --accountId your-contract.near
```

### Initialize Contract
```bash
near call your-contract.near new '{"owner_id": "owner.near"}' --accountId owner.near
```

### Create Election
```bash
near call your-contract.near create_election '{
  "title": "Test Election",
  "description": "Test Description",
  "start_date": "1234567890000000000",
  "end_date": "1234567890000000000",
  "votes_per_voter": 1,
  "voter_eligibility": "Open",
  "voting_type": "Simple",
  "election_type": {"General": null},
  "candidates": ["candidate1.near"]
}' --accountId owner.near --deposit 1
```

## Testing

Run the test suite:
```bash
cargo test
```

Key test cases cover:
- Election creation and management
- Voting mechanics
- Eligibility verification
- Result calculation
