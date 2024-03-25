use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ListInternal {
    // don't need ID since it's the key, but should include in ListExternal
    pub name: String,
    pub description: Option<String>,
    pub owner: AccountId,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
    pub default_registration_status: RegistrationStatus,
    // consider adding list status, e.g. draft, active, inactive, etc.
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedList {
    Current(ListInternal),
}

impl From<VersionedList> for ListInternal {
    fn from(list: VersionedList) -> Self {
        match list {
            VersionedList::Current(current) => current,
        }
    }
}

// Ephemeral data structure used for view methods, not stored within contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ListExternal {
    pub id: ListId,
    pub name: String,
    pub description: Option<String>,
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub created_at: TimestampMs,
    pub updated_at: TimestampMs,
    pub default_registration_status: RegistrationStatus,
    pub total_registrations_count: u64,
    pub total_upvotes_count: u64,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn create_list(
        &mut self,
        name: String,
        description: Option<String>,
        admins: Option<Vec<AccountId>>,
        default_registration_status: RegistrationStatus,
    ) -> ListExternal {
        let initial_storage_usage = env::storage_usage();
        let list_id = self.next_list_id;
        let list_internal = ListInternal {
            name,
            description,
            owner: env::predecessor_account_id(),
            created_at: env::block_timestamp_ms(),
            updated_at: env::block_timestamp_ms(),
            default_registration_status,
        };
        self.lists_by_id
            .insert(&list_id, &VersionedList::Current(list_internal.clone()));
        let mut list_ids_by_owner = self
            .list_ids_by_owner
            .get(&env::predecessor_account_id())
            .unwrap_or(UnorderedSet::new(StorageKey::ListIdsByOwnerInner {
                owner: list_internal.owner.clone(),
            }));
        list_ids_by_owner.insert(&list_id);
        self.list_ids_by_owner
            .insert(&env::predecessor_account_id(), &list_ids_by_owner);
        let mut admins_set =
            UnorderedSet::new(StorageKey::ListAdminsByListIdInner { list_id: list_id });
        if let Some(admins) = admins.clone() {
            for admin in admins {
                admins_set.insert(&admin);
            }
        }
        self.list_admins_by_list_id.insert(&list_id, &admins_set);
        self.registration_ids_by_list_id.insert(
            &list_id,
            &UnorderedSet::new(StorageKey::RegistrationIdsByListIdInner {
                list_id: self.next_list_id,
            }),
        );
        self.upvotes_by_list_id.insert(
            &list_id,
            &UnorderedSet::new(StorageKey::UpvotesByListIdInner {
                list_id: self.next_list_id,
            }),
        );
        self.next_list_id += 1;
        let formatted_list = self.format_list(list_id, list_internal);
        log_create_list_event(&formatted_list);
        refund_deposit(initial_storage_usage);
        formatted_list
    }

    #[payable]
    pub fn update_list(
        &mut self,
        list_id: ListId,
        name: Option<String>,
        description: Option<String>,
        default_registration_status: Option<RegistrationStatus>,
    ) -> ListExternal {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        let mut list =
            ListInternal::from(self.lists_by_id.get(&list_id).expect("List does not exist"));
        if let Some(name) = name {
            list.name = name;
        }
        if let Some(description) = description {
            list.description = Some(description);
        }
        if let Some(default_registration_status) = default_registration_status {
            list.default_registration_status = default_registration_status;
        }
        list.updated_at = env::block_timestamp_ms();
        self.lists_by_id
            .insert(&list_id, &VersionedList::Current(list.clone()));
        log_update_list_event(&list);
        refund_deposit(initial_storage_usage);
        self.format_list(list_id, list)
    }

    #[payable]
    pub fn delete_list(&mut self, list_id: ListId) {
        self.assert_list_owner(&list_id);
        let initial_storage_usage = env::storage_usage();
        self.lists_by_id.remove(&list_id);
        self.list_ids_by_owner
            .get(&env::predecessor_account_id())
            .expect("List IDs by owner do not exist")
            .remove(&list_id);
        self.list_admins_by_list_id.remove(&list_id);
        let list_registrations = self.registration_ids_by_list_id.remove(&list_id);
        // remove all registrations for this list
        if let Some(list_registrations) = list_registrations {
            for registration_id in list_registrations.iter() {
                self.registrations_by_id.remove(&registration_id);
                let registration = RegistrationInternal::from(
                    self.registrations_by_id
                        .get(&registration_id)
                        .expect("No registration found"),
                );
                self.registration_ids_by_registrant_id
                    .get(&registration.registrant_id)
                    .expect("Registrant IDs by registrant do not exist")
                    .remove(&registration_id);
            }
        }
        self.registration_ids_by_list_id.remove(&list_id);
        self.upvotes_by_list_id.remove(&list_id);
        log_delete_list_event(list_id);
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn upvote(&mut self, list_id: ListId) {
        let initial_storage_usage = env::storage_usage();
        let mut upvotes = self
            .upvotes_by_list_id
            .get(&list_id)
            .expect("Upvotes by list ID do not exist");
        let inserted = upvotes.insert(&env::predecessor_account_id());
        self.upvotes_by_list_id.insert(&list_id, &upvotes);
        if inserted {
            log_upvote_event(list_id, env::predecessor_account_id());
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn remove_upvote(&mut self, list_id: ListId) {
        let initial_storage_usage = env::storage_usage();
        let mut upvotes = self
            .upvotes_by_list_id
            .get(&list_id)
            .expect("Upvotes by list ID do not exist");
        let removed = upvotes.remove(&env::predecessor_account_id());
        self.upvotes_by_list_id.insert(&list_id, &upvotes);
        if removed {
            log_remove_upvote_event(list_id, env::predecessor_account_id());
        }
        refund_deposit(initial_storage_usage);
    }

    pub fn get_lists(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<ListExternal> {
        self.lists_by_id
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(u64::MAX) as usize)
            .map(|(list_id, list)| self.format_list(list_id, ListInternal::from(list)))
            .collect()
    }

    pub fn get_lists_for_owner(&self, owner_id: AccountId) -> Vec<ListExternal> {
        self.list_ids_by_owner
            .get(&owner_id)
            .expect("List IDs by owner do not exist")
            .iter()
            .map(|list_id| {
                self.format_list(
                    list_id,
                    ListInternal::from(
                        self.lists_by_id.get(&list_id).expect("List does not exist"),
                    ),
                )
            })
            .collect()
    }

    pub fn get_lists_for_registrant(&self, registrant_id: AccountId) -> Vec<ListExternal> {
        self.list_ids_by_registrant
            .get(&registrant_id)
            .expect("List IDs by registrant do not exist")
            .iter()
            .map(|list_id| {
                self.format_list(
                    list_id,
                    ListInternal::from(
                        self.lists_by_id.get(&list_id).expect("List does not exist"),
                    ),
                )
            })
            .collect()
    }

    pub fn get_upvotes_for_list(
        &self,
        list_id: ListId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<AccountId> {
        self.upvotes_by_list_id
            .get(&list_id)
            .expect("Upvotes by list ID do not exist")
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(u64::MAX) as usize)
            .collect()
    }

    pub(crate) fn format_list(&self, list_id: ListId, list_internal: ListInternal) -> ListExternal {
        ListExternal {
            id: list_id,
            name: list_internal.name,
            description: list_internal.description,
            owner: list_internal.owner,
            admins: self
                .list_admins_by_list_id
                .get(&list_id)
                .expect("List admins by list ID do not exist")
                .to_vec(),
            created_at: list_internal.created_at,
            updated_at: list_internal.updated_at,
            default_registration_status: list_internal.default_registration_status,
            total_registrations_count: self
                .registration_ids_by_list_id
                .get(&list_id)
                .expect("Registration IDs by list ID do not exist")
                .len(),
            total_upvotes_count: self
                .upvotes_by_list_id
                .get(&list_id)
                .expect("Upvotes by list ID do not exist")
                .len(),
        }
    }
}
