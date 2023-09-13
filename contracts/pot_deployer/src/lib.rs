use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::env::STORAGE_PRICE_PER_BYTE;
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, require, serde_json, AccountId, Balance, BorshStorageKey,
    Gas, Promise, PromiseError,
};

/// The address of a deployed Pot contract
pub type PotId = AccountId;
type TimestampMs = u64;
const EXTRA_BYTES: usize = 10000;
const GAS: Gas = Gas(50_000_000_000_000);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SBTRequirement {
    // consider importing/sharing this from Pot contract to avoid redefining
    pub registry_id: AccountId,
    pub issuer_id: AccountId,
    pub class_id: u64,
}

const POT_WASM_CODE: &[u8] = include_bytes!("../../pot/out/main.wasm");

/// Pot Deployer Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pots: UnorderedSet<PotId>,
    protocol_fee: u128,
    chef_fee: u128,
    max_round_time: u128,
    max_application_time: u128,
    max_milestones: u32,
    max_protocol_fee_basis_points: u32,
    max_chef_fee_basis_points: u32,
    initial_chef_fee_basis_points: u32,
    initial_protocol_fee_basis_points: u32,
    admin: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Pots,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotArgs {
    chef_id: AccountId,
    round_name: String,
    round_description: String,
    round_start_time: TimestampMs,
    round_end_time: TimestampMs,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    max_projects: u32,
    base_currency: AccountId,
    created_by: AccountId,
    milestone_threshold: U64,
    basis_points_paid_upfront: u32,
    application_requirement: Option<SBTRequirement>,
    donation_requirement: Option<SBTRequirement>,
    patron_referral_fee_basis_points: u32,
    max_patron_referral_fee: U128,
    round_manager_fee_basis_points: u32,
    protocol_fee_basis_points: u32,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        protocol_fee: u128,
        chef_fee: u128,
        max_round_time: u128,
        max_application_time: u128,
        max_milestones: u32,
        max_protocol_fee_basis_points: u32,
        max_chef_fee_basis_points: u32,
        initial_chef_fee_basis_points: u32,
        initial_protocol_fee_basis_points: u32,
        admin: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            pots: UnorderedSet::new(StorageKey::Pots),
            protocol_fee,
            chef_fee,
            max_round_time,
            max_application_time,
            max_milestones,
            max_protocol_fee_basis_points,
            max_chef_fee_basis_points,
            initial_chef_fee_basis_points,
            initial_protocol_fee_basis_points,
            admin,
        }
    }

    fn get_min_attached_deposit(&self, args: &PotArgs) -> u128 {
        ((POT_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as Balance
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }

    #[payable]
    pub fn deploy_pot(&mut self, pot_on_chain_name: String, pot_args: PotArgs) -> Promise {
        // TODO: ensure caller has appropriate permissions!
        let pot_account_id_str = format!("{}.{}", pot_on_chain_name, env::current_account_id());
        assert!(
            env::is_valid_account_id(pot_account_id_str.as_bytes()),
            "Pot Account ID is invalid"
        );
        let pot_account_id = AccountId::new_unchecked(pot_account_id_str);
        let required_deposit = self.get_min_attached_deposit(&pot_args);

        let initial_storage_usage = env::storage_usage();

        let storage_balance_used =
            Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;

        Promise::new(pot_account_id)
            .create_account()
            .transfer(required_deposit - storage_balance_used)
            .deploy_contract(POT_WASM_CODE.to_vec())
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&pot_args).unwrap(),
                0,
                GAS,
            )
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            pots: UnorderedSet::new(StorageKey::Pots),
            protocol_fee: 0,
            chef_fee: 0,
            max_round_time: 0,
            max_application_time: 0,
            max_milestones: 0,
            max_protocol_fee_basis_points: 0,
            max_chef_fee_basis_points: 0,
            initial_chef_fee_basis_points: 0,
            initial_protocol_fee_basis_points: 0,
            admin: AccountId::new_unchecked("".to_string()),
        }
    }
}
