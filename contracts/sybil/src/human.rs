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
        if self.blacklisted_accounts.contains(&account_id) {
            return total_score;
        }

        let mut group_scores: HashMap<u64, Vec<u32>> = HashMap::new(); // Scores indexed by group ID
        let mut provider_to_smallest_group: HashMap<ProviderId, u64> = HashMap::new(); // Maps provider to its smallest group by size

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
                            let score = provider.default_weight;

                            // Determine the smallest group for this provider
                            if let Some(group_ids) = self.group_ids_for_provider.get(&provider_id) {
                                let mut smallest_group_size = u64::MAX;
                                let mut smallest_group_id = 0;

                                for group_id in group_ids.iter() {
                                    // if let Some(group) = self.groups_by_id.get(&group_id) {
                                    let providers =
                                        self.provider_ids_for_group.get(&group_id).unwrap();
                                    let group_size = providers.len() as u64;
                                    if group_size < smallest_group_size {
                                        smallest_group_size = group_size;
                                        smallest_group_id = group_id;
                                    }
                                    // }
                                }

                                // Update the provider to its smallest group mapping
                                provider_to_smallest_group
                                    .insert(provider_id.clone(), smallest_group_id);

                                // Add the provider's score to its smallest group's scores
                                group_scores
                                    .entry(smallest_group_id)
                                    .or_insert_with(Vec::new)
                                    .push(score);
                            } else {
                                // Providers not in any group are added directly to the total score
                                total_score += score;
                            }
                        }
                    }
                }
            }
        }

        // Now, apply group rules to calculate the scores for each group
        for (&group_id, scores) in &group_scores {
            if let Some(group) = self.groups_by_id.get(&group_id) {
                let group_rule = &group.rule; // Assuming a `rule` field exists in the Group struct

                let group_score = match group_rule {
                    Rule::Highest => *scores.iter().max().unwrap_or(&0),
                    Rule::Lowest => *scores.iter().min().unwrap_or(&0),
                    Rule::Sum(max_value_option) => {
                        let sum: u32 = scores.iter().sum();
                        max_value_option.map_or(sum, |max_value| std::cmp::min(sum, max_value))
                    }
                    Rule::DiminishingReturns(factor) => {
                        calculate_diminishing_returns(scores, *factor)
                    }
                    Rule::IncreasingReturns(factor) => {
                        calculate_increasing_returns(scores, *factor)
                    }
                };

                total_score += group_score;
            }
        }

        total_score
    }
}
