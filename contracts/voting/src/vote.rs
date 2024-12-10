use near_sdk::{Gas, PromiseError, PromiseOrValue, log};
use crate::*;
use crate::ext::{list_contract, XCC_SUCCESS};


#[near(serializers=[borsh, json])]
#[derive(Clone)]
pub struct Vote {
    pub voter: AccountId,
    pub candidate_id: AccountId,
    pub weight: u32,  // Used for both weighted and point-based voting
    pub timestamp: U64,
}


#[near_bindgen]
impl Contract {
    #[payable]
    pub fn vote(&mut self, election_id: ElectionId, vote: (AccountId, u32)) -> PromiseOrValue<bool> {
        let election = self.elections.get(&election_id).cloned().expect("Election not found");
        assert!(
            env::block_timestamp() >= election.start_date.0 &&
                env::block_timestamp() <= election.end_date.0,
            "Voting period is not active"
        );

        let voter = env::predecessor_account_id();
        let result = self.assert_voter_eligible(&election, &voter, vote.clone());
        log_vote_event(election_id, vote);
        return result
    }

    fn record_vote(&mut self, voter: &AccountId, candidate_id: &AccountId, weight: u32, election_id: &ElectionId) {
        let vote = Vote {
            voter: voter.clone(),
            candidate_id: candidate_id.clone(),
            weight,
            timestamp: U64(env::block_timestamp()),
        };
        let election_votes = self.votes.get_mut(&election_id).expect("Votes not found for election");

        let voter_votes = election_votes.entry(voter.clone()).or_insert_with(Vec::new);
        voter_votes.push(vote);

        let candidates_map = self.candidates.get_mut(&election_id).expect("Candidates map not found");
        let candidate = candidates_map.get_mut(candidate_id).expect("Candidate not found");
        candidate.votes_received += weight as u64;

    }


