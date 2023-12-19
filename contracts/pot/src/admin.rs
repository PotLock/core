use crate::*;

#[near_bindgen]
impl Contract {
    // CHANGE OWNER
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        self.owner = new_owner;
    }

    // ADD/REMOVE ADMINS
    pub fn owner_add_admins(&mut self, new_admins: Vec<AccountId>) {
        self.assert_owner();
        for new_admin in new_admins.iter() {
            self.admins.insert(new_admin);
        }
    }

    pub fn owner_remove_admins(&mut self, admins_to_remove: Vec<AccountId>) {
        self.assert_owner();
        for admin_to_remove in admins_to_remove.iter() {
            self.admins.remove(admin_to_remove);
        }
    }

    pub fn owner_set_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        for account_id in account_ids {
            self.admins.remove(&account_id);
        }
    }

    pub fn owner_clear_admins(&mut self) {
        self.assert_owner();
        self.admins.clear();
    }

    // CHEF
    pub fn admin_set_chef(&mut self, chef: AccountId) {
        self.assert_admin_or_greater();
        self.chef.set(&chef);
    }

    pub fn admin_remove_chef(&mut self) {
        self.assert_admin_or_greater();
        self.chef.remove();
    }

    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) {
        self.assert_admin_or_greater();
        self.chef_fee_basis_points = chef_fee_basis_points;
    }

    // POT CONFIG
    pub fn admin_set_pot_name(&mut self, pot_name: String) {
        self.assert_admin_or_greater();
        self.pot_name = pot_name;
    }

    pub fn admin_set_pot_description(&mut self, pot_description: String) {
        self.assert_admin_or_greater();
        self.pot_description = pot_description;
    }

    pub fn admin_set_max_projects(&mut self, max_projects: u32) {
        self.assert_admin_or_greater();
        self.max_projects = max_projects;
    }

    pub fn admin_set_base_currency(&mut self, base_currency: AccountId) {
        self.assert_admin_or_greater();
        // only "near" allowed for now
        assert_eq!(
            base_currency,
            AccountId::new_unchecked("near".to_string()),
            "Only NEAR is supported"
        );
        self.base_currency = base_currency;
    }

    pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) {
        self.assert_admin_or_greater();
        self.application_start_ms = application_start_ms;
    }

    pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) {
        self.assert_admin_or_greater();
        assert!(
            application_end_ms <= self.public_round_end_ms,
            "Application end must be before public round end"
        );
        self.application_end_ms = application_end_ms;
    }

    pub fn admin_set_public_round_start_ms(&mut self, public_round_start_ms: u64) {
        self.assert_admin_or_greater();
        self.public_round_start_ms = public_round_start_ms;
    }

    pub fn admin_set_public_round_end_ms(&mut self, public_round_end_ms: u64) {
        self.assert_admin_or_greater();
        self.public_round_end_ms = public_round_end_ms;
    }

    pub fn admin_set_public_round_open(&mut self, public_round_end_ms: TimestampMs) {
        self.assert_admin_or_greater();
        self.public_round_start_ms = env::block_timestamp_ms();
        self.public_round_end_ms = public_round_end_ms;
    }

    pub fn admin_set_public_round_closed(&mut self) {
        self.assert_admin_or_greater();
        self.public_round_end_ms = env::block_timestamp_ms();
    }

    pub fn admin_set_registry_provider(&mut self, contract_id: AccountId, method_name: String) {
        self.assert_admin_or_greater();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.registry_provider.set(&provider_id);
    }

    pub fn admin_remove_registry_provider(&mut self) {
        self.assert_admin_or_greater();
        self.registry_provider.remove();
    }

    pub fn admin_set_min_matching_pool_donation_amount(
        &mut self,
        min_matching_pool_donation_amount: U128,
    ) {
        self.assert_admin_or_greater();
        self.min_matching_pool_donation_amount = min_matching_pool_donation_amount;
    }

    pub fn admin_set_sybil_wrapper_provider(
        &mut self,
        contract_id: AccountId,
        method_name: String,
    ) {
        self.assert_admin_or_greater();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.sybil_wrapper_provider.set(&provider_id);
    }

    pub fn admin_remove_sybil_wrapper_provider(&mut self) {
        self.assert_admin_or_greater();
        self.sybil_wrapper_provider.remove();
    }

    pub fn admin_set_custom_sybil_checks(&mut self, custom_sybil_checks: Vec<CustomSybilCheck>) {
        self.assert_admin_or_greater();
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
        self.assert_admin_or_greater();
        self.custom_sybil_checks.remove();
    }

    pub fn admin_set_custom_min_threshold_score(&mut self, custom_min_threshold_score: u32) {
        self.assert_admin_or_greater();
        self.custom_min_threshold_score
            .set(&custom_min_threshold_score);
    }

    pub fn admin_remove_custom_min_threshold_score(&mut self) {
        self.assert_admin_or_greater();
        self.custom_min_threshold_score.remove();
    }

    pub fn admin_set_referral_fee_matching_pool_basis_points(
        &mut self,
        referral_fee_matching_pool_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        self.referral_fee_matching_pool_basis_points = referral_fee_matching_pool_basis_points;
    }

    pub fn admin_set_referral_fee_public_round_basis_points(
        &mut self,
        referral_fee_public_round_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        self.referral_fee_public_round_basis_points = referral_fee_public_round_basis_points;
    }

    pub fn admin_set_cooldown_period_complete(&mut self) {
        self.assert_admin_or_greater();
        self.cooldown_end_ms.set(&env::block_timestamp_ms());
    }
}
