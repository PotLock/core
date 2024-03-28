use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct HumanScoreResponse {
    pub is_human: bool,
    pub score: u32,
}

fn calculate_diminishing_returns(scores: &Vec<u32>, factor: u32) -> u32 {
    let mut total = 0;
    for (i, score) in scores.iter().enumerate() {
        // Calculate the diminishing factor for this score
        let diminishing_factor = 100 - (i as u32 * factor);
        let adjusted_score = *score * diminishing_factor / 100;
        total += adjusted_score;
    }
    total
}

fn calculate_increasing_returns(scores: &Vec<u32>, factor: u32) -> u32 {
    let mut total = 0;
    for (i, score) in scores.iter().enumerate() {
        // Calculate the increasing factor for this score
        let increasing_factor = 100 + (i as u32 * factor);
        let adjusted_score = *score * increasing_factor / 100;
        total += adjusted_score;
    }
    total
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
        let mut provider_scores: HashMap<ProviderId, (u32, usize)> = HashMap::new();
        let mut group_scores: HashMap<String, Vec<u32>> = HashMap::new();

        if let Some(user_stamp_ids) = self.stamp_ids_for_user.get(&account_id) {
            for stamp_id in user_stamp_ids.iter() {
                if let Some(versioned_stamp) = self.stamps_by_id.get(&stamp_id) {
                    let stamp = Stamp::from(versioned_stamp);
                    let provider_id = stamp.provider_id.clone();

                    if let Some(versioned_provider) = self.providers_by_id.get(&provider_id) {
                        let provider = Provider::from(versioned_provider);
                        let current_time_ms = env::block_timestamp_ms();
                        let stamp_age_ms = current_time_ms - stamp.validated_at_ms;

                        if provider
                            .stamp_validity_ms
                            .map_or(true, |validity_period| stamp_age_ms <= validity_period)
                        {
                            // Aggregate scores, considering provider is part of any group
                            for (group_name, group) in self.groups_by_name.iter() {
                                let providers_for_group =
                                    self.provider_ids_for_group.get(&group_name);
                                if let Some(providers) = providers_for_group {
                                    if providers.contains(&provider_id) {
                                        // Ensure group_size is u64 here as intended
                                        let group_size: usize = providers.len() as usize;
                                        let score = provider.default_weight;

                                        provider_scores
                                            .entry(provider_id.clone())
                                            .and_modify(|e| {
                                                // Now e.1 is expected to be u64, so no type mismatch
                                                if group_size < e.1 {
                                                    *e = (score, group_size);
                                                    group_scores
                                                        .entry(group_name.clone())
                                                        .or_default()
                                                        .push(score);
                                                }
                                            })
                                            .or_insert_with(|| {
                                                group_scores
                                                    .entry(group_name.clone())
                                                    .or_default()
                                                    .push(score);
                                                (score, group_size)
                                            });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calculate group scores based on the rules
        for (group_id, scores) in group_scores {
            let group_rule = &self.groups_by_name.get(&group_id).unwrap().rule;

            let group_score = match group_rule {
                Rule::Highest => *scores.iter().max().unwrap_or(&0),
                Rule::Lowest => *scores.iter().min().unwrap_or(&0),
                Rule::Sum(max_value_option) => {
                    let sum: u32 = scores.iter().sum();
                    if let Some(max_value) = max_value_option {
                        std::cmp::min(sum, *max_value)
                    } else {
                        sum
                    }
                }
                Rule::DiminishingReturns(factor) => calculate_diminishing_returns(&scores, *factor),
                Rule::IncreasingReturns(factor) => calculate_increasing_returns(&scores, *factor),
            };

            total_score += group_score;
        }

        // No need to handle ungrouped providers explicitly if stamps cover all providers

        total_score
    }
}
