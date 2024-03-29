use crate::*;

pub type GroupId = u64;

// Enum to specify the rule type
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Rule {
    Highest,                 // Take the highest score from the group
    Lowest,                  // Take the lowest score from the group
    Sum(Option<u32>),        // Sum all scores with optional max value
    DiminishingReturns(u32), // Sum with diminishing returns, factor in percentage (e.g., 10 for 10% reduction each)
    IncreasingReturns(u32), // Sum with increasing returns, factor in percentage (e.g., 10 for 10% increase each)
}

// Represents a group containing providers and a rule
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Group {
    pub name: String,
    pub rule: Rule,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupExternal {
    pub id: GroupId,
    pub name: String,
    pub providers: Vec<ProviderId>,
    pub rule: Rule,
}

#[near_bindgen]
impl Contract {
    // Function to add a new group or modify an existing one
    #[payable]
    pub fn add_or_update_group(
        &mut self,
        group_name: String,
        providers: Vec<ProviderId>,
        rule: Rule,
    ) -> GroupExternal {
        self.assert_owner_or_admin(); // Ensure that only authorized users can modify groups

        // Check if any other group has this exact set of providers
        for (group_id, group) in self.groups_by_id.iter() {
            if let Some(provider_ids) = self.provider_ids_for_group.get(&group_id) {
                if provider_ids.to_vec() == providers {
                    env::panic_str("Group with the same providers already exists");
                }
            }
        }

        // If no providers are found in other groups, proceed to add or update the group
        let initial_storage_usage = env::storage_usage();
        let group = Group {
            name: group_name.clone(),
            rule: rule.clone(),
        };
        let group_id = self.next_group_id;
        self.next_group_id += 1;

        // update groups_by_id
        self.groups_by_id.insert(&group_id, &group);
        // update provider_ids_for_group
        let mut providers_set = UnorderedSet::new(StorageKey::ProviderIdsForGroupInner {
            group_name: group_name.clone(),
        });
        for provider_id in providers.iter() {
            providers_set.insert(provider_id);
        }
        self.provider_ids_for_group
            .insert(&group_id, &providers_set);
        // update group_ids_for_provider
        for provider_id in providers.iter() {
            let mut groups_for_provider_set = self
                .group_ids_for_provider
                .get(&provider_id)
                .unwrap_or(UnorderedSet::new(StorageKey::GroupIdsForProviderInner {
                    provider_id: provider_id.clone(),
                }));
            groups_for_provider_set.insert(&group_id);
            self.group_ids_for_provider
                .insert(&provider_id, &groups_for_provider_set);
        }

        refund_deposit(initial_storage_usage); // Refund any unused deposit

        let formatted_group = GroupExternal {
            id: group_id,
            name: group_name,
            providers,
            rule,
        };
        log_add_or_update_group_event(&formatted_group); // Log the event
        formatted_group
    }

    // Function to remove a group
    #[payable]
    pub fn delete_group(&mut self, group_id: GroupId) {
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        let removed = self.groups_by_id.remove(&group_id);
        if removed.is_some() {
            // For each provider in the group, remove the group from the provider's group list
            let provider_ids_for_group =
                self.provider_ids_for_group.get(&group_id).unwrap().to_vec();
            for provider_id in provider_ids_for_group.iter() {
                let mut groups_for_provider_set =
                    self.group_ids_for_provider.get(&provider_id).unwrap();
                groups_for_provider_set.remove(&group_id);
                self.group_ids_for_provider
                    .insert(&provider_id, &groups_for_provider_set);
            }
            self.provider_ids_for_group.get(&group_id).unwrap().clear();
            self.provider_ids_for_group.remove(&group_id);
            refund_deposit(initial_storage_usage);
            log_delete_group_event(&group_id);
        }
    }

    // Get groups
    pub fn get_groups(&self) -> Vec<GroupExternal> {
        // Iterate through the groups map, transforming each (key, value) pair into a GroupExternal
        self.groups_by_id
            .iter()
            .map(|(group_id, group)| GroupExternal {
                id: group_id.clone(),
                name: group.name.clone(),
                providers: self.provider_ids_for_group.get(&group_id).unwrap().to_vec(),
                rule: group.rule.clone(),
            })
            .collect()
    }

    // Function to get a group by name
    pub fn get_group(&self, group_id: GroupId) -> Option<GroupExternal> {
        // Get the group by name from the groups map
        self.groups_by_id.get(&group_id).map(|group| GroupExternal {
            id: group_id.clone(),
            name: group.name.clone(),
            providers: self.provider_ids_for_group.get(&group_id).unwrap().to_vec(),
            rule: group.rule.clone(),
        })
    }
}
