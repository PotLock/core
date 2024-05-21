#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use anyhow::Result;
    use env_logger;
    use log::info;
    use near_sdk::json_types::U128;
    use near_sdk::serde_json;
    use near_workspaces::operations::Function;
    use near_workspaces::result::{ExecutionFinalResult, ValueOrReceiptId};
    use near_workspaces::{
        sandbox, types::NearToken, Account, AccountId, Contract, DevNetwork, Worker,
    };
    use std::sync::Once;

    const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

    // Initialize logger only once for all tests
    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            println!("Logger initialized");
            info!("Logger initialized"); // Log to confirm initialization
        });
    }

    async fn create_campaign(
        contract: &Contract,
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
        let res = contract
            .call("create_campaign")
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
            .deposit(NearToken::from_near(1))
            .transact()
            .await;
        return res;
    }

    async fn init(worker: &Worker<impl DevNetwork>) -> Result<(Contract, Account, Account)> {
        let campaigns_contract = worker
            .dev_deploy(include_bytes!("../out/main.wasm"))
            .await?;

        let res = campaigns_contract
            .call("new_default_meta")
            .args_json((campaigns_contract.id(),))
            .max_gas()
            .transact()
            .await?;
        info!("res: {:?}", res);
        assert!(res.is_success());

        let alice = campaigns_contract
            .as_account()
            .create_subaccount("alice")
            .initial_balance(NearToken::from_near(10))
            .transact()
            .await?
            .into_result()?;

        let bob = campaigns_contract
            .as_account()
            .create_subaccount("bob")
            .initial_balance(NearToken::from_near(10))
            .transact()
            .await?
            .into_result()?;

        return Ok((campaigns_contract, alice, bob));
    }

    #[tokio::test]
    async fn test_create_campaign() -> Result<()> {
        init_logger();
        // let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
        let worker = sandbox().await?;
        let (contract, _alice, bob) = init(&worker).await?;

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
        let event_json_start =
            campaign_create_log.find("EVENT_JSON:").unwrap() + "EVENT_JSON:".len();
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

        Ok(())
    }
}
