use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn admin_update_provider(
        &mut self,
        provider_id: ProviderId,
        provider: Provider,
    ) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(_) = self.providers_by_id.get(&provider_id) {
            // update provider
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    // convenience methods

    #[payable]
    pub fn admin_activate_provider(&mut self, provider_id: ProviderId) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let mut provider = Provider::from(versioned_provider);
            provider.is_active = true;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    #[payable]
    pub fn admin_deactivate_provider(&mut self, provider_id: ProviderId) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let mut provider = Provider::from(versioned_provider);
            provider.is_active = false;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    #[payable]
    pub fn admin_flag_provider(&mut self, provider_id: ProviderId) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let mut provider = Provider::from(versioned_provider);
            provider.is_flagged = true;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    #[payable]
    pub fn admin_unflag_provider(&mut self, provider_id: ProviderId) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let mut provider = Provider::from(versioned_provider);
            provider.is_flagged = false;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
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
        for provider_id in provider_ids {
            self.default_provider_ids.insert(&provider_id);
        }
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_add_default_providers(&mut self, provider_id: ProviderId) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // add new default provider
        self.default_provider_ids.insert(&provider_id);
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_default_human_threshold(&mut self, default_human_threshold: u32) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        // set default human threshold
        self.default_human_threshold = default_human_threshold;
    }
}
