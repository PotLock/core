use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderId(pub String);

const PROVIDER_ID_DELIMITER: &str = ":"; // separates contract_id and method_name in ProviderId

// Generate ProviderId ("{CONTRACT_ADDRESS}:{METHOD_NAME}") from contract_id and method_name
impl ProviderId {
    fn new(contract_id: String, method_name: String) -> Self {
        ProviderId(format!(
            "{}{}{}",
            contract_id, PROVIDER_ID_DELIMITER, method_name
        ))
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Provider {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    // TODO: consider adding optional `gas`, `type`/`description` (e.g. "face scan", "twitter", "captcha", etc.), `icon`, `external_url`
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedProvider {
    Current(Provider),
}

impl From<VersionedProvider> for Provider {
    fn from(provider: VersionedProvider) -> Self {
        match provider {
            VersionedProvider::Current(current) => current,
        }
    }
}

// external/ephemeral Provider that contains contract_id and method_name
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderJson {
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: String,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
}

impl ProviderJson {
    /// Creates a `ProviderJson` (view-only representation of a Provider) from a provider ID string + Provider.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - A string representing the provider ID.
    /// * `name` - A string representing the name of the provider.
    /// * `default_weight` - A u32 representing the default weight.
    ///
    /// # Returns
    ///
    /// * `ProviderJson` object.
    pub fn from_provider_id(provider_id: &str, provider: Provider) -> Self {
        let parts: Vec<&str> = provider_id.split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid provider ID format. Expected 'contract_id:method_name'.");
        }

        ProviderJson {
            contract_id: parts[0].to_string(),
            method_name: parts[1].to_string(),
            name: provider.name,
            default_weight: provider.default_weight,
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn add_provider(
        &mut self,
        contract_id: String,
        method_name: String,
        name: String,
        default_weight: u32,
    ) -> ProviderId {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // generate provider ID
        let provider_id = ProviderId::new(contract_id, method_name);
        // create provider
        let provider = Provider {
            name,
            default_weight,
        };
        // store provider
        self.providers_by_id.insert(
            &provider_id,
            &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
        );

        // TODO: consider adding event logging

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        // return provider ID
        provider_id
    }

    #[payable]
    pub fn set_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
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
    pub fn add_default_provider(&mut self, provider_id: ProviderId) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        // add new default provider
        self.default_provider_ids.insert(&provider_id);
        // refund any unused deposit
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn set_default_human_threshold(&mut self, default_human_threshold: u32) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        // set default human threshold
        self.default_human_threshold = default_human_threshold;
    }

    // * VIEW METHODS *

    pub fn get_provider(&self, contract_id: String, method_name: String) -> Option<ProviderJson> {
        let provider_id = ProviderId::new(contract_id, method_name);
        let provider = self.providers_by_id.get(&provider_id);
        if let Some(provider) = provider {
            Some(ProviderJson::from_provider_id(
                &provider_id.0,
                Provider::from(provider),
            ))
        } else {
            None
        }
    }

    pub fn get_providers(&self) -> Vec<ProviderJson> {
        self.providers_by_id
            .iter()
            .map(|(provider_id, provider)| {
                ProviderJson::from_provider_id(&provider_id.0, Provider::from(provider))
            })
            .collect()
    }

    pub fn get_default_providers(&self) -> Vec<ProviderJson> {
        self.default_provider_ids
            .iter()
            .map(|provider_id| {
                ProviderJson::from_provider_id(
                    &provider_id.0,
                    Provider::from(self.providers_by_id.get(&provider_id).unwrap()),
                )
            })
            .collect()
    }
}
