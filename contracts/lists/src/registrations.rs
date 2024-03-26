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
    pub registered_by: Option<AccountId>, // Could be list owner or a list admin. If None, use registrant_id. Used for processing refund on deletion of registration.
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
    pub registered_by: AccountId,
}

pub(crate) fn format_registration(
    registration_id: RegistrationId,
    registration_internal: RegistrationInternal,
) -> RegistrationExternal {
    RegistrationExternal {
        id: registration_id,
        registrant_id: registration_internal.registrant_id.clone(),
        list_id: registration_internal.list_id,
        status: registration_internal.status,
        submitted_ms: registration_internal.submitted_ms,
        updated_ms: registration_internal.updated_ms,
        admin_notes: registration_internal.admin_notes,
        registrant_notes: registration_internal.registrant_notes,
        registered_by: registration_internal
            .registered_by
            .unwrap_or_else(|| registration_internal.registrant_id.clone()),
    }
}

// Ephemeral struct for admins to use when registering a registrant
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RegistrationInput {
    pub registrant_id: AccountId,
    pub status: RegistrationStatus,
    pub submitted_ms: Option<TimestampMs>,
    pub updated_ms: Option<TimestampMs>,
    pub notes: Option<String>,
}

#[near_bindgen]
impl Contract {
    pub(crate) fn register(
        &mut self,
        list_id: ListId,
        list: ListInternal,
        notes: Option<String>,
        _registrant_id: Option<AccountId>,
        _submitted_ms: Option<TimestampMs>, // added temporarily for the purposes of migrating existing Registry contract
        _updated_ms: Option<TimestampMs>, // added temporarily for the purposes of migrating existing Registry contract
        _status: Option<RegistrationStatus>, // added temporarily for the purposes of migrating existing Registry contract
    ) -> RegistrationExternal {
        let caller_is_admin_or_greater = self.is_caller_list_admin_or_greater(&list_id);

        if list.admin_only_registrations && !caller_is_admin_or_greater {
            panic!("Only admins can create registrations for this list");
        }

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
            "Registration already exists for {} on this list",
            registrant_id
        );

        let status = if caller_is_admin_or_greater {
            if let Some(_status) = _status {
                _status
            } else {
                RegistrationStatus::Approved
            }
        } else {
            list.default_registration_status
        };

        let block_timestamp_ms = env::block_timestamp_ms();

        let submitted_ms = if caller_is_admin_or_greater {
            if let Some(_submitted_ms) = _submitted_ms {
                _submitted_ms
            } else {
                block_timestamp_ms
            }
        } else {
            block_timestamp_ms
        };

        let updated_ms = if caller_is_admin_or_greater {
            if let Some(_updated_ms) = _updated_ms {
                _updated_ms
            } else {
                block_timestamp_ms
            }
        } else {
            block_timestamp_ms
        };

        // create registration
        let registration_internal = RegistrationInternal {
            registrant_id: registrant_id.clone(),
            list_id,
            status,
            submitted_ms,
            updated_ms,
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
            registered_by: if caller_is_admin_or_greater {
                Some(env::predecessor_account_id())
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

        // return formatted registration
        formatted_registration
    }

    #[payable]
    pub fn register_batch(
        &mut self,
        list_id: ListId,
        notes: Option<String>, // provided by non-admin registrants
        registrations: Option<Vec<RegistrationInput>>, // provided by admin registrants
    ) -> Vec<RegistrationExternal> {
        let list = ListInternal::from(self.lists_by_id.get(&list_id).expect("List does not exist"));
        let caller_is_admin_or_greater = self.is_caller_list_admin_or_greater(&list_id);
        let mut registrations = if caller_is_admin_or_greater {
            registrations.expect("registrations arg is required for admin calls")
        } else {
            Vec::new()
        };
        if !caller_is_admin_or_greater {
            registrations.push(RegistrationInput {
                registrant_id: env::predecessor_account_id(),
                status: list.default_registration_status,
                submitted_ms: None,
                updated_ms: None,
                notes,
            });
        }
        // make sure batch size is within limit
        assert!(
            registrations.len() <= MAX_REGISTRATION_BATCH_SIZE as usize,
            "Batch size exceeds limit"
        );
        let initial_storage_usage = env::storage_usage();
        let mut completed_registrations = Vec::new();
        // iterate through registrations and call self.register each one
        for registration in registrations {
            completed_registrations.push(self.register(
                list_id,
                list.clone(),
                registration.notes,
                Some(registration.registrant_id),
                registration.submitted_ms,
                registration.updated_ms,
                Some(registration.status),
            ));
        }

        // refund any unused deposit
        refund_deposit(initial_storage_usage, None);

        // log events
        for formatted_registration in completed_registrations.iter() {
            log_create_registration_event(formatted_registration);
        }

        // return formatted registrations
        completed_registrations
    }

    #[payable]
    pub fn unregister(&mut self, list_id: Option<ListId>, registration_id: Option<RegistrationId>) {
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
        // track storage freed to refund to registrant (if caller is owner or admin)
        let initial_storage_usage = env::storage_usage();
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
        // refund to original account that registered the registration
        refund_deposit(
            initial_storage_usage,
            Some(
                registration_internal
                    .registered_by
                    .unwrap_or_else(|| registrant_id),
            ),
        );
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

        // refund any unused deposit
        refund_deposit(initial_storage_usage, None);

        // log event
        log_update_registration_event(&registration_external);

        registration_external
    }

    pub fn get_registration(&self, registration_id: RegistrationId) -> RegistrationExternal {
        let registration_internal = RegistrationInternal::from(
            self.registrations_by_id
                .get(&registration_id)
                .expect("No registration found"),
        );
        format_registration(registration_id, registration_internal)
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
        list_id: Option<ListId>, // Optional for now because it has to be compatible with current Pot implementation of RegistryProvider, which calls a contract providing only "account_id" arg
        account_id: RegistrantId,
        required_status: Option<RegistrationStatus>,
    ) -> bool {
        let list_id = list_id.unwrap_or(1); // defaults to potlock public goods registry which is list ID 1
        let registration_ids = self.registration_ids_by_registrant_id.get(&account_id);
        let registration_ids_vec = if let Some(registration_ids) = registration_ids {
            registration_ids.to_vec()
        } else {
            vec![]
        };
        registration_ids_vec.into_iter().any(|registration_id| {
            let registration_internal = RegistrationInternal::from(
                self.registrations_by_id
                    .get(&registration_id)
                    .expect("No registration found"),
            );
            registration_internal.list_id == list_id
                && (registration_internal.status
                    == required_status.unwrap_or(RegistrationStatus::Approved))
        })
    }
}
