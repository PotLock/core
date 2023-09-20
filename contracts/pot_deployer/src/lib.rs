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

pub mod utils;
pub use crate::utils::*;

pub const TGAS: u64 = 1_000_000_000_000;
pub const XXC_GAS: u64 = TGAS * 5;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SBTRequirement {
    // consider importing/sharing this from Pot contract to avoid redefining
    pub registry_id: AccountId,
    pub issuer_id: AccountId,
    pub class_id: u64,
}

const POT_WASM_CODE: &[u8] = include_bytes!("../../pot/out/main.wasm");

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Pot {
    pub pot_id: AccountId,
    pub on_chain_name: String,
    pub deployed_by: AccountId,
}

/// Pot Deployer Contract
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pot_ids: UnorderedSet<PotId>,
    pots_by_id: UnorderedMap<PotId, Pot>,
    protocol_fee_basis_points: u32,
    max_protocol_fee_basis_points: u32,
    default_chef_fee_basis_points: u32,
    max_chef_fee_basis_points: u32,
    max_round_time: u128,
    max_application_time: u128,
    // max_milestones: u32,
    admin: AccountId,
    whitelisted_deployers: UnorderedSet<AccountId>,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    PotsById,
    PotIds,
    WhitelistedDeployers,
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotArgs {
    chef_id: AccountId,
    round_name: String,
    round_description: String,
    round_start_ms: TimestampMs,
    round_end_ms: TimestampMs,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    max_projects: u32,
    base_currency: AccountId,
    // milestone_threshold: U64,
    // basis_points_paid_upfront: u32,
    donation_requirement: Option<SBTRequirement>,
    patron_referral_fee_basis_points: u32,
    max_patron_referral_fee: U128,
    round_manager_fee_basis_points: u32,
    protocol_fee_basis_points: u32,
}

/// `PotArgs` + `created_by`
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotArgsInternal {
    chef_id: AccountId,
    round_name: String,
    round_description: String,
    round_start_ms: TimestampMs,
    round_end_ms: TimestampMs,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    max_projects: u32,
    base_currency: AccountId,
    created_by: AccountId,
    // milestone_threshold: U64,
    // basis_points_paid_upfront: u32,
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
        max_round_time: u128,
        max_application_time: u128,
        // max_milestones: u32,
        protocol_fee_basis_points: u32,
        max_protocol_fee_basis_points: u32,
        default_chef_fee_basis_points: u32,
        max_chef_fee_basis_points: u32,
        admin: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            pot_ids: UnorderedSet::new(StorageKey::PotIds),
            pots_by_id: UnorderedMap::new(StorageKey::PotsById),
            protocol_fee_basis_points,
            default_chef_fee_basis_points,
            max_protocol_fee_basis_points,
            max_chef_fee_basis_points,
            max_round_time,
            max_application_time,
            // max_milestones,
            admin,
            whitelisted_deployers: UnorderedSet::new(StorageKey::WhitelistedDeployers),
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
        let pot_account_id_str = format!(
            "{}.{}",
            slugify(&pot_on_chain_name),
            env::current_account_id()
        );
        assert!(
            env::is_valid_account_id(pot_account_id_str.as_bytes()),
            "Pot Account ID {} is invalid",
            pot_account_id_str
        );
        let pot_account_id = AccountId::new_unchecked(pot_account_id_str);
        let required_deposit = self.get_min_attached_deposit(&pot_args);

        let initial_storage_usage = env::storage_usage();

        let storage_balance_used =
            Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;

        let pot_args_internal = PotArgsInternal {
            chef_id: pot_args.chef_id,
            round_name: pot_args.round_name,
            round_description: pot_args.round_description,
            round_start_ms: pot_args.round_start_ms,
            round_end_ms: pot_args.round_end_ms,
            application_start_ms: pot_args.application_start_ms,
            application_end_ms: pot_args.application_end_ms,
            max_projects: pot_args.max_projects,
            base_currency: pot_args.base_currency,
            created_by: env::predecessor_account_id(),
            // milestone_threshold: pot_args.milestone_threshold,
            // basis_points_paid_upfront: pot_args.basis_points_paid_upfront,
            donation_requirement: pot_args.donation_requirement,
            patron_referral_fee_basis_points: pot_args.patron_referral_fee_basis_points,
            max_patron_referral_fee: pot_args.max_patron_referral_fee,
            round_manager_fee_basis_points: pot_args.round_manager_fee_basis_points,
            protocol_fee_basis_points: pot_args.protocol_fee_basis_points,
        };

        Promise::new(pot_account_id.clone())
            .create_account()
            .transfer(required_deposit - storage_balance_used)
            .deploy_contract(POT_WASM_CODE.to_vec())
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&pot_args_internal).unwrap(),
                0,
                GAS,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .deploy_pot_callback(
                        pot_on_chain_name,
                        env::predecessor_account_id(),
                        pot_account_id.clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn deploy_pot_callback(
        &mut self,
        on_chain_name: String,
        deployed_by: AccountId,
        pot_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Pot {
        if call_result.is_err() {
            env::panic_str("There was an error deploying the Pot contract.");
        }
        let pot = Pot {
            pot_id: pot_id.clone(),
            on_chain_name,
            deployed_by,
        };
        self.pot_ids.insert(&pot_id);
        self.pots_by_id.insert(&pot_id, &pot);
        pot
    }

    pub fn get_pots(&self) -> Vec<Pot> {
        self.pot_ids
            .iter()
            .map(|pot_id| self.pots_by_id.get(&pot_id).unwrap())
            .collect()
    }
}

// TODO: not sure why this is necessary
impl Default for Contract {
    fn default() -> Self {
        Self {
            pot_ids: UnorderedSet::new(StorageKey::PotIds),
            pots_by_id: UnorderedMap::new(StorageKey::PotsById),
            protocol_fee_basis_points: 0,
            default_chef_fee_basis_points: 0,
            max_round_time: 0,
            max_application_time: 0,
            // max_milestones: 0,
            max_protocol_fee_basis_points: 0,
            max_chef_fee_basis_points: 0,
            admin: AccountId::new_unchecked("".to_string()),
            whitelisted_deployers: UnorderedSet::new(StorageKey::WhitelistedDeployers),
        }
    }
}
