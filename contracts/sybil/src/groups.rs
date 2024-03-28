use crate::*;

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
    pub rule: Rule,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct GroupExternal {
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
        for (name, group) in self.groups_by_name.iter() {
            if let Some(provider_ids) = self.provider_ids_for_group.get(&name) {
                if &name != &group_name && provider_ids.to_vec() == providers {
                    env::panic_str("Group with the same providers already exists");
                }
            }
        }

        // If no providers are found in other groups, proceed to add or update the group
        let initial_storage_usage = env::storage_usage();
        let group = Group { rule: rule.clone() };
        self.groups_by_name.insert(&group_name, &group);
        let mut providers_set = UnorderedSet::new(StorageKey::ProviderIdsForGroupInner {
            group_name: group_name.clone(),
        });
        for provider_id in providers.iter() {
            providers_set.insert(provider_id);
        }
        self.provider_ids_for_group
            .insert(&group_name, &providers_set);

        refund_deposit(initial_storage_usage); // Refund any unused deposit

        let formatted_group = GroupExternal {
            name: group_name,
            providers,
            rule,
        };
        log_add_or_update_group_event(&formatted_group); // Log the event
        formatted_group
    }

    // Function to remove a group
    #[payable]
    pub fn delete_group(&mut self, group_name: String) {
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        self.groups_by_name.remove(&group_name);
        self.provider_ids_for_group.remove(&group_name);
        refund_deposit(initial_storage_usage);
        log_delete_group_event(&group_name);
    }

    // Get groups
    pub fn get_groups(&self) -> Vec<GroupExternal> {
        // Iterate through the groups map, transforming each (key, value) pair into a GroupExternal
        self.groups_by_name
            .iter()
            .map(|(name, group)| {
                GroupExternal {
                    name: name.clone(), // Clone the group name for the GroupExternal struct
                    providers: self.provider_ids_for_group.get(&name).unwrap().to_vec(), // Get the providers from the provider_ids_for_group map
                    rule: group.rule.clone(), // Clone the rule from the Group struct
                }
            })
            .collect()
    }

    // Function to get a group by name
    pub fn get_group(&self, group_name: String) -> Option<GroupExternal> {
        // Get the group by name from the groups map
        self.groups_by_name
            .get(&group_name)
            .map(|group| GroupExternal {
                name: group_name.clone(), // Clone the group name for the GroupExternal struct
                providers: self
                    .provider_ids_for_group
                    .get(&group_name)
                    .unwrap()
                    .to_vec(), // Get the providers from the provider_ids_for_group map
                rule: group.rule.clone(), // Clone the rule from the Group struct
            })
    }
}
