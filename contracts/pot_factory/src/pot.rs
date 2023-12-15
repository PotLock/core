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
    owner: Option<AccountId>,
    admins: Option<Vec<AccountId>>,
    chef: Option<AccountId>,
    pot_name: String,
    pot_description: String,
    max_projects: u32,
    application_start_ms: TimestampMs,
    application_end_ms: TimestampMs,
    public_round_start_ms: TimestampMs,
    public_round_end_ms: TimestampMs,
    registry_provider: Option<ProviderId>,
    sybil_wrapper_provider: Option<ProviderId>,
    custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    custom_min_threshold_score: Option<u32>,
    patron_referral_fee_basis_points: u32,
    public_round_referral_fee_basis_points: u32,
    chef_fee_basis_points: u32,
    protocol_config_provider: Option<ProviderId>,
    source_metadata: ContractSourceMetadata,
}

#[near_bindgen]
impl Contract {
    /// Deploy a new Pot. A `None` response indicates an unsuccessful deployment.
    #[payable]
    pub fn deploy_pot(&mut self, mut pot_args: PotArgs) -> Promise {
        self.assert_admin_or_whitelisted_deployer();
        let pot_account_id_str = format!(
            "{}.{}",
            slugify(&pot_args.pot_name),
            env::current_account_id()
        );
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

        // add protocol config provider to pot args
        pot_args.protocol_config_provider = Some(ProviderId::new(
            env::current_account_id().to_string(),
            "get_protocol_config".to_string(),
        ));

        let min_deployment_deposit = self.get_min_deployment_deposit(&pot_args);

        // insert dummy record in advance to check required deposit
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

        // delete temporarily created pot
        self.pots_by_id.remove(&pot_account_id);

        // deploy pot
        Promise::new(pot_account_id.clone())
            .create_account()
            .transfer(min_deployment_deposit)
            .deploy_contract(POT_WASM_CODE.to_vec())
            .add_full_access_key(env::signer_account_pk()) // TODO: REMOVE THIS AFTER TESTING
            .function_call(
                "new".to_string(),
                serde_json::to_vec(&pot_args).unwrap(),
                0,
                XCC_GAS,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .deploy_pot_callback(pot_account_id.clone(), min_deployment_deposit, deposit),
            )
    }

    #[private] // Public fn, but only callable by env::current_account_id()
    pub fn deploy_pot_callback(
        &mut self,
        pot_id: AccountId,
        min_deployment_deposit: Balance,
        deposit: Balance,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Option<PotExternal> {
        if call_result.is_err() {
            let error_message = format!(
                "There was an error deploying the Pot contract. Returning deposit to signer."
            );
            env::log_str(&error_message);
            // return deposit to signer
            Promise::new(env::signer_account_id()).transfer(deposit);
            // don't panic or refund transfer won't occur! instead, return `None`
            return None;
        }

        let pot = Pot {
            deployed_by: env::signer_account_id(),
            deployed_at_ms: env::block_timestamp_ms(),
        };

        let initial_storage_usage = env::storage_usage();

        self.pots_by_id
            .insert(&pot_id, &VersionedPot::Current(pot.clone()));

        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        // env::storage_byte_cost() * Balance::from(storage_used);
        let total_cost = required_deposit + min_deployment_deposit;
        if deposit > total_cost {
            Promise::new(env::signer_account_id()).transfer(deposit - total_cost);
        }

        Some(PotExternal {
            id: pot_id,
            deployed_by: pot.deployed_by,
            deployed_at_ms: pot.deployed_at_ms,
        })
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

    pub fn get_min_deployment_deposit(&self, args: &PotArgs) -> u128 {
        ((POT_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as Balance
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }
}
