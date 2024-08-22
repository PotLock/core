mod test_envs;

use anyhow::Result;
use log::info;
use near_sdk::env::{block_timestamp, block_timestamp_ms};
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use near_workspaces::result::ExecutionFinalResult;

use chrono::Utc;
use serde_json::json;
use test_envs::{init, init_logger};

use near_workspaces::{sandbox, types::NearToken, Account, AccountId, Block, Contract};

type TimestampMs = u64;
const ONE_NEAR: NearToken = NearToken::from_near(1);

async fn create_campaign(
    contract: &Contract,
    caller: Account,
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
) -> Result<ExecutionFinalResult, near_workspaces::error::Error> {
    let res = caller
        .call(contract.id(), "create_campaign")
        .args_json((
            name,
            description,
            cover_image_url,
            recipient,
            start_ms,
            end_ms,
            ft_id,
            target_amount,
            min_amount,
            max_amount,
            referral_fee_basis_points,
            creator_fee_basis_points,
            allow_fee_avoidance,
        ))
        .max_gas()
        // .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .deposit(ONE_NEAR.into())
        .transact()
        .await;
    return res;
}

// make an update_campaign function that takes contract and params and calls the update_campaign function

async fn update_campaign(
    contract: &Contract,
    campaign_id: Option<u64>,
    name: Option<String>,
    description: Option<String>,
    cover_image_url: Option<String>,
    recipient: Option<AccountId>,
    start_ms: Option<TimestampMs>,
    end_ms: Option<TimestampMs>,
    ft_id: Option<AccountId>,
    target_amount: Option<U128>,
    min_amount: Option<U128>,
    max_amount: Option<U128>,
    referral_fee_basis_points: Option<u32>,
    creator_fee_basis_points: Option<u32>,
    allow_fee_avoidance: Option<bool>,
) -> Result<ExecutionFinalResult, near_workspaces::error::Error> {
    let res = contract
        .call("update_campaign")
        .args_json(json!({
            "campaign_id": campaign_id,
            "name": name,
            "description": description,
            "cover_image_url": cover_image_url,
            "recipient": recipient,
            "start_ms": start_ms,
            "end_ms": end_ms,
            "ft_id": ft_id,
            "target_amount": target_amount,
            "min_amount": min_amount,
            "max_amount": max_amount,
            "referral_fee_basis_points": referral_fee_basis_points,
            "creator_fee_basis_points": creator_fee_basis_points,
            "allow_fee_avoidance": allow_fee_avoidance,
        }
        ))
        .max_gas()
        // .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .deposit(ONE_NEAR)
        .transact()
        .await;
    return res;
}

