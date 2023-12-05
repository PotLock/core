use crate::*;

const XCC_GAS: Gas = Gas(10u64.pow(13));

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProviderId(String);

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
    // TODO: consider adding optional `gas`
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
        // emit event
        log_add_provider_event(&provider_id, &provider);

        // TODO: REFUND UNUSED DEPOSIT

        // return provider ID
        provider_id
    }

    #[payable]
    pub fn set_default_providers(&mut self, provider_ids: Vec<ProviderId>) {
        // only contract owner or admin can call this method
        self.assert_owner_or_admin();
        // clear existing default providers
        self.default_provider_ids.clear();
        // add new default providers
        for provider_id in provider_ids {
            self.default_provider_ids.insert(&provider_id);
        }
        // TODO: REFUND UNUSED DEPOSIT
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

    pub fn is_human(&self, account_id: String) -> Promise {
        let mut current_promise: Option<Promise> = None;
        let mut providers: Vec<ProviderJson> = Vec::new(); // TODO: add option for caller to specify providers

        for provider_id in self.default_provider_ids.iter() {
            if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
                let provider = Provider::from(versioned_provider);
                let provider_json = ProviderJson::from_provider_id(&provider_id.0, provider);

                let args = json!({ "account_id": account_id }).to_string().into_bytes();

                let new_promise =
                    Promise::new(AccountId::new_unchecked(provider_json.contract_id.clone()))
                        .function_call(provider_json.method_name.clone(), args, 0, XCC_GAS);

                current_promise = Some(match current_promise {
                    Some(promise) => promise.and(new_promise),
                    None => new_promise,
                });

                providers.push(provider_json);
            }
        }

        match current_promise {
            Some(promise) => promise.then(
                Self::ext(env::current_account_id())
                    .with_static_gas(XCC_GAS)
                    .is_human_callback(providers),
            ),
            None => Promise::new(env::current_account_id()), // No providers available // TODO: come back here
        }
    }

    #[private]
    pub fn is_human_callback(&self, providers: Vec<ProviderJson>) -> bool {
        let mut total_score = 0;

        for index in 0..providers.len() {
            match env::promise_result(index as u64) {
                PromiseResult::Successful(value) => {
                    let is_human: bool = near_sdk::serde_json::from_slice(&value).unwrap_or(false);
                    log!("Promise result #{}: {}", index + 1, is_human);
                    log!("Weight: {}", providers[index].default_weight);
                    if is_human {
                        total_score += providers[index].default_weight;
                    }
                }
                _ => {} // Handle failed or not ready promises as needed // TODO: come back here
            }
        }
        log!("total_score: {}", total_score);
        log!("default_human_threshold: {}", self.default_human_threshold);

        total_score >= self.default_human_threshold
    }
}
