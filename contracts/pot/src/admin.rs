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

    // ROUND
    pub fn admin_set_round_open(&mut self, round_end_ms: TimestampMs) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_round_open_callback(
                        env::predecessor_account_id().clone(),
                        round_end_ms,
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_round_open_callback(
        &mut self,
        caller_id: AccountId,
        round_end_ms: TimestampMs,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.round_start_ms = env::block_timestamp_ms();
        self.round_end_ms = round_end_ms;
    }

    pub fn admin_close_round(&mut self) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_close_round_callback(env::predecessor_account_id().clone()),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_close_round_callback(
        &mut self,
        caller_id: AccountId,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.round_end_ms = env::block_timestamp_ms();
    }

    // PAYOUTS
    pub fn admin_process_payouts(&mut self) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_process_payouts_callback(env::predecessor_account_id().clone()),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_process_payouts_callback(
        &mut self,
        caller_id: AccountId,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.paid_out == false,
            "Payouts have already been processed"
        );
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // pay out each project
        // for each approved project...
        for (project_id, application) in self.applications_by_project_id.iter() {
            self.assert_approved_application(&project_id);
            // ...if there are payouts for the project...
            if let Some(payout_ids_for_project) = self.payout_ids_by_project_id.get(&project_id) {
                // TODO: handle milestones (for now just paying out all payouts)
                for payout_id in payout_ids_for_project.iter() {
                    let mut payout = self.payouts_by_id.get(&payout_id).expect("no payout");
                    if payout.paid_at.is_none() {
                        // ...transfer funds...
                        Promise::new(application.project_id.clone())
                            .transfer(payout.amount_total.0); // TODO: what happens if this fails?
                                                              // ...and update payout to indicate that funds have been transferred
                                                              // TODO: handle via Promise callback?
                        payout.paid_at = Some(env::block_timestamp_ms());
                        self.payouts_by_id.insert(&payout_id, &payout);
                    }
                }
            }
        }
        // set paid_out to true
        self.paid_out = true;
    }

    pub fn admin_set_cooldown_period_complete(&mut self) -> Promise {
        pot_deployer::ext(self.pot_deployer_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_admin()
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .admin_set_cooldown_period_complete_callback(
                        env::predecessor_account_id().clone(),
                    ),
            )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn admin_set_cooldown_period_complete_callback(
        &mut self,
        caller_id: AccountId,
        #[callback_result] call_result: Result<AccountId, PromiseError>,
    ) {
        assert_eq!(
            caller_id,
            call_result.expect("Failed to get admin on Pot Deployer contract."),
            "Caller is not admin on Pot Deployer contract."
        );
        self.cooldown_end_ms = Some(env::block_timestamp_ms());
    }
}
