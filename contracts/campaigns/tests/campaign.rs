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
async fn test_create_campaigns() -> Result<()> {
    init_logger();
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    // let _ = contract.as_account().transfer_near(alice.id(), NearToken::from_yoctonear(99999481833998144000000000)).await?;

    let state_p_funds = contract.view_account().await?;
    println!(
        "state before creation of election.... : {:?}",
        state_p_funds
    );

    // Create first campaign
    let name1 = "Test Campaign 1".to_string();
    let description1 = Some("Test Description 1".to_string());
    let cover_image_url1 = Some("https://example.com/image1.jpg".to_string());
    let recipient = bob.id().clone();
    let now = Utc::now().timestamp_millis();
    let start_ms = (now + 1000_000) as u64;
    let end_ms = Some(start_ms + 10_000_000);
    let target_amount1 = U128::from(100);

    // Create second campaign
    let name2 = "Test Campaign 2".to_string();
    let description2 = Some("Test Description 2".to_string());
    let cover_image_url2 = Some("https://example.com/image2.jpg".to_string());
    let target_amount2 = U128::from(200);

    // Create third campaign
    let name3 = "Test Campaign 3".to_string();
    let description3 = Some("Test Description 3".to_string());
    let cover_image_url3 = Some("https://example.com/image3.jpg".to_string());
    let target_amount3 = U128::from(300);

    // Create all three campaigns
    let campaigns = vec![
        (name1, description1, cover_image_url1, target_amount1),
        (name2, description2, cover_image_url2, target_amount2),
        (name3, description3, cover_image_url3, target_amount3),
    ];

    for (name, description, cover_image_url, target_amount) in campaigns {
        let res = create_campaign(
            &contract,
            alice.clone(),
            name,
            description,
            cover_image_url,
            recipient.clone(),
            start_ms,
            end_ms,
            None, // ft_id
            target_amount,
            Some(U128::from(10)),
            Some(U128::from(1000)),
            Some(100),
            Some(100),
            Some(true),
        )
        .await?;
        println!("multui creator.. {:?}", res);
        assert!(res.is_success());
    }

    // Get all campaigns for alice
    let owner_campaigns: serde_json::Value = alice
        .view(contract.id(), "get_campaigns_by_owner")
        .args_json(json!({
            "owner_id": alice.id(),
            "from_index": 0,
            "limit": 10
        }))
        .await?
        .json()?;

    println!("Owner campaigns: {:?}", owner_campaigns);

    // Assert we got all three campaigns
    assert_eq!(
        owner_campaigns.as_array().unwrap().len(),
        3,
        "Expected 3 campaigns, got {}",
        owner_campaigns.as_array().unwrap().len()
    );

    // Verify each campaign has unique details
    let campaigns = owner_campaigns.as_array().unwrap();
    assert!(campaigns
        .iter()
        .any(|c| c["name"].as_str().unwrap().contains("1")));
    assert!(campaigns
        .iter()
        .any(|c| c["name"].as_str().unwrap().contains("2")));
    assert!(campaigns
        .iter()
        .any(|c| c["name"].as_str().unwrap().contains("3")));

    Ok(())
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
    let now = Utc::now().timestamp_millis();
    let start_ms = (now + 10000) as u64;
    let end_ms = Some(start_ms + 10_0000);
    let ft_id = None;
    let target_amount = U128::from(100);
    let min_amount = Some(U128::from(10));
    let max_amount = Some(U128::from(200));
    let referral_fee_basis_points = Some(100);
    let creator_fee_basis_points = Some(100);
    let allow_fee_avoidance = Some(true);

    // let _ = contract.as_account().transfer_near(alice.id(), NearToken::from_yoctonear(95736646527937956500000000)).await?;

    let state_p_funds = contract.view_account().await?;
    println!(
        "state before creation of campaign.... : {:?}",
        state_p_funds
    );

    let res = create_campaign(
        &contract,
        alice.clone(),
        name.clone(),
        description.clone(),
        cover_image_url.clone(),
        recipient.clone(),
        start_ms,
        end_ms,
        ft_id.clone(),
        target_amount,
        min_amount,
        max_amount,
        referral_fee_basis_points,
        creator_fee_basis_points,
        allow_fee_avoidance,
    )
    .await?;

    println!("created camoaing... {:?}", res);
    // Ensure the transaction succeeded
    assert!(res.is_success());

    let state = contract.view_account().await?;

    println!("state after creating of campaign.... : {:?}", state);

    let storage_used = state.storage_usage - state_p_funds.storage_usage;

    println!("total used.... : {}", storage_used);

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

    let vcp = alice
        .view(contract.id(), "get_campaigns_by_recipient")
        .args_json(json!({
            "recipient_id": recipient.clone(),
        }))
        .await?;
    println!("campaign viewed: {:?}", vcp.json::<serde_json::Value>()?);

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
        .args_json(json!({"campaign_id": 1,"name": "Updated Test Campaign","description": "Updated Test Description", "cover_image_url": "https://fifi.png", "max_amount": "100000000000", "target_amount": "1000000000"}))
        .max_gas()
        .deposit(ONE_NEAR)
        .transact()
        .await?;
    // Ensure the transaction succeeded
    println!("Update.. sucess >>> {:?}", update_result);
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

    let state3 = contract.view_account().await?;

    println!("state after creating of all.... : {:?}", state);

    let storage_used = state3.storage_usage - state_p_funds.storage_usage;

    println!("total used all2geda.... : {}", storage_used);

    Ok(())
}