#[tokio::test]
async fn test_create_campaign() -> Result<()> {
    init_logger();
    // let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    let name = "Test Campaign".to_string();
    let description = Some("Test Description".to_string());
    let cover_image_url = Some("https://example.com/image.jpg".to_string());
    let recipient = bob.id().clone();
    let start_ms = near_sdk::env::block_timestamp() + 1000;
    let end_ms = Some(start_ms + 10_000);
    let ft_id = None;
    let target_amount = U128::from(100);
    let min_amount = Some(U128::from(10));
    let max_amount = Some(U128::from(200));
    let referral_fee_basis_points = Some(100);
    let creator_fee_basis_points = Some(100);
    let allow_fee_avoidance = Some(true);

    let res = create_campaign(
        &contract,
        alice.clone(),
        name.clone(),
        description.clone(),
        cover_image_url.clone(),
        recipient.clone(),
        start_ms,
        end_ms,
        ft_id,
        target_amount,
        min_amount,
        max_amount,
        referral_fee_basis_points,
        creator_fee_basis_points,
        allow_fee_avoidance,
    )
    .await?;
    // Ensure the transaction succeeded
    assert!(res.is_success());

    // Extract the execution outcome
    let logs = res.logs();
    let campaign_create_log = logs
        .iter()
        .find(|log| log.contains("campaign_create"))
        .expect("Campaign creation log not found");
    let event_json_start = campaign_create_log.find("EVENT_JSON:").unwrap() + "EVENT_JSON:".len();
    let event_json_str = &campaign_create_log[event_json_start..];
    let event_json: serde_json::Value = serde_json::from_str(event_json_str)?;

    // Verify that the details of the new campaign match the input
    let campaign_data = &event_json["data"][0]["campaign"];
    assert_eq!(campaign_data["name"], name);
    match description {
        Some(description_value) => assert_eq!(campaign_data["description"], description_value),
        None => assert!(campaign_data.get("description").is_none()),
    }
    match cover_image_url {
        Some(cover_image_url_value) => {
            assert_eq!(campaign_data["cover_image_url"], cover_image_url_value)
        }
        None => assert!(campaign_data.get("cover_image_url").is_none()),
    }
    assert_eq!(campaign_data["recipient"], recipient.to_string());
    assert_eq!(campaign_data["start_ms"], start_ms);
    match end_ms {
        Some(end_ms_value) => assert_eq!(campaign_data["end_ms"], end_ms_value),
        None => assert!(campaign_data.get("end_ms").is_none()),
    }
    let target_amount_str = campaign_data["target_amount"]
        .as_str()
        .expect("target_amount should be a string");
    let target_amount_json = U128::from(
        target_amount_str
            .parse::<u128>()
            .expect("Invalid U128 string"),
    );
    assert_eq!(target_amount_json, target_amount);
    match min_amount {
        Some(min_amount_value) => {
            let min_amount_str = campaign_data["min_amount"]
                .as_str()
                .expect("min_amount should be a string");
            let min_amount_json =
                U128::from(min_amount_str.parse::<u128>().expect("Invalid U128 string"));
            assert_eq!(min_amount_json, min_amount_value);
        }
        None => assert!(campaign_data.get("min_amount").is_none()),
    }
    match max_amount {
        Some(max_amount_value) => {
            let max_amount_str = campaign_data["max_amount"]
                .as_str()
                .expect("max_amount should be a string");
            let max_amount_json =
                U128::from(max_amount_str.parse::<u128>().expect("Invalid U128 string"));
            assert_eq!(max_amount_json, max_amount_value);
        }
        None => assert!(campaign_data.get("max_amount").is_none()),
    }
    match referral_fee_basis_points {
        Some(referral_fee_basis_points_value) => {
            assert_eq!(
                campaign_data["referral_fee_basis_points"],
                referral_fee_basis_points_value
            )
        }
        None => assert!(campaign_data.get("referral_fee_basis_points").is_none()),
    }
    match creator_fee_basis_points {
        Some(creator_fee_basis_points_value) => {
            assert_eq!(
                campaign_data["creator_fee_basis_points"],
                creator_fee_basis_points_value
            )
        }
        None => assert!(campaign_data.get("creator_fee_basis_points").is_none()),
    }

    // test update_campaign name and dexcription

    let new_name = "New Test Campaign".to_string();
    // let new_description = Some("New Test Description".to_string());
    // let new_cover_image_url = Some("https://example.com/new_image.jpg".to_string());
    // let new_recipient = bob.id().clone();
    // let new_start_ms = near_sdk::env::block_timestamp() + 1000;
    // let new_end_ms = Some(new_start_ms + 10_000);
    // let new_ft_id = None;
    // let new_target_amount = U128::from(100);
    // let new_min_amount = Some(U128::from(10));
    // let new_max_amount = Some(U128::from(200));
    // let new_referral_fee_basis_points = Some(100);
    // let new_creator_fee_basis_points = Some(100);
    // let new_allow_fee_avoidance = Some(true);

    let campaign_id = campaign_data["id"].as_u64().unwrap();

    // Update campaign
    let update_result = alice
        .call(contract.id(), "update_campaign")
        .args_json(json!({
            "campaign_id": campaign_id,
            "name": "Updated Test Campaign",
            "description": "Updated Test Description",
        }))
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await?;
    // Ensure the transaction succeeded
    assert!(update_result.is_success());
    println!("Update.. sucess >>>");

    // Verify update
    let updated_campaign: serde_json::Value = contract
        .call("get_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;
    assert_eq!(updated_campaign["name"], "Updated Test Campaign");
    assert_eq!(updated_campaign["description"], "Updated Test Description");

    Ok(())
}

#[tokio::test]
async fn test_donate_to_campaign_with_target() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    let now = Utc::now().timestamp_millis();
    let a_minute_from_now = (now + 60);

    // Create campaign with target
    println!(
        "check bob bal.... {:?}, >> {:?}",
        bob.view_account().await?.balance,
        bob.id()
    );
    let name = "Test Campaign".to_string();
    let description = Some("Test Description".to_string());
    let cover_image_url = Some("https://example.com/image.jpg".to_string());
    let recipient = bob.id().clone();
    let start_ms = now as u64;
    let end_ms = Some(start_ms + 10_000);
    let ft_id = None;
    let target_amount = U128::from(3000000000000000000000000);
    let min_amount = Some(U128::from(3000000000000000000000000));
    let max_amount = Some(U128::from(5000000000000000000000000));
    let referral_fee_basis_points = Some(100);
    let creator_fee_basis_points = Some(100);
    let allow_fee_avoidance = Some(true);
    let res = create_campaign(
        &contract,
        alice.clone(),
        name.clone(),
        description.clone(),
        cover_image_url.clone(),
        recipient.clone(),
        start_ms,
        end_ms,
        ft_id,
        target_amount,
        min_amount,
        max_amount,
        referral_fee_basis_points,
        creator_fee_basis_points,
        allow_fee_avoidance,
    )
    .await?;

    let logs = res.logs();
    let campaign_create_log = logs
        .iter()
        .find(|log| log.contains("campaign_create"))
        .expect("Campaign creation log not found");
    let event_json_start = campaign_create_log.find("EVENT_JSON:").unwrap() + "EVENT_JSON:".len();
    let event_json_str = &campaign_create_log[event_json_start..];
    let event_json: serde_json::Value = serde_json::from_str(event_json_str)?;

    // Verify that the details of the new campaign match the input
    let campaign_data = &event_json["data"][0]["campaign"];
    let campaign_id = campaign_data["id"].as_u64().unwrap();
    println!("campaign_id: {:?}", campaign_data);

    // Donate to campaign
    let donation_amount = NearToken::from_near(2);

    donate_to_campaign(&alice, &contract, campaign_id, donation_amount).await?;

    // Verify donation
    let campaign: serde_json::Value = contract
        .call("get_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;


    assert_eq!(
        campaign["total_raised_amount"],
        donation_amount.as_yoctonear().to_string()
    );
    // assert!(campaign["status"] == "ONGOING" || campaign["status"] == "COMPLETED");

    let donate_result2 = bob
        .call(contract.id(), "donate")
        .args_json(json!({
            "campaign_id": campaign_id,
        }))
        .max_gas()
        .deposit(donation_amount)
        .transact()
        .await?;

    assert!(donate_result2.is_success());

    // println!("check bob bal after finsali.... {:?}, >> {:?}", bob.view_account().await?.balance, alice.view_account().await?.balance);

    let campaign2: serde_json::Value = bob
        .call(contract.id(), "get_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;

    println!(
        "campaign campana: {:?}, {}",
        campaign2["min_amount"],
        campaign2["total_raised_amount"]
    );

    assert_eq!(
        campaign2["total_raised_amount"],
        donation_amount
            .checked_add(donation_amount)
            .unwrap()
            .as_yoctonear()
            .to_string()
    );
    // get donations for campaign
    let campaign_donations: serde_json::Value = contract
        .call("get_donations_for_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;

    assert_eq!(campaign_donations.as_array().unwrap().len(), 2);

    // processescrowed donations by calling the `process_escrowed_donations_batch` function

    let process_escrowed_donations_batch_result = contract
        .call("process_escrowed_donations_batch")// call with campaign id
        .args_json(json!({ "campaign_id": campaign_id }))
        .max_gas()
        .transact()
        .await?;

    println!("Go berserk.... {:?}", process_escrowed_donations_batch_result);
    assert!(process_escrowed_donations_batch_result.is_success());

    let campaign_donations2: serde_json::Value = contract
        .call("get_donations_for_donor")
        .args_json(json!({ "donor_id": alice.id() }))
        .view()
        .await?
        .json()?;

    println!(
        "campaign donations 22lejo: {:?}, {}",
        campaign_donations2, campaign_id
    );


    Ok(())
}


#[tokio::test]
async fn test_campaign_refunds_when_target_not_met() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    // Create a campaign with a target that won't be met
    let now = Utc::now().timestamp_millis() as u64;
    println!("SHOYUT PLSSS... {}",now);
    let campaign_duration = 10_000; // 10 seconds
    let campaign_id = create_test_campaign(&contract, &alice, now, campaign_duration, U128::from(10_000_000_000_000_000_000_000_000)).await?;

    // Make donations from Alice and Bob
    let alice_donation = NearToken::from_near(1);
    let bob_donation = NearToken::from_near(2);

    let alice_initial_balance = alice.view_account().await?.balance;
    let bob_initial_balance = bob.view_account().await?.balance;

    donate_to_campaign(&alice, &contract, campaign_id, alice_donation).await?;
    donate_to_campaign(&bob, &contract, campaign_id, bob_donation).await?;

    // Verify donations were made
    let campaign: serde_json::Value = get_campaign(&contract, campaign_id).await?;
    assert_eq!(
        campaign["total_raised_amount"],
        (alice_donation.saturating_add(bob_donation)).as_yoctonear().to_string()
    );

    // Wait for the campaign to end
    worker.fast_forward(100).await?;

    // Process refunds
    process_refunds(&contract, campaign_id).await?;

    // Verify refunds were processed
    let alice_final_balance = alice.view_account().await?.balance;
    let bob_final_balance = bob.view_account().await?.balance;

    // Check if Alice and Bob received their refunds (minus gas fees)
    assert!(alice_final_balance > alice_initial_balance.saturating_sub(NearToken::from_near(1)));
    assert!(bob_final_balance > bob_initial_balance.saturating_sub(NearToken::from_near(1)));

    // Verify campaign state after refunds
    let campaign_after_refund: serde_json::Value = get_campaign(&contract, campaign_id).await?;
    assert_eq!(campaign_after_refund["escrow_balance"], "0");

    // Check if donations are marked as refunded
    let campaign_donations: Vec<serde_json::Value> = get_campaign_donations(&contract, campaign_id).await?;
    // for donation in campaign_donations {
    //     assert!(donation["returned_at_ms"].is_string());
    // }

    Ok(())
}

async fn create_test_campaign(
    contract: &Contract,
    creator: &Account,
    start_ms: u64,
    duration: u64,
    target_amount: U128,
) -> Result<u64> {
    let res = create_campaign(
        contract,
        creator.clone(),
        "Test Refund Campaign".to_string(),
        Some("Campaign for testing refunds".to_string()),
        None,
        creator.id().clone(),
        start_ms,
        Some(start_ms + duration),
        None,
        target_amount,
        Some(target_amount),
        None,
        Some(100),
        Some(100),
        Some(true),
    )
        .await?;

    let campaign_id = extract_campaign_id_from_logs(&res)?;
    Ok(campaign_id)
}

async fn donate_to_campaign(
    donor: &Account,
    contract: &Contract,
    campaign_id: u64,
    amount: NearToken,
) -> Result<()> {
    let donate_result = donor
        .call(contract.id(), "donate")
        .args_json(json!({
            "campaign_id": campaign_id,
        }))
        .max_gas()
        .deposit(amount)
        .transact()
        .await?;
    println!("less DONAT SUMMON!... {:?}", donate_result);
    assert!(donate_result.is_success());
    Ok(())
}

async fn process_refunds(contract: &Contract, campaign_id: u64) -> Result<()> {
    let process_refunds_result = contract
        .call("process_refunds_batch")
        .args_json(json!({ "campaign_id": campaign_id }))
        .max_gas()
        .transact()
        .await?;
    println!("less havit... {:?}", process_refunds_result);
    assert!(process_refunds_result.is_success());
    Ok(())
}

async fn get_campaign(contract: &Contract, campaign_id: u64) -> Result<serde_json::Value> {
    let campaign: serde_json::Value = contract
        .call("get_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;
    Ok(campaign)
}

async fn get_campaign_donations(contract: &Contract, campaign_id: u64) -> Result<Vec<serde_json::Value>> {
    let campaign_donations: Vec<serde_json::Value> = contract
        .call("get_donations_for_campaign")
        .args_json(json!({ "campaign_id": campaign_id }))
        .view()
        .await?
        .json()?;
    Ok(campaign_donations)
}

fn extract_campaign_id_from_logs(res: &ExecutionFinalResult) -> Result<u64> {
    let logs = res.logs();
    let campaign_create_log = logs
        .iter()
        .find(|log| log.contains("campaign_create"))
        .expect("Campaign creation log not found");
    let event_json_start = campaign_create_log.find("EVENT_JSON:").unwrap() + "EVENT_JSON:".len();
    let event_json_str = &campaign_create_log[event_json_start..];
    let event_json: serde_json::Value = serde_json::from_str(event_json_str)?;
    let campaign_id = event_json["data"][0]["campaign"]["id"]
        .as_u64()
        .expect("Failed to extract campaign ID");
    Ok(campaign_id)
}

