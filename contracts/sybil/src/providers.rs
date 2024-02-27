use crate::*;
// use serde_json::{json, Value as JsonValue};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProviderStatus {
    Pending,
    Active,
    Deactivated,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderV1 {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderV2 {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of account ID arg, e.g. `"account_id"` or `"accountId"` or `"account"`
    pub account_id_arg_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Provider {
    // NB: contract address/ID and method name are contained in the Provider's ID (see `ProviderId`) so do not need to be stored here
    /// Name of account ID arg, e.g. `"account_id"` or `"accountId"` or `"account"`
    pub account_id_arg_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
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
    /// Milliseconds that stamps from this provider are valid for before they expire
    pub stamp_validity_ms: Option<u64>,
    /// Custom args as Base64VecU8
    pub custom_args: Option<Base64VecU8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedProvider {
    V1(ProviderV1),
    V2(ProviderV2),
    Current(Provider),
}

impl From<VersionedProvider> for Provider {
    fn from(provider: VersionedProvider) -> Self {
        match provider {
            VersionedProvider::V1(v1) => Provider {
                account_id_arg_name: "account_id".to_string(),
                name: v1.name,
                description: v1.description,
                status: v1.status,
                admin_notes: v1.admin_notes,
                default_weight: v1.default_weight,
                gas: v1.gas,
                tags: v1.tags,
                icon_url: v1.icon_url,
                external_url: v1.external_url,
                submitted_by: v1.submitted_by,
                submitted_at_ms: v1.submitted_at_ms,
                stamp_count: v1.stamp_count,
                stamp_validity_ms: None,
                custom_args: None,
            },
            VersionedProvider::V2(v2) => Provider {
                account_id_arg_name: v2.account_id_arg_name,
                name: v2.name,
                description: v2.description,
                status: v2.status,
                admin_notes: v2.admin_notes,
                default_weight: v2.default_weight,
                gas: v2.gas,
                tags: v2.tags,
                icon_url: v2.icon_url,
                external_url: v2.external_url,
                submitted_by: v2.submitted_by,
                submitted_at_ms: v2.submitted_at_ms,
                stamp_count: v2.stamp_count,
                stamp_validity_ms: None,
                custom_args: None,
            },
            VersionedProvider::Current(current) => current,
        }
    }
}

// external/ephemeral Provider that contains contract_id and method_name
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderExternal {
    /// Provider ID
    pub provider_id: ProviderId,
    /// Contract ID of the external contract that is the source of this provider
    pub contract_id: String,
    /// Method name of the external contract that is the source of this provider
    pub method_name: String,
    /// Account ID arg name
    pub account_id_arg_name: String,
    /// Name of the provider, e.g. "I Am Human"
    pub name: String,
    /// Description of the provider
    pub description: Option<String>,
    /// Status of the provider
    pub status: ProviderStatus,
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
    /// Milliseconds that stamps from this provider are valid for before they expire
    pub stamp_validity_ms: Option<u64>,
    /// Custom args as readable JSON
    pub custom_args: Option<JsonValue>, // This will hold the readable JSON
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
            account_id_arg_name: provider.account_id_arg_name,
            name: provider.name,
            default_weight: provider.default_weight,
            description: provider.description,
            status: provider.status,
            admin_notes: provider.admin_notes,
            gas: provider.gas,
            tags: provider.tags,
            icon_url: provider.icon_url,
            external_url: provider.external_url,
            submitted_by: provider.submitted_by,
            submitted_at_ms: provider.submitted_at_ms,
            stamp_count: provider.stamp_count,
            stamp_validity_ms: provider.stamp_validity_ms,
            custom_args: provider.custom_args.as_ref().map(|base64_data| {
                // Decode Base64VecU8 to Vec<u8>
                let bytes = &base64_data.0; // Access the Vec<u8> directly

                // Parse the byte array as JSON
                near_sdk::serde_json::from_slice(bytes)
                    .unwrap_or_else(|_| json!({ "error": "Invalid JSON" }))
            }),
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
        account_id_arg_name: Option<String>, // defaults to "account_id" if None
        name: String,
        description: Option<String>,
        gas: Option<u64>,
        tags: Option<Vec<String>>,
        icon_url: Option<String>,
        external_url: Option<String>,
        stamp_validity_ms: Option<u64>,
        custom_args: Option<Base64VecU8>,
        default_weight: Option<u32>, // owner/admin-only
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
            account_id_arg_name: account_id_arg_name.unwrap_or("account_id".to_string()),
            name,
            description,
            status: ProviderStatus::Pending,
            admin_notes: None,
            default_weight: PROVIDER_DEFAULT_WEIGHT,
            gas,
            tags,
            icon_url,
            external_url,
            stamp_validity_ms,
            submitted_by: submitter_id.clone(),
            submitted_at_ms: env::block_timestamp_ms(),
            stamp_count: 0,
            custom_args,
        };

        if self.is_owner_or_admin() {
            if let Some(default_weight) = default_weight {
                provider.default_weight = default_weight;
            }

            provider.status = ProviderStatus::Active;
        }

        // TODO: consider setting status to active by default if caller is owner/admin

        // validate contract ID, method name and account ID arg name
        let gas = Gas(gas.unwrap_or(XCC_GAS_DEFAULT));
        // Create a HashMap and insert the dynamic account_id_arg_name and value
        let mut args_map = std::collections::HashMap::new();

        if let Some(custom_args_base64) = provider.clone().custom_args {
            // Decode custom_args from Base64 to Vec<u8>, then to String, and finally parse as JSON Value
            let custom_args_bytes = custom_args_base64.0;
            let custom_args_str =
                String::from_utf8(custom_args_bytes).expect("Invalid UTF-8 sequence");
            let custom_args_json: JsonValue =
                near_sdk::serde_json::from_str(&custom_args_str).expect("Invalid JSON format");

            // Ensure custom_args_json is an object before attempting to spread its contents
            if let JsonValue::Object(contents) = custom_args_json {
                for (key, value) in contents {
                    // Spread custom_args into args_map, converting JSON Values back to Strings or other appropriate formats
                    args_map.insert(key, value);
                }
            }
        }

        // Wrap the account_id string in a serde_json::Value::String before inserting
        args_map.insert(
            provider.account_id_arg_name.clone(),
            near_sdk::serde_json::Value::String(env::current_account_id().to_string()),
        );

        // Serialize the HashMap to JSON string and then to bytes
        let args = near_sdk::serde_json::to_string(&args_map)
            .expect("Failed to serialize args")
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
                    self.handle_store_provider(provider_id.clone(), provider.clone());

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
                    None
                }
            }
            Err(_) => {
                // Error occurred in cross-contract call. Refund deposit.
                log!("Error occurred while verifying provider; refunding deposit");
                Promise::new(submitter_id).transfer(attached_deposit);
                None
            }
        }
    }

    #[payable]
    pub fn update_provider(
        &mut self,
        provider_id: ProviderId,
        account_id_arg_name: Option<String>,
        name: Option<String>,
        description: Option<String>,
        gas: Option<u64>,
        tags: Option<Vec<String>>,
        icon_url: Option<String>,
        external_url: Option<String>,
        stamp_validity_ms: Option<u64>,
        custom_args: Option<Base64VecU8>,
        default_weight: Option<u32>,    // owner/admin-only
        status: Option<ProviderStatus>, // owner/admin-only
        admin_notes: Option<String>,    // owner/admin-only
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
        if let Some(account_id_arg_name) = account_id_arg_name {
            provider.account_id_arg_name = account_id_arg_name;
            // TODO: validate account_id_arg_name against provider contract
        }

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
        if let Some(stamp_validity_ms) = stamp_validity_ms {
            provider.stamp_validity_ms = Some(stamp_validity_ms);
        }
        if let Some(custom_args) = custom_args {
            provider.custom_args = Some(custom_args);
        }

        // owner/admin-only
        if self.is_owner_or_admin() {
            if let Some(default_weight) = default_weight {
                provider.default_weight = default_weight;
            }
            if let Some(status) = status {
                if status != provider.status {
                    let old_status = provider.status.clone();
                    // remove from old status set
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
                    provider.status = status;
                }
            }
            if let Some(admin_notes) = admin_notes {
                provider.admin_notes = Some(admin_notes);
            }
        }

        // update & store provider
        self.handle_store_provider(provider_id.clone(), provider.clone());

        // Refund any unused deposit
        refund_deposit(initial_storage_usage);

        // log event
        log_update_provider_event(&provider_id, &provider);

        ProviderExternal::from_provider_id(&provider_id.0, provider)
    }

    pub(crate) fn handle_store_provider(&mut self, provider_id: ProviderId, provider: Provider) {
        self.providers_by_id.insert(
            &provider_id,
            &VersionedProvider::from(VersionedProvider::Current(provider.clone())),
        );
        match provider.status {
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
        status: Option<ProviderStatus>,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ProviderExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        if let Some(status) = status {
            match status {
                ProviderStatus::Pending => {
                    assert!(
                        (self.pending_provider_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.pending_provider_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|provider_id| {
                            ProviderExternal::from_provider_id(
                                &provider_id.0,
                                Provider::from(self.providers_by_id.get(&provider_id).unwrap()),
                            )
                        })
                        .collect()
                }
                ProviderStatus::Active => {
                    assert!(
                        (self.active_provider_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.active_provider_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|provider_id| {
                            ProviderExternal::from_provider_id(
                                &provider_id.0,
                                Provider::from(self.providers_by_id.get(&provider_id).unwrap()),
                            )
                        })
                        .collect()
                }
                ProviderStatus::Deactivated => {
                    assert!(
                        (self.deactivated_provider_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.deactivated_provider_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|provider_id| {
                            ProviderExternal::from_provider_id(
                                &provider_id.0,
                                Provider::from(self.providers_by_id.get(&provider_id).unwrap()),
                            )
                        })
                        .collect()
                }
            }
        } else {
            assert!(
                (self.providers_by_id.len() as u64) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            self.providers_by_id
                .iter()
                .skip(start_index as usize)
                .take(limit.try_into().unwrap())
                .map(|(provider_id, provider)| {
                    ProviderExternal::from_provider_id(&provider_id.0, Provider::from(provider))
                })
                .collect()
        }
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
