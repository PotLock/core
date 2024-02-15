use crate::*;

pub const POT_WASM_CODE: &[u8] = include_bytes!("../../pot/out/main.wasm");

/// The address of a deployed Pot contract
pub type PotId = AccountId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Pot {
    pub deployed_by: AccountId,
    pub deployed_at_ms: TimestampMs,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedPot {
    Current(Pot),
}

impl From<VersionedPot> for Pot {
    fn from(pot: VersionedPot) -> Self {
        match pot {
            VersionedPot::Current(current) => current,
        }
    }
}

/// Ephemeral-only (used for views; not stored in contract)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PotExternal {
    id: PotId,
    deployed_by: AccountId,
    deployed_at_ms: TimestampMs,
}

/// Arguments that must be provided to deploy a new Pot; these must be kept up-to-date with the Pot contract
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotArgs {
    pub owner: Option<AccountId>,
    pub admins: Option<Vec<AccountId>>,
    pub chef: Option<AccountId>,
    pub pot_name: String,
    pub pot_description: String,
    pub max_projects: u32,
    pub application_start_ms: TimestampMs,
    pub application_end_ms: TimestampMs,
    pub public_round_start_ms: TimestampMs,
    pub public_round_end_ms: TimestampMs,
    pub min_matching_pool_donation_amount: Option<U128>,
    pub cooldown_period_ms: Option<u64>,
    pub registry_provider: Option<ProviderId>,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    pub custom_min_threshold_score: Option<u32>,
    pub referral_fee_matching_pool_basis_points: u32,
    pub referral_fee_public_round_basis_points: u32,
    pub chef_fee_basis_points: u32,
    pub protocol_config_provider: Option<ProviderId>,
    pub source_metadata: ContractSourceMetadata,
}

#[near_bindgen]
impl Contract {
    /// Deploy a new Pot. A `None` response indicates an unsuccessful deployment.
    #[payable]
    pub fn deploy_pot(&mut self, mut pot_args: PotArgs, pot_handle: Option<String>) -> Promise {
        // TODO: add protocol_config_provider to pot_args
        if self.require_whitelist {
            self.assert_admin_or_whitelisted_deployer();
        }
        let handle = pot_handle.unwrap_or_else(|| slugify(&pot_args.pot_name));
        let pot_account_id_str = format!("{}.{}", handle, env::current_account_id());
        assert!(
            env::is_valid_account_id(pot_account_id_str.as_bytes()),
            "Pot Account ID {} is invalid",
            pot_account_id_str
        );
        let pot_account_id = AccountId::new_unchecked(pot_account_id_str);

        // check no pot exists with this id
        assert!(
            self.pots_by_id.get(&pot_account_id).is_none(),
            "Pot with id {} already exists",
            pot_account_id
        );

        // validate pot args
        assert_valid_pot_args(&pot_args);

        // TODO: validate registry & sybil wrapper providers (if present) by calling them

        // add protocol config provider to pot args
        pot_args.protocol_config_provider = Some(ProviderId::new(
            env::current_account_id().to_string(),
            "get_protocol_config".to_string(),
        ));

        let min_deployment_deposit = self.calculate_min_deployment_deposit(&pot_args);

        // insert record in advance to validate required deposit & avoid race conditions
        let pot = Pot {
            deployed_by: env::signer_account_id(),
            deployed_at_ms: env::block_timestamp_ms(),
        };

        let initial_storage_usage = env::storage_usage();

        self.pots_by_id
            .insert(&pot_account_id, &VersionedPot::Current(pot.clone())); // TODO: review this for race conditions

        let deposit = env::attached_deposit();
        let required_storage_deposit = calculate_required_storage_deposit(initial_storage_usage);

        // total required deposit
        let total_required_deposit = required_storage_deposit + min_deployment_deposit;

        // assert total_required_deposit
        assert!(
            deposit >= total_required_deposit,
            "Attached deposit of {} is less than required deposit of {}",
            deposit,
            total_required_deposit
        );

        // if more is attached than needed, return the difference
        if deposit > total_required_deposit {
            Promise::new(env::signer_account_id()).transfer(deposit - total_required_deposit);
        }

        // deploy pot
        Promise::new(pot_account_id.clone())
            .create_account()
            .transfer(min_deployment_deposit)
            .deploy_contract(POT_WASM_CODE.to_vec())
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&pot_args).unwrap(),
                0,
                XCC_GAS,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .deploy_pot_callback(
                        pot_account_id.clone(),
                        pot.clone(),
                        total_required_deposit,
                    ),
            )
    }

    #[private] // Public fn, but only callable by env::current_account_id()
    pub fn deploy_pot_callback(
        &mut self,
        pot_id: AccountId,
        pot: Pot,
        total_required_deposit: Balance,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Option<PotExternal> {
        if call_result.is_err() {
            let error_message = format!(
                "There was an error deploying the Pot contract. Returning deposit to signer."
            );
            env::log_str(&error_message);
            // delete pot that was created in initial call
            self.pots_by_id.remove(&pot_id);
            // return total_required_deposit to signer (difference between attached deposit and required deposit was already refunded in initial call)
            Promise::new(env::signer_account_id()).transfer(total_required_deposit);
            // don't panic or refund transfer won't occur! instead, return `None`
            None
        } else {
            let pot_external = PotExternal {
                id: pot_id,
                deployed_by: pot.deployed_by,
                deployed_at_ms: pot.deployed_at_ms,
            };

            log_deploy_pot_event(&pot_external);

            Some(pot_external)
        }
    }

    pub fn get_pots(&self) -> Vec<PotExternal> {
        self.pots_by_id
            .iter()
            .map(|(id, v)| {
                let pot = Pot::from(v);
                PotExternal {
                    id,
                    deployed_by: pot.deployed_by.clone(),
                    deployed_at_ms: pot.deployed_at_ms,
                }
            })
            .collect()
    }

    pub fn calculate_min_deployment_deposit(&self, args: &PotArgs) -> u128 {
        ((POT_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as Balance
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }
}