#[tokio::test]
async fn test_donate_to_campaign_with_target() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    let now = Utc::now().timestamp_millis();
    let start_ms = (now + 1000) as u64;
    let end_ms = Some(start_ms + 10_0000);

    // Create campaign with target
    println!(
        "check bob bal.... {:?}, >> {:?}",
        bob.view_account().await?.balance,
        now
    );
    let name = "Test Campaign".to_string();
    let description = Some("Test Description".to_string());
    let cover_image_url = Some("https://example.com/image.jpg".to_string());
    let recipient = bob.id().clone();
    let ft_id = None;
    let target_amount = U128::from(3000000000000000000000000);
    let min_amount = Some(U128::from(3000000000000000000000000));
    let max_amount = Some(U128::from(5000000000000000000000000));
    let referral_fee_basis_points = Some(100);
    let creator_fee_basis_points = Some(100);
    let allow_fee_avoidance = Some(true);

    let stateA = contract.view_account().await?;

    println!("state before first donation.... : {:?}", stateA);
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

    println!("get whole res {:?}", res);

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

    println!("Result 2.. {:?}", donate_result2);
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
        campaign2["min_amount"], campaign2["total_raised_amount"]
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
        .call("process_escrowed_donations_batch") // call with campaign id
        .args_json(json!({ "campaign_id": campaign_id }))
        .max_gas()
        .transact()
        .await?;

    println!(
        "Go berserk.... {:?}",
        process_escrowed_donations_batch_result
    );
    assert!(process_escrowed_donations_batch_result.is_success());

    let stateB = contract.view_account().await?;

    println!("state after all donation.... : {:?}", stateB);

    let storage_used = stateB.storage_usage - stateA.storage_usage;

    println!("total used all2geda.... : {}", storage_used);

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
async fn test_donate_to_campaign_with_target_without_min_amount() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    let now = Utc::now().timestamp_millis();
    let start_ms = (now + 1000) as u64;
    let end_ms = Some(start_ms + 10_000);

    // Create campaign with target
    println!(
        "check bob bal.... {:?}, >> {:?}",
        bob.view_account().await?.balance,
        now
    );
    let name = "Test Campaign".to_string();
    let description = Some("Test Description".to_string());
    let cover_image_url = Some("https://example.com/image.jpg".to_string());
    let recipient = bob.id().clone();
    let ft_id = None;
    let target_amount = U128::from(3000000000000000000000000);
    let max_amount = Some(U128::from(5000000000000000000000000));
    let referral_fee_basis_points = Some(100);
    let creator_fee_basis_points = Some(100);
    let allow_fee_avoidance = Some(true);

    let stateA = contract.view_account().await?;

    println!("state before first donation.... : {:?}", stateA);
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
        None,
        max_amount,
        referral_fee_basis_points,
        creator_fee_basis_points,
        allow_fee_avoidance,
    )
    .await?;

    println!("get whole res {:?}", res);

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
        campaign2["min_amount"], campaign2["total_raised_amount"]
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

    let stateB = contract.view_account().await?;

    println!("state after all donation.... : {:?}", stateB);

    let storage_used = stateB.storage_usage - stateA.storage_usage;

    println!("total used all2geda.... : {}", storage_used);

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
    let now = Utc::now().timestamp_millis();
    let start_ms = (now + 1000) as u64;
    let end_ms = Some(start_ms + 10_000);
    println!("SHOYUT PLSSS... {}", now);
    let campaign_duration = 10_000; // 10 seconds
    let campaign_id = create_test_campaign(
        &contract,
        &alice,
        start_ms,
        end_ms,
        U128::from(10_000_000_000_000_000_000_000_000),
    )
    .await?;

    // Make donations from Alice and Bob
    let alice_donation = NearToken::from_near(1);
    let bob_donation = NearToken::from_near(2);

    let alice_initial_balance = alice.view_account().await?.balance;
    let bob_initial_balance = bob.view_account().await?.balance;

    donate_to_campaign(&alice, &contract, campaign_id, alice_donation).await?;
    donate_to_campaign(&bob, &contract, campaign_id, bob_donation).await?;
    donate_to_campaign(&alice, &contract, campaign_id, alice_donation).await?;
    donate_to_campaign(&alice, &contract, campaign_id, alice_donation).await?;

    // Verify donations were made
    let campaign: serde_json::Value = get_campaign(&contract, campaign_id).await?;
    let total_donations = alice_donation
        .saturating_mul(3)
        .saturating_add(bob_donation);
    assert_eq!(
        campaign["total_raised_amount"],
        total_donations.as_yoctonear().to_string()
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
    println!(
        "unescroed baalnce campaign..... {:?}",
        campaign_after_refund
    );
    assert_eq!(campaign_after_refund["escrow_balance"], "0");

    // Check if donations are marked as refunded
    let campaign_donations: Vec<serde_json::Value> =
        get_campaign_donations(&contract, campaign_id).await?;
    // for donation in campaign_donations {
    //     assert!(donation["returned_at_ms"].is_string());
    // }

    Ok(())
}

