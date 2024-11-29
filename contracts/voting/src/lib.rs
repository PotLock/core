use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, near, NearToken, log, require, serde_json::json, Promise};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::store::{IterableSet, LookupMap, IterableMap};
use near_sdk::json_types::U64;



pub mod vote;
pub mod elections;
pub mod events;
mod ext;
pub mod util;
mod internal;

pub use crate::vote::*;
pub use crate::elections::*;
pub use crate::util::*;
pub use crate::events::*;

#[near(serializers=[borsh, json])]
#[derive(Debug, PartialEq, Clone)]
pub enum VotingType {
    Simple,
    Weighted(u32),  // Maximum weight a voter vote can carry
}
#[near(serializers = [borsh])]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Elections,
    Admins,
    Candidates { election_id: u64 },
    Votes,
    VoterEligibility { election_id: u64 },
    CandidateEligibility { election_id: u64 },
    ElectionVotes { election_id: u64 },
}

pub type ElectionId = u64;

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    admins: IterableSet<AccountId>,
    elections: IterableMap<ElectionId, Election>,
    candidates: LookupMap<ElectionId, IterableMap<AccountId, Candidate>>,
    votes: LookupMap<ElectionId, IterableMap<AccountId, Vec<Vote>>>,
    election_counter: ElectionId,
    paused: bool,
}


#[near]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        let contract = Self {
            owner: owner_id,
            admins: IterableSet::new(StorageKey::Admins),
            elections: IterableMap::new(StorageKey::Elections),
            candidates: LookupMap::new(b"c"),
            votes: LookupMap::new(b"v"),
            election_counter: 0,
            paused: false,
        };
        contract
    }

    #[payable]
    pub fn pause(&mut self) {
        self.assert_admin_or_owner();
        self.paused = true;
        log!("Contract paused");
    }

    #[payable]
    pub fn unpause(&mut self) {
        self.assert_admin_or_owner();
        assert!(self.paused, "Contract is not paused");
        self.paused = false;
        log!("Contract unpaused");
    }
}