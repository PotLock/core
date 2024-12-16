use anyhow::Result;
use chrono::Utc;
use near_sdk::json_types::{U128, U64};
use near_sdk::AccountId;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::NearToken;
use near_workspaces::{sandbox, Account, Contract, Worker};
use serde_json::{json, Value};
const ONE_NEAR: NearToken = NearToken::from_near(1);

// Helper struct for test environment
struct TestEnv {
    worker: Worker<Sandbox>,
    contract: Contract,
    owner: Account,
    admin: Account,
    voter: Account,
    candidate: Account,
}

// Helper functions for common contract calls
async fn create_election(
    contract: &Contract,
    caller: Account,
    title: String,
    description: String,
    start_date: U64,
    end_date: U64,
    votes_per_voter: u32,
    voter_eligibility: String,
    voting_type: String,
    election_type: Value,
    candidates: Vec<AccountId>,
) -> Result<ExecutionFinalResult, near_workspaces::error::Error> {
    // Convert string inputs to proper enum types
    println!("Creating election with candidates: {:?}", election_type);

    caller
        .call(contract.id(), "create_election")
        .args_json(json!({
            "title": title,
            "description": description,
            "start_date": start_date,
            "end_date": end_date,
            "votes_per_voter": votes_per_voter,
            "voter_eligibility": voter_eligibility,
            "voting_type": voting_type,
            "election_type": election_type,
            "candidates": candidates
        }))
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
}

async fn apply_as_candidate(
    contract: &Contract,
    caller: Account,
    election_id: u64,
) -> Result<ExecutionFinalResult, near_workspaces::error::Error> {
    caller
        .call(contract.id(), "apply")
        .args_json(json!({
            "election_id": election_id
        }))
        .max_gas()
        .transact()
        .await
}

async fn vote_in_election(
    contract: &Contract,
    caller: Account,
    election_id: u64,
    vote: (AccountId, u32),
) -> Result<ExecutionFinalResult, near_workspaces::error::Error> {
    caller
        .call(contract.id(), "vote")
        .args_json(json!({
            "election_id": election_id,
            "vote": vote
        }))
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await
}

// Initialize test environment
async fn init() -> anyhow::Result<TestEnv> {
    let worker = sandbox().await?;

    let owner = worker.dev_create_account().await?;
    let contract = worker
        .dev_deploy(include_bytes!(
            "../../target/near/voting_contract/voting_contract.wasm"
        ))
        .await?;
    let res = contract
        .call("new")
        .args_json(json!({
            "owner_id": owner.id()
        }))
        .max_gas()
        .transact()
        .await?;
    println!("Contract deployed: {:?}", res);
    assert!(res.is_success());

    let admin = worker.dev_create_account().await?;
    let voter = worker.dev_create_account().await?;
    let candidate = worker.dev_create_account().await?;

    Ok(TestEnv {
        worker,
        contract,
        owner,
        admin,
        voter,
        candidate,
    })
}

/*
#[tokio::test]
async fn test_create_elections() -> anyhow::Result<()> {
    let env = init().await?;

    let now = near_sdk::env::block_timestamp();

    // Create multiple elections
    let elections = vec![
        ("Election 1", "First test election"),
        ("Election 2", "Second test election"),
        ("Election 3", "Third test election"),
        ("Election 4", "Fourth test election"),
    ];

    for (title, description) in elections {
        let res = create_election(
            &env.contract,
            env.owner.clone(),
            title.to_string(),
            description.to_string(),
            U64(now + 1_000_000),
            U64(now + 2_000_000),
            1,
            "Open".to_string(),
            "Simple".to_string(),
            "GeneralElection".to_string(),
            vec![env.candidate.id().clone()]
        )
        .await?
        .into_result()?;

        println!("Election created: {:?}", res);

        // assert!(res.is_success());
    }

    // Verify elections were created
    let owner_elections: serde_json::Value = env.contract
        .view("get_elections")
        .args_json(json!({
            "from_index": 0,
            "limit": 10
        }))
        .await?
        .json()?;

    let elections = owner_elections.as_array().unwrap();
    assert_eq!(elections.len(), 4);

    Ok(())
}
*/

