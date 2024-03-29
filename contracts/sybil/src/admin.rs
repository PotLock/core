use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn admin_update_provider_status(
        &mut self,
        provider_id: ProviderId,
        status: ProviderStatus,
    ) -> ProviderExternal {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let initial_storage_usage = env::storage_usage();
            let mut provider = Provider::from(versioned_provider);
            let old_status = provider.status;
            provider.status = status.clone();
            // add provider to mapping
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            // remove provider from old status set
            match old_status {
                ProviderStatus::Pending => {
                    self.pending_provider_ids.remove(&provider_id);
                }
                ProviderStatus::Active => {
                    self.active_provider_ids.remove(&provider_id);
                }
                ProviderStatus::Deactivated => {
                    self.deactivated_provider_ids.remove(&provider_id);
                }
            }
            // add provider to new status set
            match status {
                ProviderStatus::Pending => {
                    self.pending_provider_ids.insert(&provider_id);
                }
                ProviderStatus::Active => {
                    self.active_provider_ids.insert(&provider_id);
                }
                ProviderStatus::Deactivated => {
                    self.deactivated_provider_ids.insert(&provider_id);
                }
            }
            refund_deposit(initial_storage_usage);

            let formatted_provider = format_provider(&provider_id, &provider);
            // log event
            log_add_or_update_provider_event(&formatted_provider);
            formatted_provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    #[payable]
    pub fn admin_activate_provider(&mut self, provider_id: ProviderId) -> ProviderExternal {
        self.admin_update_provider_status(provider_id, ProviderStatus::Active)
    }

    #[payable]
    pub fn admin_deactivate_provider(&mut self, provider_id: ProviderId) -> ProviderExternal {
        self.admin_update_provider_status(provider_id, ProviderStatus::Deactivated)
    }

    // config

    #[payable]
    pub fn admin_set_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // clear existing default providers
        self.default_provider_ids.clear();
        // add new default providers
        for provider_id in provider_ids.clone() {
            self.default_provider_ids.insert(&provider_id);
        }
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
        log_update_default_providers_event(self.default_provider_ids.iter().collect());
    }

    #[payable]
    pub fn admin_add_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // add new default providers
        for provider_id in provider_ids.clone() {
            self.default_provider_ids.insert(&provider_id);
        }
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
        log_update_default_providers_event(self.default_provider_ids.iter().collect());
    }

    #[payable]
    pub fn admin_remove_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // remove default providers
        for provider_id in provider_ids {
            self.default_provider_ids.remove(&provider_id);
        }
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
        log_update_default_providers_event(self.default_provider_ids.iter().collect());
    }

    #[payable]
    pub fn admin_clear_default_providers(&mut self) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // clear default providers
        self.default_provider_ids.clear();
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
        log_update_default_providers_event(self.default_provider_ids.iter().collect());
    }

    #[payable]
    pub fn admin_set_default_human_threshold(&mut self, default_human_threshold: u32) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // set default human threshold
        self.default_human_threshold = default_human_threshold;
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
        log_update_default_human_threshold_event(default_human_threshold);
    }
}
