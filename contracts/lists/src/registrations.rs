use crate::*;

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone, Copy,
)]
#[serde(crate = "near_sdk::serde")]
pub enum RegistrationStatus {
    Pending,
    Approved,
    Rejected,
    Graylisted,
    Blacklisted,
}

// CURRENT - RegistrationInternal is the data structure that is stored within the contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RegistrationInternal {
    // don't need to store ID since it's the key
    pub registrant_id: AccountId,
    pub list_id: ListId,
    pub status: RegistrationStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub admin_notes: Option<String>,
    pub registrant_notes: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedRegistrationInternal {
    Current(RegistrationInternal),
}

impl From<VersionedRegistrationInternal> for RegistrationInternal {
    fn from(registration_internal: VersionedRegistrationInternal) -> Self {
        match registration_internal {
            VersionedRegistrationInternal::Current(current) => current,
        }
    }
}

// Ephemeral data structure used for view methods, not stored within contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RegistrationExternal {
    pub id: RegistrationId,
    pub registrant_id: AccountId,
    pub list_id: ListId,
    pub status: RegistrationStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub admin_notes: Option<String>,
    pub registrant_notes: Option<String>,
}

pub(crate) fn format_registration(
    registration_id: RegistrationId,
    registration_internal: RegistrationInternal,
) -> RegistrationExternal {
    RegistrationExternal {
        id: registration_id,
        registrant_id: registration_internal.registrant_id,
        list_id: registration_internal.list_id,
        status: registration_internal.status,
        submitted_ms: registration_internal.submitted_ms,
        updated_ms: registration_internal.updated_ms,
        admin_notes: registration_internal.admin_notes,
        registrant_notes: registration_internal.registrant_notes,
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register(
        &mut self,
        list_id: ListId,
        _registrant_id: Option<AccountId>,
        notes: Option<String>,
    ) -> RegistrationExternal {
        let initial_storage_usage = env::storage_usage();

        let list = ListInternal::from(self.lists_by_id.get(&list_id).expect("List does not exist"));
        let caller_is_admin_or_greater = self.is_caller_list_admin_or_greater(&list_id);

        // _registrant_id can only be specified by admin or greater; otherwise, it is the caller
        let mut registrant_id = env::predecessor_account_id();
        if _registrant_id.is_some() && caller_is_admin_or_greater {
            registrant_id = _registrant_id.unwrap();
        }

        // make sure registration doesn't already exist for this registrant on this list
        let mut list_ids_for_registrant = self
            .list_ids_by_registrant
            .get(&registrant_id)
            .unwrap_or(UnorderedSet::new(StorageKey::ListIdsByRegistrantInner {
                registrant: registrant_id.clone(),
            }));
        assert!(
            !list_ids_for_registrant.contains(&list_id),
            "Registration already exists for this registrant on this list"
        );

        // create registration
        let registration_internal = RegistrationInternal {
            registrant_id: registrant_id.clone(),
            list_id,
            status: if self.is_caller_list_admin_or_greater(&list_id) {
                RegistrationStatus::Approved
            } else {
                list.default_registration_status
            },
            submitted_ms: env::block_timestamp_ms(),
            updated_ms: env::block_timestamp_ms(),
            admin_notes: if caller_is_admin_or_greater {
                notes.clone()
            } else {
                None
            },
            registrant_notes: if !caller_is_admin_or_greater {
                notes.clone()
            } else {
                None
            },
        };

        // update mappings
        list_ids_for_registrant.insert(&list_id);
        self.list_ids_by_registrant
            .insert(&registrant_id, &list_ids_for_registrant);
        let registration_id = self.next_registration_id;
        self.next_registration_id += 1;
        self.registrations_by_id.insert(
            &registration_id,
            &VersionedRegistrationInternal::Current(registration_internal.clone()),
        );
        let mut registration_ids_for_list = self
            .registration_ids_by_list_id
            .get(&list_id)
            .expect("Registration IDs by list ID do not exist");
        registration_ids_for_list.insert(&registration_id);
        self.registration_ids_by_list_id
            .insert(&list_id, &registration_ids_for_list);
        let mut registration_ids_for_registrant = self
            .registration_ids_by_registrant_id
            .get(&registrant_id)
            .unwrap_or(UnorderedSet::new(
                StorageKey::RegistrationIdsByRegistrantIdInner {
                    registrant_id: registrant_id.clone(),
                },
            ));
        registration_ids_for_registrant.insert(&registration_id);
        self.registration_ids_by_registrant_id
            .insert(&registrant_id, &registration_ids_for_registrant);

        let formatted_registration = format_registration(registration_id, registration_internal);

        log_create_registration_event(&formatted_registration);

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        // return formatted registration
        formatted_registration
    }

    #[payable]
    pub fn unregister(&mut self, list_id: Option<ListId>, registration_id: Option<RegistrationId>) {
        let initial_storage_usage = env::storage_usage();
        let registrant_id = env::predecessor_account_id();

        // unregister by list ID
        if let Some(list_id) = list_id {
            let registration_ids = self
                .registration_ids_by_list_id
                .get(&list_id)
                .expect("Registration IDs by list ID do not exist");
            let registration_ids = registration_ids.to_vec();
            let registration_id = registration_ids.into_iter().find(|registration_id| {
                let registration_internal = RegistrationInternal::from(
                    self.registrations_by_id
                        .get(&registration_id)
                        .expect("No registration found"),
                );
                registration_internal.registrant_id == registrant_id
            });
            if let Some(registration_id) = registration_id {
                self.unregister_by_registration_id(registration_id);
                log_delete_registration_event(registration_id);
            }
        }
        // unregister by registration ID
        else if let Some(registration_id) = registration_id {
            self.unregister_by_registration_id(registration_id);
            log_delete_registration_event(registration_id);
        }

        // refund any unused deposit
        refund_deposit(initial_storage_usage);
    }

    pub(crate) fn unregister_by_registration_id(&mut self, registration_id: RegistrationId) {
        let registration_internal = RegistrationInternal::from(
            self.registrations_by_id
                .get(&registration_id)
                .expect("No registration found"),
        );
        let registrant_id = registration_internal.registrant_id;
        let list_id = registration_internal.list_id;
        let caller_is_admin_or_greater = self.is_caller_list_admin_or_greater(&list_id);
        // only the registrant or an admin or owner of the list can unregister
        assert!(
            registrant_id == env::predecessor_account_id() || caller_is_admin_or_greater,
            "Caller is not the registrant or an admin or owner of the list"
        );

        if !caller_is_admin_or_greater {
            // status must be pending in order for registrant to unregister
            assert!(
                registration_internal.status == RegistrationStatus::Pending,
                "Registrant can only unregister if status is pending"
            );
        }

        // update mappings
        let mut registration_ids_for_list = self
            .registration_ids_by_list_id
            .get(&list_id)
            .expect("Registration IDs by list ID do not exist");
        registration_ids_for_list.remove(&registration_id);
        self.registration_ids_by_list_id
            .insert(&list_id, &registration_ids_for_list);
        let mut registration_ids_for_registrant = self
            .registration_ids_by_registrant_id
            .get(&registrant_id)
            .expect("Registration IDs by registrant ID do not exist");
        registration_ids_for_registrant.remove(&registration_id);
        self.registration_ids_by_registrant_id
            .insert(&registrant_id, &registration_ids_for_registrant);
        self.registrations_by_id.remove(&registration_id);
    }

    #[payable]
    pub fn update_registration(
        &mut self,
        registration_id: RegistrationId,
        status: Option<RegistrationStatus>,
        notes: Option<String>,
    ) -> RegistrationExternal {
        let initial_storage_usage = env::storage_usage();
        let mut registration_internal = RegistrationInternal::from(
            self.registrations_by_id
                .get(&registration_id)
                .expect("No registration found"),
        );
        let caller_is_admin_or_greater =
            self.is_caller_list_admin_or_greater(&registration_internal.list_id);

        // update registration
        registration_internal.status = if caller_is_admin_or_greater && status.is_some() {
            status.unwrap()
        } else {
            registration_internal.status
        };
        registration_internal.updated_ms = env::block_timestamp_ms();
        registration_internal.admin_notes = if caller_is_admin_or_greater {
            if let Some(notes) = notes.clone() {
                Some(notes.clone())
            } else {
                registration_internal.admin_notes
            }
        } else {
            registration_internal.admin_notes
        };
        registration_internal.registrant_notes = if !caller_is_admin_or_greater {
            if let Some(notes) = notes.clone() {
                Some(notes.clone())
            } else {
                registration_internal.registrant_notes
            }
        } else {
            registration_internal.registrant_notes
        };

        // update mappings
        self.registrations_by_id.insert(
            &registration_id,
            &VersionedRegistrationInternal::Current(registration_internal.clone()),
        );

        // format registration
        let registration_external = format_registration(registration_id, registration_internal);

        // log event
        log_update_registration_event(&registration_external);

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        registration_external
    }

    pub fn get_registrations_for_list(
        &self,
        list_id: ListId,
        status: Option<RegistrationStatus>,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<RegistrationExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let registration_ids = self
            .registration_ids_by_list_id
            .get(&list_id)
            .expect("Registration IDs by list ID do not exist");
        let registration_ids = registration_ids.to_vec();
        let registration_ids = if let Some(status) = status {
            registration_ids
                .into_iter()
                .filter(|registration_id| {
                    let registration_internal = RegistrationInternal::from(
                        self.registrations_by_id
                            .get(&registration_id)
                            .expect("No registration found"),
                    );
                    registration_internal.status == status
                })
                .collect()
        } else {
            registration_ids
        };
        assert!(
            (registration_ids.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        registration_ids
            .into_iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|registration_id| {
                let registration_internal = RegistrationInternal::from(
                    self.registrations_by_id
                        .get(&registration_id)
                        .expect("No registration found"),
                );
                format_registration(registration_id, registration_internal)
            })
            .collect()
    }

    pub fn get_registrations_for_registrant(
        &self,
        registrant_id: AccountId,
        status: Option<RegistrationStatus>,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<RegistrationExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let registration_ids = self
            .registration_ids_by_registrant_id
            .get(&registrant_id)
            .expect("Registration IDs by registrant ID do not exist");
        let registration_ids = registration_ids.to_vec();
        let registration_ids = if let Some(status) = status {
            registration_ids
                .into_iter()
                .filter(|registration_id| {
                    let registration_internal = RegistrationInternal::from(
                        self.registrations_by_id
                            .get(&registration_id)
                            .expect("No registration found"),
                    );
                    registration_internal.status == status
                })
                .collect()
        } else {
            registration_ids
        };
        assert!(
            (registration_ids.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        registration_ids
            .into_iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|registration_id| {
                let registration_internal = RegistrationInternal::from(
                    self.registrations_by_id
                        .get(&registration_id)
                        .expect("No registration found"),
                );
                format_registration(registration_id, registration_internal)
            })
            .collect()
    }

    pub fn is_registered(
        &self,
        list_id: ListId,
        account_id: RegistrantId,
        required_status: Option<RegistrationStatus>,
    ) -> bool {
        let registration_ids = self
            .registration_ids_by_list_id
            .get(&list_id)
            .expect("Registration IDs by list ID do not exist");
        let registration_ids = registration_ids.to_vec();
        registration_ids.into_iter().any(|registration_id| {
            let registration_internal = RegistrationInternal::from(
                self.registrations_by_id
                    .get(&registration_id)
                    .expect("No registration found"),
            );
            registration_internal.registrant_id == account_id
                && (required_status.is_none()
                    || registration_internal.status == required_status.unwrap())
        })
    }
}