#[tokio::test]
async fn test_referral_fee_distribution() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;

    // Create another account to be the referrer
    let referrer = worker.dev_create_account().await?;

    let now = Utc::now().timestamp_millis() as u64;

    // Record initial balances
    let referrer_initial_balance = referrer.view_account().await?.balance;

    // Create campaign with specific referral fee (500 basis points = 5%)
    let res = create_campaign(
        &contract,
        alice.clone(),
        "Test Referral Campaign".to_string(),
        Some("Testing referral fees".to_string()),
        None,
        bob.id().clone(), // bob is the recipient
        now + 1000,
        Some(now + 10000),
        None,                                           // No FT
        U128::from(10_000_000_000_000_000_000_000_000), // 10 NEAR target
        None,                                           // No min amount
        None,                                           // No max amount
        Some(500),                                      // 5% referral fee
        Some(100),                                      // 1% creator fee
        Some(false),                                    // Don't allow fee avoidance
    )
    .await?;

    assert!(res.is_success());
    let campaign_id = extract_campaign_id_from_logs(&res)?;

    // Make a donation with referrer
    let donation_amount = NearToken::from_near(10); // 10 NEAR
    let donate_result = alice
        .call(contract.id(), "donate")
        .args_json(json!({
            "campaign_id": campaign_id,
            "referrer_id": referrer.id(),
            "message": "Donation with referral"
        }))
        .deposit(donation_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(donate_result.is_success());

    // Get the donation details
    let donations: Vec<serde_json::Value> = contract
        .call("get_donations_for_campaign")
        .args_json(json!({
            "campaign_id": campaign_id,
            "from_index": 0,
            "limit": 10
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(donations.len(), 1);

    let donation = &donations[0];

    // Calculate expected referral fee (5% of donation amount)
    let expected_referral_fee = donation_amount.as_yoctonear() * 500 / 10000;

    // Verify referral fee in donation record
    let actual_referral_fee = donation["referrer_fee"]
        .as_str()
        .unwrap()
        .parse::<u128>()
        .unwrap();
    assert_eq!(actual_referral_fee, expected_referral_fee);

    // Verify referrer_id in donation record
    assert_eq!(
        donation["referrer_id"].as_str().unwrap(),
        referrer.id().to_string()
    );

    // Wait a bit and check referrer's balance
    worker.fast_forward(5).await?;
    let referrer_final_balance = referrer.view_account().await?.balance;

    // Verify referrer received the fee
    // Note: We check if the balance increased by at least 90% of expected fee
    // to account for gas fees and rounding
    let balance_increase = referrer_final_balance.saturating_sub(referrer_initial_balance);
    assert!(
        balance_increase.as_yoctonear() >= expected_referral_fee * 90 / 100,
        "Referrer didn't receive the expected fee. Expected around {} yoctoNEAR, got {} yoctoNEAR",
        expected_referral_fee,
        balance_increase.as_yoctonear()
    );

    // Also test a donation without referrer
    let donate_without_referral = alice
        .call(contract.id(), "donate")
        .args_json(json!({
            "campaign_id": campaign_id,
            "message": "Donation without referral"
        }))
        .deposit(donation_amount)
        .max_gas()
        .transact()
        .await?;

    assert!(donate_without_referral.is_success());

    // Verify the donation without referral
    let all_donations: Vec<serde_json::Value> = contract
        .call("get_donations_for_campaign")
        .args_json(json!({
            "campaign_id": campaign_id,
            "from_index": 0,
            "limit": 10
        }))
        .view()
        .await?
        .json()?;

    let donation_without_referral = &all_donations[1];
    assert!(donation_without_referral["referrer_id"].is_null());
    assert!(donation_without_referral["referrer_fee"].is_null());

    Ok(())
}

#[tokio::test]
async fn test_referral_fee_edge_cases() -> Result<()> {
    let worker = sandbox().await?;
    let (contract, alice, bob) = init(&worker).await?;
    let referrer = worker.dev_create_account().await?;

    let now = Utc::now().timestamp_millis() as u64;

    // Test with maximum allowed referral fee (10%)
    let res = create_campaign(
        &contract,
        alice.clone(),
        "Max Referral Fee Campaign".to_string(),
        None,
        None,
        bob.id().clone(),
        now + 1000,
        Some(now + 10000),
        None,
        U128::from(10_000_000_000_000_000_000_000_000),
        None,
        None,
        Some(1000), // 10% - maximum allowed
        None,
        None,
    )
    .await?;

    assert!(res.is_success());
    let campaign_id = extract_campaign_id_from_logs(&res)?;

    // Test with very small donation amount
    let small_donation = NearToken::from_near(1);
    let small_donate_result = alice
        .call(contract.id(), "donate")
        .args_json(json!({
            "campaign_id": campaign_id,
            "referrer_id": referrer.id(),
        }))
        .deposit(small_donation)
        .max_gas()
        .transact()
        .await?;

    assert!(small_donate_result.is_success());

    // Try to create campaign with above maximum referral fee (should fail)
    let invalid_res = create_campaign(
        &contract,
        alice.clone(),
        "Invalid Referral Fee Campaign".to_string(),
        None,
        None,
        bob.id().clone(),
        now + 1000,
        Some(now + 10000),
        None,
        U128::from(100),
        None,
        None,
        Some(1100), // 11% - above maximum
        None,
        None,
    )
    .await;

    assert!(invalid_res.is_err() || !invalid_res.unwrap().is_success());

    Ok(())
}

async fn create_test_campaign(
    contract: &Contract,
    creator: &Account,
    start_ms: u64,
    end: Option<u64>,
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
        end,
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

async fn get_campaign_donations(
    contract: &Contract,
    campaign_id: u64,
) -> Result<Vec<serde_json::Value>> {
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
