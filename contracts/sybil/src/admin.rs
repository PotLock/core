use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn admin_activate_provider(
        &mut self,
        provider_id: ProviderId,
        default_weight: u32,
    ) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update provider
            let initial_storage_usage = env::storage_usage();
            let mut provider = Provider::from(versioned_provider);
            provider.is_active = true;
            provider.default_weight = default_weight;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            refund_deposit(initial_storage_usage);
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
            let initial_storage_usage = env::storage_usage();
            let mut provider = Provider::from(versioned_provider);
            provider.is_active = false;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            // remove provider from devault providers
            self.default_provider_ids.remove(&provider_id);
            refund_deposit(initial_storage_usage);
            // return provider
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
            let initial_storage_usage = env::storage_usage();
            let mut provider = Provider::from(versioned_provider);
            provider.is_flagged = true;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            refund_deposit(initial_storage_usage);
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
            let initial_storage_usage = env::storage_usage();
            let mut provider = Provider::from(versioned_provider);
            provider.is_flagged = false;
            self.providers_by_id
                .insert(&provider_id, &VersionedProvider::Current(provider.clone()));
            refund_deposit(initial_storage_usage);
            provider
        } else {
            env::panic_str("Provider does not exist");
        }
    }

    #[payable]
    pub fn admin_update_provider_method_name(
        &mut self,
        provider_id: ProviderId,
        method_name: String,
    ) -> Provider {
        self.assert_owner_or_admin();
        // check that provider exists
        if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
            // update its ID by replacing the old key with the new key
            let initial_storage_usage = env::storage_usage();
            let (contract_id, old_method_name) = provider_id.decompose();
            let new_id = ProviderId::new(contract_id, method_name.clone());
            self.providers_by_id.remove(&provider_id);
            self.providers_by_id.insert(&new_id, &versioned_provider);
            refund_deposit(initial_storage_usage);
            Provider::from(self.providers_by_id.get(&new_id).unwrap())
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
    pub fn admin_add_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // add new default providers
        for provider_id in provider_ids {
            self.default_provider_ids.insert(&provider_id);
        }
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
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
    }

    #[payable]
    pub fn admin_set_default_human_threshold(&mut self, default_human_threshold: u32) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        // set default human threshold
        self.default_human_threshold = default_human_threshold;
    }
}
