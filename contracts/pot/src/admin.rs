use crate::*;

#[near_bindgen]
impl Contract {
    // APPLICATION START
    pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_application_start_ms_callback(
                        env::predecessor_account_id().clone(),
                        application_start_ms.clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_application_start_ms_callback(
        &mut self,
        caller_id: AccountId,
        application_start_ms: u64,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.application_start_ms = application_start_ms;
    }

    // APPLICATION END

    pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_application_end_ms_callback(
                        env::predecessor_account_id().clone(),
                        application_end_ms.clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_application_end_ms_callback(
        &mut self,
        caller_id: AccountId,
        application_end_ms: u64,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.application_end_ms = application_end_ms;
    }

    // CHEF

    pub fn admin_set_chef(&mut self, chef_id: AccountId) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_chef_callback(
                        env::predecessor_account_id().clone(),
                        chef_id.clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_chef_callback(
        &mut self,
        caller_id: AccountId,
        chef_id: AccountId,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.chef_id = chef_id;
    }

    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_chef_fee_basis_points_callback(
                        env::predecessor_account_id().clone(),
                        chef_fee_basis_points.clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_chef_fee_basis_points_callback(
        &mut self,
        caller_id: AccountId,
        chef_fee_basis_points: u32,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.chef_fee_basis_points = chef_fee_basis_points;
    }
}