    pub(crate) fn assert_voter_eligible(&mut self, election: &Election, voter: &AccountId, votes: (AccountId, u32)) -> PromiseOrValue<bool> {
        match &election.voter_eligibility {
            EligibilityType::Open => {
                PromiseOrValue::Value(
                    self.handle_voting(voter, election, votes)
                )
            },
            EligibilityType::ListBased(contract_id, list_id) => {
                let promise = list_contract::ext(contract_id.clone())
                    .with_static_gas(Gas::from_tgas(5))
                    .is_registered(Some(list_id.0 as u64), voter.clone());
                return PromiseOrValue::Promise(promise.then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas::from_tgas(XCC_SUCCESS))
                        .eligible_voting_callback(voter, election, votes)
                ))
            },
            EligibilityType::TokenBased(_token_contract, _min_balance) => {
                unimplemented!()
                // env::log_str("Token-based eligibility check not fully implemented");
                // PromiseOrValue::Value(true)
            },
            EligibilityType::Custom(_contract_addr) => {
                unimplemented!()
                // env::log_str("Custom eligibility check not fully implemented");
                // PromiseOrValue::Value(true)
            },
        }
    }

    pub fn handle_voting(
        &mut self,
        voter: &AccountId,
        election: &Election,
        votes: (AccountId, u32)  // (candidate_id, weight)
    ) -> bool {
        let election_votes = self.votes.get(&election.id).expect("Votes not found for election");
        
        // Check if voter has already voted
        if let Some(existing_votes) = election_votes.get(voter) {
            // Check votes_per_voter limit
            assert!(
                existing_votes.len() as u32 + 1 <= election.votes_per_voter,
                "Used up allowed number of votes per voter: {} + {} > {}",
                existing_votes.len(),
                1,
                election.votes_per_voter
            );

            // Check for duplicate candidate votes
            assert!(
                !existing_votes.iter().any(|v| v.candidate_id == votes.0),
                "Cannot vote for the same candidate twice"
            );
        }

        // Check voting type constraints
        match election.voting_type {
            VotingType::Simple => {
                // For simple voting, weight must be 1
                assert!(votes.1 == 1, "Simple voting only allows weight of 1");
            },
            VotingType::Weighted(max_weight) => {
                // For weighted voting, check single vote weight limit
                assert!(
                    votes.1 <= max_weight,
                    "Vote weight exceeds maximum allowed: {} > {}", 
                    votes.1,
                    max_weight
                );
            }
        }
        
        // Record the vote
        self.record_vote(voter, &votes.0, votes.1, &election.id);
        
        true
    }

    #[private]
    pub fn eligible_voting_callback(
        &mut self,
        voter: &AccountId,
        election: &Election,
        votes: (AccountId, u32),
        #[callback_result] call_result: Result<bool, PromiseError>,
    ) -> bool {
        if call_result.is_err() {
            log!("There was an error checking eligibility");
            return false;
        }
        // if call_result.unwrap().
        self.handle_voting(voter, election, votes)

    }

    // increase votes_per_voter
    pub fn increase_votes_per_voter(&mut self, election_id: &ElectionId, amount: u32) {
        let election = self.elections.get_mut(election_id).expect("Election not found");
        election.votes_per_voter += amount;
    }

    /// Returns all votes cast by a specific voter in a given election
    pub fn get_voter_votes(&self, election_id: &ElectionId, voter: &AccountId) -> Option<Vec<Vote>> {
        self.votes
            .get(election_id)
            .and_then(|election_votes| election_votes.get(voter))
            .cloned()
    }

    pub fn get_candidate_votes(&self, election_id: ElectionId, candidate_id: AccountId) -> Vec<Vote> {
        let votes_map = self.votes
            .get(&election_id)
            .expect("Election not found");
        
        votes_map.values()
            .flat_map(|votes| votes.iter().cloned())
            .filter(|vote| vote.candidate_id == candidate_id)
            .collect()
    }

    pub fn get_candidate_vote_count(&self, election_id: ElectionId, candidate_id: AccountId) -> u32 {
        let votes = self.get_candidate_votes(election_id, candidate_id);
        votes.iter().map(|vote| vote.weight).sum()
    }

    pub fn get_election_results(&self, election_id: ElectionId) -> Vec<(AccountId, u32)> {
        let candidates_map = self.candidates
            .get(&election_id)
            .expect("Election not found");
        
        candidates_map
            .keys()
            .map(|candidate_id| {
                let vote_count = self.get_candidate_vote_count(election_id, candidate_id.clone());
                (candidate_id.clone(), vote_count)
            })
            .collect()
    }

    /// Returns the total number of votes cast in an election
    pub fn get_election_vote_count(&self, election_id: &ElectionId) -> u64 {
        self.votes
            .get(election_id)
            .map(|election_votes| {
                election_votes
                    .values()
                    .flat_map(|votes| votes.iter())
                    .count() as u64
            })
            .unwrap_or(0)
    }

    /// Returns the total weight/points received by a candidate
    pub fn get_candidate_vote_weight(&self, election_id: &ElectionId, candidate_id: &AccountId) -> u64 {
        self.candidates
            .get(election_id)
            .and_then(|candidates| candidates.get(candidate_id))
            .map(|candidate| candidate.votes_received)
            .unwrap_or(0)
    }

    /// Returns whether a voter has participated in an election
    pub fn has_voter_participated(&self, election_id: &ElectionId, voter: &AccountId) -> bool {
        self.votes
            .get(election_id)
            .and_then(|election_votes| election_votes.get(voter))
            .map(|votes| !votes.is_empty())
            .unwrap_or(false)
    }

    /// Returns the remaining votes/points available for a voter in an election
    pub fn get_voter_remaining_capacity(&self, election_id: &ElectionId, voter: &AccountId) -> Option<u32> {
        let election = self.elections.get(election_id)?;
        let current_votes = self.get_voter_votes(election_id, voter).unwrap_or_default();

        match election.voting_type {
            VotingType::Simple => {
                Some(election.votes_per_voter.saturating_sub(current_votes.len() as u32))
            },
            VotingType::Weighted(max_weight) => {
                let used_weight: u32 = current_votes.iter().map(|v| v.weight).sum();
                Some(max_weight.saturating_sub(used_weight))
            },
        }
    }
}