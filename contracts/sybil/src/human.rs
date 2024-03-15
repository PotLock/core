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
        let mut total_score = 0;
        let mut scored_providers: Vec<ProviderId> = Vec::new(); // Track which providers have been scored

        // handle grouped providers
        for group in self.groups.values() {
            let mut group_scores: Vec<u32> = Vec::new();

            // Get scores for providers in this group, ignoring expired stamps
            for provider_id in group.providers.iter() {
                if let Some(user_providers) = self.provider_ids_for_user.get(&account_id) {
                    if user_providers.contains(provider_id) {
                        // user has stamp from this provider
                        if let Some(versioned_provider) = self.providers_by_id.get(provider_id) {
                            let provider = Provider::from(versioned_provider);
                            // Check stamp validity
                            // let stamp_id = StampId::new(account_id.clone(), provider_id.clone());
                            // if let Some(stamp) = self.stamps_by_id.get(&stamp_id) {
                            //     if stamp.is_expired() {
                            //         continue;
                            //     }
                            // }
                            if provider.stamp_validity_ms.map_or(true, |validity_period| { // TODO: also check that the stamp has not been deactivated
                                // let stamp_creation_time = provider.creation_time_ms.unwrap_or(0);
                                let stamp_id =
                                    StampId::new(account_id.clone(), provider_id.clone());
                                let stamp = Stamp::from(
                                    self.stamps_by_id.get(&stamp_id).expect("stamp not found"),
                                );
                                env::block_timestamp_ms() <= stamp.validated_at_ms + validity_period
                            }) {
                                group_scores.push(provider.default_weight);
                                scored_providers.push(provider_id.clone()); // Mark this provider as scored
                            }
                        }
                    }
                }
            }

            // Apply the group rule to aggregate the scores
            let group_score = match group.rule {
                Rule::Highest => group_scores.into_iter().max().unwrap_or(0),
                Rule::Lowest => group_scores.into_iter().min().unwrap_or(0),
                Rule::Sum(max_value_option) => {
                    let sum: u32 = group_scores.iter().sum(); // Calculate the sum of all scores in the group
                    match max_value_option {
                        Some(max_value) => std::cmp::min(sum, max_value), // If a max value is specified, cap the sum at this value
                        None => sum, // If no max value is specified, use the calculated sum as is
                    }
                }
                Rule::DiminishingReturns(factor) => {
                    let mut sum = 0;
                    let highest_score = *group_scores.iter().max().unwrap_or(&0); // Find the highest score in the group

                    for (i, score) in group_scores.iter().enumerate() {
                        // Apply diminishing returns factor
                        let adjusted_score = score * (100 - (i as u32 * factor)) / 100;
                        sum += adjusted_score;
                    }

                    // Ensure the sum doesn't fall below the highest single stamp's score
                    std::cmp::max(sum, highest_score)
                }
                Rule::IncreasingReturns(factor) => {
                    let mut sum = 0;
                    for (i, score) in group_scores.iter().enumerate() {
                        // Apply increasing returns factor
                        let adjusted_score = score * (100 + (i as u32 * factor)) / 100;
                        sum += adjusted_score;
                    }
                    sum
                }
            };

            total_score += group_score;
        }

        // Handle ungrouped providers
        if let Some(user_providers) = self.provider_ids_for_user.get(&account_id) {
            for provider_id in user_providers.iter() {
                // Skip providers that have already been scored as part of a group
                if scored_providers.contains(&provider_id) {
                    continue;
                }

                if let Some(provider) = self.providers_by_id.get(&provider_id) {
                    total_score += Provider::from(provider).default_weight;
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
