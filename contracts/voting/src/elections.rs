use crate::*;
use near_sdk::json_types::U128;

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub enum ElectionType {
    GeneralElection,
    ProjectProposal(AccountId),
    Pot(AccountId),
    Custom(String, Option<AccountId>),
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}
#[near(serializers=[borsh, json])]
pub enum ElectionPhase {
    Pending,
    Nomination,
    Voting,
    Ended,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub struct Candidate {
    pub account_id: AccountId,
    pub status: ApplicationStatus,
    pub votes_received: u64,
    pub application_date: U64,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub enum EligibilityType {
    Open,
    ListBased(AccountId, U128),  // list contract and id
    TokenBased(AccountId, U128), // Token contract and minimum balance
    Custom(String),              // Custom eligibility contract address
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub enum ElectionStatus {
    Pending,
    NominationPeriod,
    VotingPeriod,
    ChallengePeriod,
    Completed,
    Cancelled,
}

#[near(serializers=[borsh, json])]
#[derive(Clone, PartialEq)]
pub struct Election {
    pub id: ElectionId,
    pub title: String,
    pub description: String,
    pub start_date: U64,
    pub end_date: U64,
    pub votes_per_voter: u32,
    pub voting_type: VotingType,
    pub voter_eligibility: EligibilityType,
    pub owner: AccountId,
    pub status: ElectionStatus,
    pub challenge_period_end: Option<U64>,
    pub winner_ids: Vec<AccountId>,
    pub election_type: ElectionType,
}

pub type ElectionId = u64;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_election(
        &mut self,
        title: String,
        description: String,
        start_date: U64,
        end_date: U64,
        votes_per_voter: u32,
        voter_eligibility: EligibilityType,
        voting_type: VotingType,
        election_type: ElectionType,
        candidates: Vec<AccountId>,
    ) -> ElectionId {
        self.assert_not_paused();
        self.assert_admin_or_owner();
        assert!(
            start_date.0 < end_date.0,
            "Start date must be before end date"
        );
        let initial_storage_usage = env::storage_usage();

        let election_id = self.election_counter;
        self.election_counter += 1;

        let election = Election {
            id: election_id,
            title,
            description,
            start_date,
            end_date,
            votes_per_voter,
            voter_eligibility,
            voting_type,
            owner: env::predecessor_account_id(),
            status: ElectionStatus::Pending,
            challenge_period_end: None,
            winner_ids: Vec::new(),
            election_type,
        };

        self.elections.insert(election_id, election.clone());

        let mut candidates_map: IterableMap<AccountId, Candidate> =
            IterableMap::new(StorageKey::Candidates { election_id });
        for candidate in candidates.clone() {
            candidates_map.insert(
                candidate.clone(),
                Candidate {
                    account_id: candidate,
                    status: ApplicationStatus::Approved,
                    votes_received: 0,
                    application_date: U64(env::block_timestamp_ms()),
                },
            );
        }

        candidates_map.flush();
        self.candidates.insert(election_id, candidates_map);

        let election_votes: IterableMap<AccountId, Vec<Vote>> =
            IterableMap::new(StorageKey::ElectionVotes { election_id });
        
        self.votes.insert(election_id, election_votes);

        self.elections.flush();
        self.candidates.flush();
        self.votes.flush();

        refund_deposit(initial_storage_usage);

        log_election_created_event(election, &candidates);

        election_id
    }

    /*
    #[payable]
    pub fn apply(&mut self, election_id: ElectionId) {
        // let election = self.elections.get(&election_id).expect("Election not found");
        // println!("Nomination end date: {:?}", env::block_timestamp_ms());


        // let applicant = env::predecessor_account_id();
        // // assert!(self.is_eligible_candidate(&election, &applicant), "Not eligible to be a candidate");

        // let candidate = Candidate {
        //     account_id: applicant.clone(),
        //     status: if election.auto_approval {
        //         ApplicationStatus::Approved
        //     } else {
        //         ApplicationStatus::Pending
        //     },
        //     votes_received: 0,
        //     application_date: U64(env::block_timestamp_ms()),
        //     approval_date: if election.auto_approval {
        //         Some(U64(env::block_timestamp_ms()))
        //     } else {
        //         None
        //     },
        // };

        // let candidates_map = self.candidates.get_mut(&election_id).expect("Candidates map not found");
        // candidates_map.insert(applicant, candidate);
        // // self.candidates.insert(election_id, &candidates_map);
    }
    */

    // #[payable]
    // pub fn add_candidates(
    //     &mut self,
    //     election_id: ElectionId,
    //     candidates: Vec<AccountId>
    // ) {
    //     // self.assert_not_paused();
    //     self.assert_admin_or_owner();

    //     let election = self.elections.get(&election_id).expect("Election not found");
    //     let mut candidates_map = self.candidates
    //         .get_mut(&election_id)
    //         .and_then(|map| Some(map.clone()))
    //         .expect("Candidates map not found");

    //     // Track initial storage for refund calculation
    //     let initial_storage = env::storage_usage();

    //     for candidate in candidates {
    //         assert!(
    //             !candidates_map.contains_key(&candidate),
    //             "Candidate {} already exists",
    //             candidate
    //         );

    //         let candidate = Candidate {
    //             account_id: candidate,
    //             status: ApplicationStatus::Approved,
    //             votes_received: 0,
    //             application_date: U64(env::block_timestamp_ms()),
    //         };

    //         candidates_map.insert(candidate.account_id.clone(), candidate);

    //     }

    //     // let candidates_map = self.candidates.get_mut(&election_id).expect("Candidates map not found");
    //     // candidates_map.insert(applicant, candidate);
    //     // // self.candidates.insert(election_id, &candidates_map);

    //     self.candidates.insert(election_id, candidates_map);

    //     // Refund excess deposit
    //     refund_deposit(initial_storage);
    // }

    // pub fn review_application(
    //     &mut self,
    //     election_id: ElectionId,
    //     candidate_id: AccountId,
    //     status: ApplicationStatus,
    // ) {
    //     self.assert_admin_or_owner();
    //     let candidates_map = self.candidates.get_mut(&election_id).expect("Candidates map not found");
    //     let candidate = candidates_map.get_mut(&candidate_id).expect("Candidate not found");

    //     candidate.status = status;
    //     if matches!(candidate.status, ApplicationStatus::Approved) {
    //         candidate.approval_date = Some(U64(env::block_timestamp_ms()));
    //     }
    // }

    pub fn get_elections(&self, from_index: Option<u128>, limit: Option<u128>) -> Vec<Election> {
        let start_index = from_index.unwrap_or_default();
        assert!(
            start_index < self.election_counter as u128,
            "Invalid start index"
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);

        self.elections
            .iter()
            .map(|(_, election)| election.clone())
            .skip(start_index as usize)
            .take(limit)
            .collect()
    }

    pub fn get_election(&self, election_id: &ElectionId) -> Option<Election> {
        self.elections.get(election_id).cloned()
    }

    /// Returns whether an election is currently in the voting period
    pub fn is_voting_period(&self, election_id: &ElectionId) -> bool {
        self.elections.get(election_id).map_or(false, |election| {
            let now = env::block_timestamp_ms();
            now >= election.start_date.0 && now < election.end_date.0
        })
    }

    /// Returns whether an election has ended
    pub fn is_election_ended(&self, election_id: &ElectionId) -> bool {
        self.elections.get(election_id).map_or(false, |election| {
            env::block_timestamp_ms() >= election.end_date.0
        })
    }

    /// Returns all elections created by a specific account
    pub fn get_elections_by_creator(&self, creator: &AccountId) -> Vec<(ElectionId, Election)> {
        self.elections
            .iter()
            .filter(|(_, election)| &election.owner == creator)
            .map(|(id, election)| (*id, election.clone()))
            .collect()
    }

    /// Returns all active elections (in nomination or voting period)
    pub fn get_active_elections(&self) -> Vec<(ElectionId, Election)> {
        let now = env::block_timestamp_ms();
        self.elections
            .iter()
            .filter(|(_, election)| now >= election.start_date.0 && now < election.end_date.0)
            .map(|(id, election)| (*id, election.clone()))
            .collect()
    }

    /// Returns the current phase of an election
    pub fn get_election_phase(&self, election_id: &ElectionId) -> Option<ElectionPhase> {
        self.elections.get(election_id).map(|election| {
            let now = env::block_timestamp_ms();
            if now < election.start_date.0 {
                ElectionPhase::Pending
            } else if now >= election.start_date.0 && now < election.end_date.0 {
                ElectionPhase::Voting
            } else {
                ElectionPhase::Ended
            }
        })
    }

    pub fn get_election_candidates(&self, election_id: ElectionId) -> Vec<Candidate> {
        let candidates_map = self
            .candidates
            .get(&election_id)
            .expect("Election not found");

        candidates_map.values().cloned().collect()
    }

    pub fn get_election_votes(&self, election_id: ElectionId, from_index: Option<u128>, limit: Option<u128>) -> Vec<Vote> {
        let start_index = from_index.unwrap_or_default();
        let votes_map = self.votes.get(&election_id).expect("Election not found");
        assert!(start_index < votes_map.len() as u128, "Invalid start index");
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        let votes_map = self.votes.get(&election_id).expect("Election not found");

        votes_map
            .values()
            .skip(start_index as usize)
            .take(limit)
            .flat_map(|votes| votes.iter().cloned())
            .collect()
    }

    /// Returns time remaining (in nanoseconds) in the current phase
    pub fn get_time_remaining(&self, election_id: &ElectionId) -> Option<u64> {
        self.elections.get(election_id).map(|election| {
            let now = env::block_timestamp_ms();
            if now < election.start_date.0 {
                election.start_date.0 - now
            } else if now < election.end_date.0 {
                election.end_date.0 - now
            } else {
                0
            }
        })
    }
}
