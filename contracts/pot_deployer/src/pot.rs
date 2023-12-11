use crate::*;

pub const POT_WASM_CODE: &[u8] = include_bytes!("../../pot/out/main.wasm");

/// The address of a deployed Pot contract
pub type PotId = AccountId;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Pot {
    pub pot_id: AccountId,
    // pub on_chain_name: String,
    pub deployed_by: AccountId,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn deploy_pot(&mut self, pot_args: PotArgs) -> Promise {
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
        let required_deposit = self.get_min_deployment_deposit(&pot_args);

        let initial_storage_usage = env::storage_usage();

        let storage_balance_used =
            Balance::from(env::storage_usage() - initial_storage_usage) * STORAGE_PRICE_PER_BYTE;

        Promise::new(pot_account_id.clone())
            .create_account()
            .transfer(required_deposit - storage_balance_used)
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
                    .deploy_pot_callback(pot_account_id.clone()),
            )
    }

    #[private] // Public fn, but only callable by env::current_account_id()
    pub fn deploy_pot_callback(
        &mut self,
        pot_id: AccountId,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Pot {
        if call_result.is_err() {
            env::panic_str("There was an error deploying the Pot contract.");
        }
        let pot = Pot {
            pot_id: pot_id.clone(),
            deployed_by: env::signer_account_id(),
        };
        self.pots_by_id.insert(&pot_id, &pot);
        pot
    }

    pub fn get_pots(&self) -> Vec<Pot> {
        self.pots_by_id
            .iter()
            .map(|(_, pot)| pot)
            .collect::<Vec<Pot>>()
    }

    pub fn get_min_deployment_deposit(&self, args: &PotArgs) -> u128 {
        ((POT_WASM_CODE.len() + EXTRA_BYTES + args.try_to_vec().unwrap().len() * 2) as Balance
            * STORAGE_PRICE_PER_BYTE)
            .into()
    }
}
