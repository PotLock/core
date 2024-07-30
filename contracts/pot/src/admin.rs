use std::hash::Hash;

use crate::*;

/// Used ephemerally in view methods
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UpdatePotArgs {
    pub owner: Option<AccountId>,
    pub admins: Option<Vec<AccountId>>,
    pub chef: Option<AccountId>,
    pub pot_name: Option<String>,
    pub pot_description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub max_projects: Option<u32>,
    pub application_start_ms: Option<TimestampMs>,
    pub application_end_ms: Option<TimestampMs>,
    pub public_round_start_ms: Option<TimestampMs>,
    pub public_round_end_ms: Option<TimestampMs>,
    pub compliance_period_ms: Option<TimestampMs>,
    pub registry_provider: Option<ProviderId>,
    pub min_matching_pool_donation_amount: Option<U128>,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
    pub custom_min_threshold_score: Option<u32>,
    pub referral_fee_matching_pool_basis_points: Option<u32>,
    pub referral_fee_public_round_basis_points: Option<u32>,
    pub chef_fee_basis_points: Option<u32>,
}

#[near_bindgen]
impl Contract {
    // CHANGE OWNER
    #[payable]
    pub fn owner_change_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.owner = owner;
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    // ADD/REMOVE ADMINS
    #[payable]
    pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for new_admin in admins.iter() {
            self.admins.insert(new_admin);
        }
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin_to_remove in admins.iter() {
            self.admins.remove(admin_to_remove);
        }
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_set_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        for account_id in admins {
            self.admins.insert(&account_id);
        }
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_clear_admins(&mut self) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    // CHEF
    #[payable]
    pub fn admin_set_chef(&mut self, chef: AccountId) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.chef.set(&chef);
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_chef(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.chef.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) {
        self.assert_admin_or_greater();
        assert_valid_chef_fee_basis_points(chef_fee_basis_points);
        self.chef_fee_basis_points = chef_fee_basis_points;
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_add_blacklisted_donors(&mut self, donor_ids: Vec<AccountId>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        for donor_id in donor_ids.iter() {
            self.blacklisted_donors.insert(donor_id);
        }
        log_update_blacklisted_donors_event();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_blacklisted_donors(&mut self, donor_ids: Vec<AccountId>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.blacklisted_donors.clear();
        for donor_id in donor_ids.iter() {
            self.blacklisted_donors.insert(donor_id);
        }
        log_update_blacklisted_donors_event();
        refund_deposit(initial_storage_usage);
    }

    // POT CONFIG
    #[payable]
    pub fn admin_set_pot_name(&mut self, pot_name: String) {
        self.assert_admin_or_greater();
        assert_valid_pot_name(&pot_name);
        let initial_storage_usage = env::storage_usage();
        self.pot_name = pot_name;
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_pot_description(&mut self, pot_description: String) {
        self.assert_admin_or_greater();
        assert_valid_pot_description(&pot_description);
        let initial_storage_usage = env::storage_usage();
        self.pot_description = pot_description;
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_tags(&mut self, tags: Vec<String>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.tags = tags;
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_max_projects(&mut self, max_projects: u32) {
        self.assert_admin_or_greater();
        assert_valid_max_projects(max_projects);
        self.max_projects = max_projects;
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_base_currency(&mut self, base_currency: AccountId) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        // only "near" allowed for now
        // TODO: once other currencies are supported, add checks for valid FT contract
        assert_eq!(
            base_currency,
            AccountId::new_unchecked("near".to_string()),
            "Only NEAR is supported"
        );
        self.base_currency = base_currency;
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_round_timestamps(
        &mut self,
        application_start_ms: Option<TimestampMs>,
        application_end_ms: Option<TimestampMs>,
        public_round_start_ms: Option<TimestampMs>,
        public_round_end_ms: Option<TimestampMs>,
    ) {
        self.assert_admin_or_greater();
        self.assert_valid_timestamps(
            application_start_ms,
            application_end_ms,
            public_round_start_ms,
            public_round_end_ms,
        );
        if let Some(application_start_ms) = application_start_ms {
            self.application_start_ms = application_start_ms;
        }
        if let Some(application_end_ms) = application_end_ms {
            self.application_end_ms = application_end_ms;
        }
        if let Some(public_round_start_ms) = public_round_start_ms {
            self.public_round_start_ms = public_round_start_ms;
        }
        if let Some(public_round_end_ms) = public_round_end_ms {
            self.public_round_end_ms = public_round_end_ms;
        }
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_compliance_period_ms(&mut self, compliance_period_ms: TimestampMs) {
        self.assert_admin_or_greater();
        self.compliance_period_ms.set(&compliance_period_ms);
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_remaining_funds_redistribution_recipient(&mut self, account_id: AccountId) {
        self.assert_admin_or_greater();
        self.assert_round_not_started(); // can only be before public round starts
        let initial_storage_usage = env::storage_usage();
        self.remaining_funds_redistribution_recipient
            .set(&account_id);
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_remaining_funds_redistribution_recipient(&mut self) {
        self.assert_admin_or_greater();
        self.assert_round_not_started(); // can only be before public round starts
        let initial_storage_usage = env::storage_usage();
        self.remaining_funds_redistribution_recipient.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_registry_provider(&mut self, contract_id: AccountId, method_name: String) {
        self.assert_admin_or_greater();
        // TODO: validate contract_id and method_name by calling method
        let initial_storage_usage = env::storage_usage();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.registry_provider.set(&provider_id);
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_registry_provider(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.registry_provider.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_min_matching_pool_donation_amount(
        &mut self,
        min_matching_pool_donation_amount: U128,
    ) {
        self.assert_admin_or_greater();
        self.min_matching_pool_donation_amount = min_matching_pool_donation_amount.0;
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_sybil_wrapper_provider(
        &mut self,
        contract_id: AccountId,
        method_name: String,
    ) {
        self.assert_admin_or_greater();
        // TODO: validate contract_id and method_name by calling method
        let initial_storage_usage = env::storage_usage();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.sybil_wrapper_provider.set(&provider_id);
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_sybil_wrapper_provider(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.sybil_wrapper_provider.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_custom_sybil_checks(&mut self, custom_sybil_checks: Vec<CustomSybilCheck>) {
        self.assert_admin_or_greater();
        // TODO: validate sybil checks
        let initial_storage_usage = env::storage_usage();
        let formatted_custom_sybil_checks: HashMap<ProviderId, SybilProviderWeight> =
            custom_sybil_checks
                .into_iter()
                .map(|custom_sybil_check| {
                    let provider_id = ProviderId::new(
                        custom_sybil_check.contract_id.to_string(),
                        custom_sybil_check.method_name,
                    );
                    provider_id.validate();
                    (provider_id, custom_sybil_check.weight)
                })
                .collect();
        self.custom_sybil_checks.set(&formatted_custom_sybil_checks);
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_custom_sybil_checks(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.custom_sybil_checks.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_custom_min_threshold_score(&mut self, custom_min_threshold_score: u32) {
        self.assert_admin_or_greater();
        self.custom_min_threshold_score
            .set(&custom_min_threshold_score);
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_remove_custom_min_threshold_score(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.custom_min_threshold_score.remove();
        log_update_pot_config_event(&self.get_config());
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_referral_fee_matching_pool_basis_points(
        &mut self,
        referral_fee_matching_pool_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        assert_valid_referral_fee_matching_pool_basis_points(
            referral_fee_matching_pool_basis_points,
        );
        self.referral_fee_matching_pool_basis_points = referral_fee_matching_pool_basis_points;
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_referral_fee_public_round_basis_points(
        &mut self,
        referral_fee_public_round_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        assert_valid_referral_fee_public_round_basis_points(referral_fee_public_round_basis_points);
        self.referral_fee_public_round_basis_points = referral_fee_public_round_basis_points;
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_set_cooldown_end_ms(&mut self, cooldown_end_ms: TimestampMs) {
        self.assert_admin_or_greater();
        // cooldown must be in process (self.cooldown_end_ms must be Some value)
        // also, cooldown_end_ms must be greater than self.cooldown_end_ms (can only extend cooldown, not reduce)
        assert!(
            self.cooldown_end_ms.get().is_some(),
            "Cooldown period not in process"
        );
        assert!(
            cooldown_end_ms > self.cooldown_end_ms.get().unwrap(),
            "Cooldown period can only be extended"
        );
        self.cooldown_end_ms.set(&cooldown_end_ms);
        log_update_pot_config_event(&self.get_config());
    }

    #[payable]
    pub fn admin_update_payouts_challenge(
        &mut self,
        challenger_id: AccountId,
        notes: Option<String>,
        resolve_challenge: Option<bool>,
    ) {
        self.assert_admin_or_greater();
        if let Some(mut payouts_challenge) = self.payouts_challenges.get(&challenger_id) {
            let initial_storage_usage = env::storage_usage();
            if let Some(notes) = notes {
                payouts_challenge.admin_notes = Some(notes);
            }
            payouts_challenge.resolved = resolve_challenge.unwrap_or(payouts_challenge.resolved);
            self.payouts_challenges
                .insert(&challenger_id, &payouts_challenge);
            refund_deposit(initial_storage_usage);
        } else {
            // panic so that indexer doesn't pick this up as a successful transaction (this was the case in V1)
            panic!("Payouts challenge not found");
        }
    }

    pub fn admin_remove_resolved_payouts_challenges(&mut self) {
        self.assert_admin_or_greater();
        self.assert_cooldown_period_complete();
        let removal_limit = 10; // conservative limit to prevent exceeding gas limit, which would fail to remove the challenges but still refund the storage cost
        let mut to_remove = vec![];
        let mut storage_refunds: HashMap<AccountId, u128> = HashMap::new();
        for (challenger_id, payouts_challenge_versioned) in self.payouts_challenges.iter() {
            let payouts_challenge = PayoutsChallenge::from(payouts_challenge_versioned);
            if payouts_challenge.resolved {
                to_remove.push(challenger_id);
                if to_remove.len() >= removal_limit {
                    break;
                }
            }
        }
        for challenger_id in to_remove {
            let storage_before = env::storage_usage();
            self.payouts_challenges.remove(&challenger_id);
            let refund = calculate_required_storage_deposit(storage_before);
            storage_refunds.insert(challenger_id, refund);
        }
        for (challenger_id, refund) in storage_refunds {
            Promise::new(challenger_id).transfer(refund);
        }
    }

    #[payable]
    pub fn admin_dangerously_set_pot_config(&mut self, update_args: UpdatePotArgs) -> PotConfig {
        // TODO: CONSIDER REMOVING THIS METHOD DUE TO POTENTIAL FOR MISUSE
        self.assert_admin_or_greater();
        // validate args
        self.assert_valid_pot_args(&update_args);
        // proceed with updates
        let initial_storage_usage = env::storage_usage();
        if let Some(owner) = update_args.owner {
            if env::signer_account_id() == self.owner {
                // only update owner if caller is owner
                self.owner = owner;
            }
        }
        if let Some(admins) = update_args.admins {
            // clear existing admins and reset to IDs provided
            self.admins.clear();
            for admin in admins.iter() {
                self.admins.insert(admin);
            }
        }
        // set chef to provided ID or remove if not present
        if let Some(chef) = update_args.chef {
            self.chef.set(&chef);
        } else {
            self.chef.remove();
        };
        if let Some(pot_name) = update_args.pot_name {
            assert_valid_pot_name(&pot_name);
            self.pot_name = pot_name;
        }
        if let Some(pot_description) = update_args.pot_description {
            assert_valid_pot_description(&pot_description);
            self.pot_description = pot_description;
        }
        if let Some(tags) = update_args.tags {
            self.tags = tags;
        }
        if let Some(max_projects) = update_args.max_projects {
            assert_valid_max_projects(max_projects);
            self.max_projects = max_projects;
        }
        // validate timestamps
        self.assert_valid_timestamps(
            update_args.application_start_ms,
            update_args.application_end_ms,
            update_args.public_round_start_ms,
            update_args.public_round_end_ms,
        );
        if let Some(application_start_ms) = update_args.application_start_ms {
            self.application_start_ms = application_start_ms;
        }
        if let Some(application_end_ms) = update_args.application_end_ms {
            self.application_end_ms = application_end_ms;
        }
        if let Some(public_round_start_ms) = update_args.public_round_start_ms {
            self.public_round_start_ms = public_round_start_ms;
        }
        if let Some(public_round_end_ms) = update_args.public_round_end_ms {
            self.public_round_end_ms = public_round_end_ms;
        }
        if let Some(compliance_period_ms) = update_args.compliance_period_ms {
            self.compliance_period_ms.set(&compliance_period_ms);
        }
        if let Some(registry_provider) = update_args.registry_provider {
            registry_provider.validate();
            // TODO: validate contract_id and method_name further by calling method
            self.registry_provider.set(&registry_provider);
        } else {
            self.registry_provider.remove();
        };
        if let Some(min_matching_pool_donation_amount) =
            update_args.min_matching_pool_donation_amount
        {
            self.min_matching_pool_donation_amount = min_matching_pool_donation_amount.0;
        }
        if let Some(sybil_wrapper_provider) = update_args.sybil_wrapper_provider {
            sybil_wrapper_provider.validate();
            // TODO: validate contract_id and method_name further by calling method
            self.sybil_wrapper_provider.set(&sybil_wrapper_provider);
        } else {
            self.sybil_wrapper_provider.remove();
        };
        if let Some(custom_sybil_checks) = update_args.custom_sybil_checks {
            // TODO: validate sybil checks further by calling method
            let formatted_custom_sybil_checks: HashMap<ProviderId, SybilProviderWeight> =
                custom_sybil_checks
                    .into_iter()
                    .map(|custom_sybil_check| {
                        let provider_id = ProviderId::new(
                            custom_sybil_check.contract_id.to_string(),
                            custom_sybil_check.method_name,
                        );
                        provider_id.validate();
                        (provider_id, custom_sybil_check.weight)
                    })
                    .collect();
            self.custom_sybil_checks.set(&formatted_custom_sybil_checks);
        } else {
            self.custom_sybil_checks.remove();
        };
        if let Some(custom_min_threshold_score) = update_args.custom_min_threshold_score {
            self.custom_min_threshold_score
                .set(&custom_min_threshold_score);
        } else {
            self.custom_min_threshold_score.remove();
        };
        if let Some(referral_fee_matching_pool_basis_points) =
            update_args.referral_fee_matching_pool_basis_points
        {
            assert_valid_referral_fee_matching_pool_basis_points(
                referral_fee_matching_pool_basis_points,
            );
            self.referral_fee_matching_pool_basis_points = referral_fee_matching_pool_basis_points;
        }
        if let Some(referral_fee_public_round_basis_points) =
            update_args.referral_fee_public_round_basis_points
        {
            assert_valid_referral_fee_public_round_basis_points(
                referral_fee_public_round_basis_points,
            );
            self.referral_fee_public_round_basis_points = referral_fee_public_round_basis_points;
        }
        if let Some(chef_fee_basis_points) = update_args.chef_fee_basis_points {
            assert_valid_chef_fee_basis_points(chef_fee_basis_points);
            self.chef_fee_basis_points = chef_fee_basis_points;
        }

        let config = self.get_config();

        log_update_pot_config_event(&config);

        refund_deposit(initial_storage_usage);

        config
    }
}
