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
    ) -> Promise {
        // generate provider ID
        let provider_id = ProviderId::new(contract_id.clone(), method_name.clone());

        // check that provider doesn't already exist
        if let Some(_) = self.providers_by_id.get(&provider_id) {
            env::panic_str(&format!("Provider {:#?} already exists", provider_id));
        }

        // validate name
        assert_valid_provider_name(&name);

        // validate description
        if let Some(description) = &description {
            assert_valid_provider_description(description);
        }

        // validate gas
        if let Some(gas) = &gas {
            assert_valid_provider_gas(gas);
        }

        // validate tags
        if let Some(tags) = &tags {
            assert_valid_provider_tags(tags);
        }

        // validate icon_url
        if let Some(icon_url) = &icon_url {
            assert_valid_provider_icon_url(icon_url);
        }

        // validate external_url
        if let Some(external_url) = &external_url {
            assert_valid_provider_external_url(external_url);
        }

        let submitter_id = env::signer_account_id();

        // create provider (but don't store yet)
        let mut provider = Provider {
            name,
            description,
            is_active: false,
            is_flagged: false,
            admin_notes: None,
            default_weight: PROVIDER_DEFAULT_WEIGHT,
            gas,
            tags,
            icon_url,
            external_url,
            submitted_by: submitter_id.clone(),
            submitted_at_ms: env::block_timestamp_ms(),
            stamp_count: 0,
        };

        // set provider to active if caller is owner/admin
        if self.is_owner_or_admin() {
            provider.is_active = true;
        }

        // validate contract ID and method name
        let gas = Gas(gas.unwrap_or(XCC_GAS_DEFAULT));
        let args = json!({ "account_id": env::current_account_id() })
            .to_string()
            .into_bytes();
        Promise::new(AccountId::new_unchecked(contract_id.clone()))
            .function_call(method_name.clone(), args, NO_DEPOSIT, gas)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(gas)
                    .verify_provider_callback(
                        submitter_id,
                        provider_id,
                        provider,
                        env::attached_deposit(),
                    ),
            )
    }

    #[private]
    pub fn verify_provider_callback(
        &mut self,
        submitter_id: AccountId,
        provider_id: ProviderId,
        provider: Provider,
        attached_deposit: Balance,
        #[callback_result] call_result: Result<near_sdk::serde_json::Value, PromiseError>,
    ) -> Option<ProviderExternal> {
        match call_result {
            Ok(val) => {
                if val.as_bool().is_some() {
                    // provider returns a bool; proceed with normal processing
                    // Get initial storage usage so submitter can pay for what they use
                    let initial_storage_usage = env::storage_usage();

                    // create & store provider
                    self.providers_by_id.insert(
                        &provider_id,
                        &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
                    );

                    // log event
                    log_add_provider_event(&provider_id, &provider);

                    // calculate storage cost
                    let required_deposit =
                        calculate_required_storage_deposit(initial_storage_usage);
                    // refund any unused deposit
                    if attached_deposit > required_deposit {
                        Promise::new(submitter_id.clone())
                            .transfer(attached_deposit - required_deposit);
                    } else if attached_deposit < required_deposit {
                        env::panic_str(&format!(
                            "Must attach {} yoctoNEAR to cover storage",
                            required_deposit
                        ));
                    }

                    // return provider
                    Some(ProviderExternal::from_provider_id(&provider_id.0, provider))
                } else {
                    // Response type is incorrect. Refund deposit.
                    log!("Received invalid response type for provider verification. Returning deposit.");
                    Promise::new(submitter_id).transfer(attached_deposit);
                    return None;
                }
            }
            Err(_) => {
                // Error occurred in cross-contract call. Refund deposit.
                log!("Error occurred while verifying provider; refunding deposit");
                Promise::new(submitter_id).transfer(attached_deposit);
                return None;
            }
        }
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
        default_weight: Option<u32>, // owner/admin-only
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
            assert_valid_provider_name(&name);
            provider.name = name;
        }
        if let Some(description) = description {
            assert_valid_provider_description(&description);
            provider.description = Some(description);
        }
        if let Some(gas) = gas {
            assert_valid_provider_gas(&gas);
            provider.gas = Some(gas);
        }
        if let Some(tags) = tags {
            assert_valid_provider_tags(&tags);
            provider.tags = Some(tags);
        }
        if let Some(icon_url) = icon_url {
            assert_valid_provider_icon_url(&icon_url);
            provider.icon_url = Some(icon_url);
        }
        if let Some(external_url) = external_url {
            assert_valid_provider_external_url(&external_url);
            provider.external_url = Some(external_url);
        }

        // owner/admin-only
        if self.is_owner_or_admin() {
            if let Some(default_weight) = default_weight {
                provider.default_weight = default_weight;
            }
        }

        // update & store provider
        self.providers_by_id.insert(
            &provider_id,
            &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
        );

        // Refund any unused deposit
        refund_deposit(initial_storage_usage);

        // log event
        log_update_provider_event(&provider_id, &provider);

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

    pub fn get_providers(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ProviderExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.providers_by_id.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.providers_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit.try_into().unwrap())
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
