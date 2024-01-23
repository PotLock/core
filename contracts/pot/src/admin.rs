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
    pub max_projects: Option<u32>,
    pub application_start_ms: Option<TimestampMs>,
    pub application_end_ms: Option<TimestampMs>,
    pub public_round_start_ms: Option<TimestampMs>,
    pub public_round_end_ms: Option<TimestampMs>,
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
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.owner = new_owner;
        refund_deposit(initial_storage_usage);
    }

    // ADD/REMOVE ADMINS
    #[payable]
    pub fn owner_add_admins(&mut self, new_admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for new_admin in new_admins.iter() {
            self.admins.insert(new_admin);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins_to_remove: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin_to_remove in admins_to_remove.iter() {
            self.admins.remove(admin_to_remove);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_set_admins(&mut self, account_ids: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        for account_id in account_ids {
            self.admins.insert(&account_id);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_clear_admins(&mut self) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        self.admins.clear();
        refund_deposit(initial_storage_usage);
    }

    // CHEF
    #[payable]
    pub fn admin_set_chef(&mut self, chef: AccountId) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.chef.set(&chef);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_chef(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.chef.remove();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.chef_fee_basis_points = chef_fee_basis_points;
        refund_deposit(initial_storage_usage);
    }

    // POT CONFIG
    #[payable]
    pub fn admin_dangerously_set_pot_config(&mut self, update_args: UpdatePotArgs) -> PotConfig {
        // TODO: CONSIDER REMOVING THIS METHOD DUE TO POTENTIAL FOR MISUSE
        self.assert_admin_or_greater();
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
            self.pot_name = pot_name;
        }
        if let Some(pot_description) = update_args.pot_description {
            self.pot_description = pot_description;
        }
        if let Some(max_projects) = update_args.max_projects {
            self.max_projects = max_projects;
        }
        if let Some(application_start_ms) = update_args.application_start_ms {
            self.application_start_ms = application_start_ms;
        }
        if let Some(application_end_ms) = update_args.application_end_ms {
            assert!(
                application_end_ms <= self.public_round_end_ms,
                "Application end must be before public round end"
            );
            self.application_end_ms = application_end_ms;
        }
        if let Some(public_round_start_ms) = update_args.public_round_start_ms {
            self.public_round_start_ms = public_round_start_ms;
        }
        if let Some(public_round_end_ms) = update_args.public_round_end_ms {
            self.public_round_end_ms = public_round_end_ms;
        }
        if let Some(registry_provider) = update_args.registry_provider {
            self.registry_provider.set(&registry_provider);
        } else {
            self.registry_provider.remove();
        };
        if let Some(min_matching_pool_donation_amount) =
            update_args.min_matching_pool_donation_amount
        {
            self.min_matching_pool_donation_amount = min_matching_pool_donation_amount;
        }
        if let Some(sybil_wrapper_provider) = update_args.sybil_wrapper_provider {
            self.sybil_wrapper_provider.set(&sybil_wrapper_provider);
        } else {
            self.sybil_wrapper_provider.remove();
        };
        if let Some(custom_sybil_checks) = update_args.custom_sybil_checks {
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
            self.referral_fee_matching_pool_basis_points = referral_fee_matching_pool_basis_points;
        }
        if let Some(referral_fee_public_round_basis_points) =
            update_args.referral_fee_public_round_basis_points
        {
            self.referral_fee_public_round_basis_points = referral_fee_public_round_basis_points;
        }
        if let Some(chef_fee_basis_points) = update_args.chef_fee_basis_points {
            self.chef_fee_basis_points = chef_fee_basis_points;
        }

        refund_deposit(initial_storage_usage);

        self.get_config()
    }

    #[payable]
    pub fn admin_set_pot_name(&mut self, pot_name: String) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.pot_name = pot_name;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_pot_description(&mut self, pot_description: String) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.pot_description = pot_description;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_max_projects(&mut self, max_projects: u32) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.max_projects = max_projects;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_base_currency(&mut self, base_currency: AccountId) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        // only "near" allowed for now
        assert_eq!(
            base_currency,
            AccountId::new_unchecked("near".to_string()),
            "Only NEAR is supported"
        );
        self.base_currency = base_currency;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.application_start_ms = application_start_ms;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) {
        self.assert_admin_or_greater();
        assert!(
            application_end_ms <= self.public_round_end_ms,
            "Application end must be before public round end"
        );
        let initial_storage_usage = env::storage_usage();
        self.application_end_ms = application_end_ms;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_public_round_start_ms(&mut self, public_round_start_ms: u64) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.public_round_start_ms = public_round_start_ms;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_public_round_end_ms(&mut self, public_round_end_ms: u64) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.public_round_end_ms = public_round_end_ms;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_public_round_open(&mut self, public_round_end_ms: TimestampMs) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.public_round_start_ms = env::block_timestamp_ms();
        self.public_round_end_ms = public_round_end_ms;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_public_round_closed(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.public_round_end_ms = env::block_timestamp_ms();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_registry_provider(&mut self, contract_id: AccountId, method_name: String) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.registry_provider.set(&provider_id);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_registry_provider(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.registry_provider.remove();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_min_matching_pool_donation_amount(
        &mut self,
        min_matching_pool_donation_amount: U128,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.min_matching_pool_donation_amount = min_matching_pool_donation_amount;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_sybil_wrapper_provider(
        &mut self,
        contract_id: AccountId,
        method_name: String,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        let provider_id = ProviderId::new(contract_id.to_string(), method_name);
        self.sybil_wrapper_provider.set(&provider_id);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_sybil_wrapper_provider(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.sybil_wrapper_provider.remove();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_custom_sybil_checks(&mut self, custom_sybil_checks: Vec<CustomSybilCheck>) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
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
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_custom_sybil_checks(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.custom_sybil_checks.remove();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_custom_min_threshold_score(&mut self, custom_min_threshold_score: u32) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.custom_min_threshold_score
            .set(&custom_min_threshold_score);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_remove_custom_min_threshold_score(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.custom_min_threshold_score.remove();
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_referral_fee_matching_pool_basis_points(
        &mut self,
        referral_fee_matching_pool_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.referral_fee_matching_pool_basis_points = referral_fee_matching_pool_basis_points;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_referral_fee_public_round_basis_points(
        &mut self,
        referral_fee_public_round_basis_points: u32,
    ) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.referral_fee_public_round_basis_points = referral_fee_public_round_basis_points;
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_cooldown_period_complete(&mut self) {
        self.assert_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.cooldown_end_ms.set(&env::block_timestamp_ms());
        refund_deposit(initial_storage_usage);
    }
}
