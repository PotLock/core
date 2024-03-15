use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HumanScoreResponse {
    pub is_human: bool,
    pub score: u32,
}

#[near_bindgen]
impl Contract {
    pub fn get_human_score(&self, account_id: AccountId) -> HumanScoreResponse {
        let total_score = self.get_score_for_account_id(account_id);
        HumanScoreResponse {
            is_human: total_score >= self.default_human_threshold,
            score: total_score,
        }
    }

    pub fn is_human(&self, account_id: AccountId) -> bool {
        // TODO: add option for caller to specify providers or custom default_human_threshold
        self.get_human_score(account_id).is_human
    }

    pub(crate) fn get_score_for_account_id(&self, account_id: AccountId) -> u32 {
        // get user stamps and add up default weights
        let mut total_score = 0;
        let user_providers = self.provider_ids_for_user.get(&account_id);
        if let Some(user_providers) = user_providers {
            for provider_id in user_providers.iter() {
                if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
                    let provider = Provider::from(versioned_provider);
                    total_score += provider.default_weight;
                }
            }
        }
        total_score
    }

    // DEPRECATED IMPLEMENTATION
    // pub fn is_human(&self, account_id: AccountId) -> Promise {
    //     // TODO: add option for caller to specify providers
    //     let mut current_promise: Option<Promise> = None;
    //     let mut providers: Vec<ProviderExternal> = Vec::new();

    //     for provider_id in self.default_provider_ids.iter() {
    //         if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
    //             let provider = Provider::from(versioned_provider);
    //             let provider_json = ProviderExternal::from_provider_id(&provider_id.0, provider);

    //             let args = json!({ "account_id": account_id }).to_string().into_bytes();

    //             let new_promise =
    //                 Promise::new(AccountId::new_unchecked(provider_json.contract_id.clone()))
    //                     .function_call(provider_json.method_name.clone(), args, 0, XCC_GAS);

    //             current_promise = Some(match current_promise {
    //                 Some(promise) => promise.and(new_promise),
    //                 None => new_promise,
    //             });

    //             providers.push(provider_json);
    //         }
    //     }

    //     match current_promise {
    //         Some(promise) => promise.then(
    //             Self::ext(env::current_account_id())
    //                 .with_static_gas(XCC_GAS)
    //                 .is_human_callback(providers),
    //         ),
    //         None => Promise::new(env::current_account_id()), // No providers available
    //     }
    // }

    // #[private]
    // pub fn is_human_callback(&self, providers: Vec<ProviderExternal>) -> bool {
    //     let mut total_score = 0;

    //     for index in 0..providers.len() {
    //         match env::promise_result(index as u64) {
    //             PromiseResult::Successful(value) => {
    //                 let is_human: bool = near_sdk::serde_json::from_slice(&value).unwrap_or(false);
    //                 log!("Promise result #{}: {}", index + 1, is_human);
    //                 log!("Weight: {}", providers[index].default_weight);
    //                 if is_human {
    //                     total_score += providers[index].default_weight;
    //                 }
    //             }
    //             _ => {} // Handle failed or not ready promises as needed
    //         }
    //     }
    //     log!("total_score: {}", total_score);
    //     log!("default_human_threshold: {}", self.default_human_threshold);

    //     total_score >= self.default_human_threshold
    // }
}
