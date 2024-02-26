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
    pub providers: Vec<ProviderId>, // TODO: consider moving this to top-level storage
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
    ) {
        self.assert_owner_or_admin(); // Ensure that only authorized users can modify groups

        // Check if any of the provided provider IDs are already in an existing group
        for (existing_group_name, group) in self.groups.iter() {
            // Skip the current group if it's the one being updated
            if existing_group_name == group_name {
                continue;
            }

            for provider_id in &providers {
                assert!(
                    !group.providers.contains(provider_id),
                    "Provider {:?} is already in another group ({})",
                    provider_id,
                    existing_group_name
                );
            }
        }

        // If no providers are found in other groups, proceed to add or update the group
        let initial_storage_usage = env::storage_usage();
        let group = Group { providers, rule };
        self.groups.insert(&group_name, &group);

        refund_deposit(initial_storage_usage); // Refund any unused deposit
    }

    // Function to remove a group
    #[payable]
    pub fn remove_group(&mut self, group_name: String) {
        self.assert_owner_or_admin();
        let initial_storage_usage = env::storage_usage();
        self.groups.remove(&group_name);
        refund_deposit(initial_storage_usage);
    }

    // Get groups
    pub fn get_groups(&self) -> Vec<GroupExternal> {
        // Iterate through the groups map, transforming each (key, value) pair into a GroupExternal
        self.groups
            .iter()
            .map(|(name, group)| {
                GroupExternal {
                    name: name.clone(), // Clone the group name for the GroupExternal struct
                    providers: group.providers.clone(), // Clone the providers from the Group struct
                    rule: group.rule.clone(), // Clone the rule from the Group struct
                }
            })
            .collect()
    }

    // Function to get a group by name
    pub fn get_group(&self, group_name: String) -> Option<GroupExternal> {
        // Get the group by name from the groups map
        self.groups.get(&group_name).map(|group| GroupExternal {
            name: group_name, // Clone the group name for the GroupExternal struct
            providers: group.providers.clone(), // Clone the providers from the Group struct
            rule: group.rule.clone(), // Clone the rule from the Group struct
        })
    }
}
