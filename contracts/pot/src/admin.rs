use crate::*;

#[near_bindgen]
impl Contract {
    // CHANGE OWNER
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        self.owner = new_owner;
    }

    // ADD ADMINS
    pub fn owner_add_admins(&mut self, new_admins: Vec<AccountId>) {
        self.assert_owner();
        for new_admin in new_admins.iter() {
            self.admins.insert(new_admin);
        }
    }

    // REMOVE ADMINS
    pub fn owner_remove_admins(&mut self, admins_to_remove: Vec<AccountId>) {
        self.assert_owner();
        for admin_to_remove in admins_to_remove.iter() {
            self.admins.remove(admin_to_remove);
        }
    }

    // CHEF
    pub fn admin_set_chef(&mut self, chef: AccountId) {
        self.assert_owner_or_admin();
        self.chef.set(&chef);
    }

    pub fn admin_remove_chef(&mut self) {
        self.assert_owner_or_admin();
        self.chef.remove();
    }

    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) {
        self.assert_owner_or_admin();
        self.chef_fee_basis_points = chef_fee_basis_points;
    }

    // POT CONFIG
    pub fn admin_set_pot_name(&mut self, pot_name: String) {
        self.assert_owner_or_admin();
        self.pot_name = pot_name;
    }

    pub fn admin_set_pot_description(&mut self, pot_description: String) {
        self.assert_owner_or_admin();
        self.pot_description = pot_description;
    }

    pub fn admin_set_max_projects(&mut self, max_projects: u32) {
        self.assert_owner_or_admin();
        self.max_projects = max_projects;
    }

    pub fn admin_set_base_currency(&mut self, base_currency: AccountId) {
        self.assert_owner_or_admin();
        // only "near" allowed for now
        assert_eq!(
            base_currency,
            AccountId::new_unchecked("near".to_string()),
            "Only NEAR is supported"
        );
        self.base_currency = base_currency;
    }

    pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) {
        self.assert_owner_or_admin();
        self.application_start_ms = application_start_ms;
    }

    pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) {
        self.assert_owner_or_admin();
        self.application_end_ms = application_end_ms;
    }

    pub fn admin_set_public_round_start_ms(&mut self, public_round_start_ms: u64) {
        self.assert_owner_or_admin();
        self.public_round_start_ms = public_round_start_ms;
    }

    pub fn admin_set_public_round_end_ms(&mut self, public_round_end_ms: u64) {
        self.assert_owner_or_admin();
        self.public_round_end_ms = public_round_end_ms;
    }

    pub fn admin_set_round_open(&mut self, public_round_end_ms: TimestampMs) {
        self.assert_owner_or_admin();
        self.public_round_start_ms = env::block_timestamp_ms();
        self.public_round_end_ms = public_round_end_ms;
    }

    pub fn admin_set_round_closed(&mut self) {
        self.assert_owner_or_admin();
        self.public_round_end_ms = env::block_timestamp_ms();
    }

    pub fn admin_set_registry_provider(&mut self, contract_id: AccountId, method_name: String) {
        self.assert_owner_or_admin();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.registry_provider.set(&provider_id);
    }

    pub fn admin_remove_registry_provider(&mut self) {
        self.assert_owner_or_admin();
        self.registry_provider.remove();
    }

    pub fn admin_set_sybil_wrapper_provider(
        &mut self,
        contract_id: AccountId,
        method_name: String,
    ) {
        self.assert_owner_or_admin();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.sybil_wrapper_provider.set(&provider_id);
    }

    pub fn admin_remove_sybil_wrapper_provider(&mut self) {
        self.assert_owner_or_admin();
        self.sybil_wrapper_provider.remove();
    }

    pub fn admin_set_custom_sybil_checks(&mut self, custom_sybil_checks: Vec<CustomSybilCheck>) {
        self.assert_owner_or_admin();
        let formatted_custom_sybil_checks: HashMap<ProviderId, SybilProviderWeight> =
            custom_sybil_checks
                .into_iter()
                .map(|custom_sybil_check| {
                    let provider_id = ProviderId::new(
                        custom_sybil_check.contract_id.to_string(),
                        custom_sybil_check.method_name,
                    );
                    (provider_id, custom_sybil_check.weight)
                })
                .collect();
        self.custom_sybil_checks.set(&formatted_custom_sybil_checks);
    }

    pub fn admin_remove_custom_sybil_checks(&mut self) {
        self.assert_owner_or_admin();
        self.custom_sybil_checks.remove();
    }

    pub fn admin_set_custom_min_threshold_score(&mut self, custom_min_threshold_score: u32) {
        self.assert_owner_or_admin();
        self.custom_min_threshold_score
            .set(&custom_min_threshold_score);
    }

    pub fn admin_remove_custom_min_threshold_score(&mut self) {
        self.assert_owner_or_admin();
        self.custom_min_threshold_score.remove();
    }

    pub fn admin_set_patron_referral_fee_basis_points(
        &mut self,
        patron_referral_fee_basis_points: u32,
    ) {
        self.assert_owner_or_admin();
        self.patron_referral_fee_basis_points = patron_referral_fee_basis_points;
    }

    pub fn admin_set_public_round_referral_fee_basis_points(
        &mut self,
        public_round_referral_fee_basis_points: u32,
    ) {
        self.assert_owner_or_admin();
        self.public_round_referral_fee_basis_points = public_round_referral_fee_basis_points;
    }

    pub fn admin_set_cooldown_period_complete(&mut self) {
        self.assert_owner_or_admin();
        self.cooldown_end_ms.set(&env::block_timestamp_ms());
    }

    // PAYOUTS
    pub fn admin_process_payouts(&mut self) {
        self.assert_owner_or_admin();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.all_paid_out == false,
            "Payouts have already been processed"
        );
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // pay out each project
        // for each approved project...
        for (project_id, v_app) in self.applications_by_id.iter() {
            // TODO: update this to only go through approved applications mapping
            self.assert_approved_application(&project_id);
            let application = Application::from(v_app);
            // ...if there are payouts for the project...
            if let Some(payout_ids_for_project) = self.payout_ids_by_project_id.get(&project_id) {
                // TODO: handle milestones (for now just paying out all payouts)
                for payout_id in payout_ids_for_project.iter() {
                    let payout =
                        Payout::from(self.payouts_by_id.get(&payout_id).expect("no payout"));
                    if payout.paid_at.is_none() {
                        // ...transfer funds...
                        Promise::new(application.project_id.clone())
                            .transfer(payout.amount.0)
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(XCC_GAS)
                                    .transfer_payout_callback(payout),
                            );
                    }
                }
            }
        }
        self.all_paid_out = true;
    }

    /// Verifies whether payout transfer completed successfully & updates payout record accordingly
    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_payout_callback(
        &mut self,
        mut payout: Payout,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            log!(format!(
                "Error paying out amount {:#?} to project {}",
                payout.amount, payout.project_id
            ));
        } else {
            log!(format!(
                "Successfully paid out amount {:#?} to project {}",
                payout.amount, payout.project_id
            ));
            // update payout to indicate that funds have been transferred
            payout.paid_at = Some(env::block_timestamp_ms());
            let payout_id = payout.id.clone();
            self.payouts_by_id
                .insert(&payout_id, &VersionedPayout::Current(payout));
        }
    }
}
