use crate::*;

const XCC_GAS: Gas = Gas(10u64.pow(13));

#[near_bindgen]
impl Contract {
    pub fn is_human(&self, account_id: String) -> Promise {
        // TODO: add option for caller to specify providers
        let mut current_promise: Option<Promise> = None;
        let mut providers: Vec<ProviderExternal> = Vec::new();

        for provider_id in self.default_provider_ids.iter() {
            if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
                let provider = Provider::from(versioned_provider);
                let provider_json = ProviderExternal::from_provider_id(&provider_id.0, provider);

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
    pub fn is_human_callback(&self, providers: Vec<ProviderExternal>) -> bool {
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