#[tokio::test]
async fn test_full_election_flow() -> anyhow::Result<()> {
    let env = init().await?;

    let now = Utc::now().timestamp() as u64;
    let now_in_nanos = now * 1_000_000_000;

    // 1. Create election with Pot type
    let res = create_election(
        &env.contract,
        env.owner.clone(),
        "Test Pot Election".to_string(),
        "Test Description".to_string(),
        U64((now + 20) * 1_000), // voting starts in 20s
        U64((now + 90) * 1_000), // voting ends in 90s
        1,
        "Open".to_string(),
        "Simple".to_string(),
        json!({ "Pot": env.owner.id() }), // Pot type with owner as pot account
        vec![env.candidate.id().clone(), env.admin.id().clone()], // Initial candidates
    )
    .await?;

    println!("Election created: {:?}", res);
    assert!(res.is_success());
    let election_id: u64 = res.json()?;

    // 2. Verify initial candidates
    let candidates: serde_json::Value = env
        .contract
        .view("get_election_candidates")
        .args_json(json!({
            "election_id": election_id
        }))
        .await?
        .json()?;

    let candidates_array = candidates.as_array().unwrap();
    assert_eq!(
        candidates_array.len(),
        2,
        "Should have 2 initial candidates"
    );

    // Advance to voting period
    env.worker.fast_forward(200).await?;

    // 3. Cast votes from multiple voters
    let voters = vec![env.voter.clone(), env.owner.clone()];
    let mut vote_count = 0;

    for voter in voters {
        let vote_result = vote_in_election(
            &env.contract,
            voter,
            election_id,
            (env.candidate.id().clone(), 1),
        )
        .await?;
        assert!(vote_result.is_success());
        vote_count += 1;
    }

    // 4. Verify votes
    let election_votes: serde_json::Value = env
        .contract
        .view("get_election_votes")
        .args_json(json!({
            "election_id": election_id
        }))
        .await?
        .json()?;

    let votes = election_votes.as_array().unwrap();
    assert_eq!(
        votes.len(),
        vote_count,
        "Should have correct number of votes"
    );

    // 5. Verify candidate vote counts
    let updated_candidates: serde_json::Value = env
        .contract
        .view("get_election_candidates")
        .args_json(json!({
            "election_id": election_id
        }))
        .await?
        .json()?;

    let winning_candidate = updated_candidates
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["account_id"] == env.candidate.id().to_string())
        .unwrap();

    assert_eq!(
        winning_candidate["votes_received"], vote_count,
        "Winning candidate should have all votes"
    );

    Ok(())
}

/*
#[tokio::test]
async fn test_weighted_voting_flow() -> anyhow::Result<()> {
    let env = init().await?;

    let now = Utc::now().timestamp() as u64;
    let now_in_nanos = now * 1_000_000_000;

    // 1. Create election with weighted voting
    let res = create_election(
        &env.contract,
        env.owner.clone(),
        "Weighted Vote Test".to_string(),
        "Testing weighted voting mechanism".to_string(),
        U64((now + 20) * 1_000_000_000),  // voting starts in 20s
        U64((now + 90) * 1_000_000_000),  // voting ends in 90s
        3,                                 // allow multiple votes per voter
        "Open".to_string(),
        "Weighted(5)".to_string(),        // max weight of 5 per vote
        "GeneralElection".to_string(),
        vec![env.candidate.id().clone()]
    )
    .await?;

    println!("Weighted election created: {:?}", res);
    assert!(res.is_success());
    let election_id: u64 = res.json()?;

    // 2. Apply multiple candidates
    let candidates = vec![
        (&env.candidate, "First Candidate"),
        (&env.admin, "Second Candidate"),    // Using admin account as second candidate
    ];

    for (candidate, description) in candidates {
        let apply_result = apply_as_candidate(
            &env.contract,
            candidate.clone(),
            election_id,
        )
        .await?;
        println!("Candidate applied: {:?}", apply_result);
        assert!(apply_result.is_success());
    }

    // Advance to voting period
    println!("Fast forwarding by 200 epochs");
    env.worker.fast_forward(200).await?;

    // 3. Cast weighted votes
    let weighted_votes = vec![
        (env.candidate.id().clone(), 3),  // Vote for first candidate with weight 3
        (env.admin.id().clone(), 2),      // Vote for second candidate with weight 2
    ];

    // Test voting with different weights
    for vote in weighted_votes {
        let vote_result = vote_in_election(
            &env.contract,
            env.voter.clone(),
            election_id,
            vote,
        )
        .await?;
        println!("Weighted vote cast: {:?}", vote_result);
        assert!(vote_result.is_success());
    }

    // 4. Verify the votes were recorded correctly
    let election_votes: serde_json::Value = env.contract
        .view("get_election_votes")
        .args_json(json!({
            "election_id": election_id,
            "voter_id": env.voter.id()
        }))
        .await?
        .json()?;

    println!("Election votes: {:?}", election_votes);

    // 5. Try to vote with weight exceeding maximum (should fail)
    let exceed_weight_result = vote_in_election(
        &env.contract,
        env.voter.clone(),
        election_id,
        (env.candidate.id().clone(), 6),  // Weight of 6 exceeds max of 5
    )
    .await;

    // This should fail due to exceeding max weight
    assert!(exceed_weight_result.is_err() || !exceed_weight_result?.is_success());

    Ok(())
}
*/
