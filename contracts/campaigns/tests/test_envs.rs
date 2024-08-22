use anyhow::Result;
use log::info;
use near_workspaces::{types::NearToken, Account, Contract, DevNetwork, Worker};

use std::sync::Once;

// Initialize logger only once for all tests
static INIT: Once = Once::new();

pub fn init_logger() {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
        println!("Logger initialized");
        info!("Logger initialized"); // Log to confirm initialization
    });
}

pub async fn init(worker: &Worker<impl DevNetwork>) -> Result<(Contract, Account, Account)> {
    let campaigns_contract = worker
        .dev_deploy(include_bytes!("../out/main.wasm"))
        .await?;

    let res = campaigns_contract
        .call("new_default_meta")
        .args_json((campaigns_contract.id(),))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let alice = worker.dev_create_account().await?;
    let bob = worker.dev_create_account().await?;

    return Ok((campaigns_contract, alice, bob));
}
