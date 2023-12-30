use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderId(pub String);

const PROVIDER_ID_DELIMITER: &str = ":"; // separates contract_id and method_name in ProviderId

// Generate ProviderId ("{CONTRACT_ADDRESS}:{METHOD_NAME}") from contract_id and method_name
impl ProviderId {
    pub fn new(contract_id: String, method_name: String) -> Self {
        ProviderId(format!(
            "{}{}{}",
            contract_id, PROVIDER_ID_DELIMITER, method_name
        ))
    }

    pub fn decompose(&self) -> (String, String) {
        let parts: Vec<&str> = self.0.split(PROVIDER_ID_DELIMITER).collect();
        if parts.len() != 2 {
            panic!("Invalid provider ID format. Expected 'contract_id:method_name'.");
        }
        (parts[0].to_string(), parts[1].to_string())
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Provider {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Whether this provider is active (updated by admin)
    pub is_active: bool,
    /// Whether this provider is flagged (updated by admin)
    pub is_flagged: bool,
    /// Admin notes, e.g. reason for flagging or marking inactive
    pub admin_notes: Option<String>,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    /// Custom gas amount required
    pub gas: Option<u64>,
    /// Optional tags
    pub tags: Option<Vec<String>>,
    /// Optional icon URL
    pub icon_url: Option<String>,
    /// Optional external URL
    pub external_url: Option<String>,
    /// User who submitted this provider
    pub submitted_by: AccountId,
    /// Timestamp of when this provider was submitted
    pub submitted_at_ms: TimestampMs,
    /// Total number of times this provider has been used successfully
    pub stamp_count: u64,
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
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderExternal {
    /// Provider ID
    pub provider_id: ProviderId,
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: String,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Whether this provider is active (updated by admin)
    pub is_active: bool,
    /// Whether this provider is flagged (updated by admin)
    pub is_flagged: bool,
    /// Admin notes, e.g. reason for flagging or marking inactive
    pub admin_notes: Option<String>,
    /// Default weight for this provider, e.g. 100
    pub default_weight: u32,
    /// Custom gas amount required
    pub gas: Option<u64>,
    /// Optional tags
    pub tags: Option<Vec<String>>,
    /// Optional icon URL
    pub icon_url: Option<String>,
    /// Optional external URL
    pub external_url: Option<String>,
    /// User who submitted this provider
    pub submitted_by: AccountId,
    /// Timestamp of when this provider was submitted
    pub submitted_at_ms: TimestampMs,
    /// Total number of times this provider has been used successfully
    pub stamp_count: u64,
}

impl ProviderExternal {
    /// Creates a `ProviderExternal` (view-only representation of a Provider) from a provider ID string + Provider.
    ///
    /// # Arguments
    ///
    /// * `provider_id` - A string representing the provider ID.
    /// * `name` - A string representing the name of the provider.
    /// * `default_weight` - A u32 representing the default weight.
    ///
    /// # Returns
    ///
    /// * `ProviderExternal` object.
    pub fn from_provider_id(provider_id: &str, provider: Provider) -> Self {
        let parts: Vec<&str> = provider_id.split(':').collect();
        if parts.len() != 2 {
            panic!("Invalid provider ID format. Expected 'contract_id:method_name'.");
        }

        ProviderExternal {
            provider_id: ProviderId(provider_id.to_string()),
            contract_id: parts[0].to_string(),
            method_name: parts[1].to_string(),
            name: provider.name,
            default_weight: provider.default_weight,
            description: provider.description,
            is_active: provider.is_active,
            is_flagged: provider.is_flagged,
            admin_notes: provider.admin_notes,
            gas: provider.gas,
            tags: provider.tags,
            icon_url: provider.icon_url,
            external_url: provider.external_url,
            submitted_by: provider.submitted_by,
            submitted_at_ms: provider.submitted_at_ms,
            stamp_count: provider.stamp_count,
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register_provider(
        &mut self,
        contract_id: String,
        method_name: String,
        name: String,
        description: Option<String>,
        gas: Option<u64>,
        tags: Option<Vec<String>>,
        icon_url: Option<String>,
        external_url: Option<String>,
    ) -> ProviderExternal {
        // Get initial storage usage so user can pay for what they use
        let initial_storage_usage = env::storage_usage();

        // generate provider ID
        let provider_id = ProviderId::new(contract_id, method_name);

        // check that provider doesn't already exist
        if let Some(_) = self.providers_by_id.get(&provider_id) {
            env::panic_str(&format!("Provider {:#?} already exists", provider_id));
        }

        // create provider
        let mut provider = Provider {
            name,
            description,
            is_active: false,
            is_flagged: false,
            admin_notes: None,
            default_weight: 100,
            gas,
            tags,
            icon_url,
            external_url,
            submitted_by: env::signer_account_id(),
            submitted_at_ms: env::block_timestamp_ms(),
            stamp_count: 0,
        };

        // set provider to active if caller is owner/admin
        if self.is_owner_or_admin() {
            provider.is_active = true;
        }

        // create & store provider
        self.providers_by_id.insert(
            &provider_id,
            &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
        );

        // TODO: consider adding event logging

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        // return provider
        ProviderExternal::from_provider_id(&provider_id.0, provider)
    }

    #[payable]
    pub fn update_provider(
        &mut self,
        provider_id: ProviderId,
        name: Option<String>,
        description: Option<String>,
        gas: Option<u64>,
        tags: Option<Vec<String>>,
        icon_url: Option<String>,
        external_url: Option<String>,
    ) -> ProviderExternal {
        // Ensure caller is Provider submitter or Owner/Admin
        assert!(
            env::signer_account_id()
                == Provider::from(
                    self.providers_by_id
                        .get(&provider_id)
                        .expect(&format!("Provider {:#?} does not exist", provider_id)),
                )
                .submitted_by
                || self.is_owner_or_admin(),
        );

        // Get initial storage usage so user can pay for what they use
        let initial_storage_usage = env::storage_usage();

        // check that provider exists
        let mut provider: Provider = self
            .providers_by_id
            .get(&provider_id)
            .expect(&format!("Provider {:#?} does not exist", provider_id))
            .into();

        // update provider
        if let Some(name) = name {
            provider.name = name;
        }
        if let Some(description) = description {
            provider.description = Some(description);
        }
        if let Some(gas) = gas {
            provider.gas = Some(gas);
        }
        if let Some(tags) = tags {
            provider.tags = Some(tags);
        }
        if let Some(icon_url) = icon_url {
            provider.icon_url = Some(icon_url);
        }
        if let Some(external_url) = external_url {
            provider.external_url = Some(external_url);
        }

        // update & store provider
        self.providers_by_id.insert(
            &provider_id,
            &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
        );

        // Refund any unused deposit
        refund_deposit(initial_storage_usage);

        ProviderExternal::from_provider_id(&provider_id.0, provider)
    }

    // * VIEW METHODS *

    pub fn get_provider(
        &self,
        contract_id: String,
        method_name: String,
    ) -> Option<ProviderExternal> {
        let provider_id = ProviderId::new(contract_id, method_name);
        let provider = self.providers_by_id.get(&provider_id);
        if let Some(provider) = provider {
            Some(ProviderExternal::from_provider_id(
                &provider_id.0,
                Provider::from(provider),
            ))
        } else {
            None
        }
    }

    pub fn get_providers(&self) -> Vec<ProviderExternal> {
        self.providers_by_id
            .iter()
            .map(|(provider_id, provider)| {
                ProviderExternal::from_provider_id(&provider_id.0, Provider::from(provider))
            })
            .collect()
    }

    pub fn get_default_providers(&self) -> Vec<ProviderExternal> {
        self.default_provider_ids
            .iter()
            .map(|provider_id| {
                ProviderExternal::from_provider_id(
                    &provider_id.0,
                    Provider::from(self.providers_by_id.get(&provider_id).unwrap()),
                )
            })
            .collect()
    }
}
