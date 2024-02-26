use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StampId(pub String); // "{USER_ID}#{PROVIDER_ID}"

const STAMP_ID_DELIMITER: &str = "#"; // separates user_id and provider_id in StampId. * NB: should not be the same as PROVIDER_ID_DELIMITER (currently set to ":")

impl StampId {
    // Generate StampId ("{USER_ID}#{PROVIDER_ID}") from user_id and provider_id
    pub fn new(user_id: AccountId, provider_id: ProviderId) -> Self {
        StampId(format!(
            "{}{}{}",
            user_id, STAMP_ID_DELIMITER, provider_id.0
        ))
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Stamp {
    // stored at user_id#provider_id
    pub validated_at_ms: TimestampMs,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedStamp {
    Current(Stamp),
}

impl From<VersionedStamp> for Stamp {
    fn from(stamp: VersionedStamp) -> Self {
        match stamp {
            VersionedStamp::Current(current) => current,
        }
    }
}

/// Ephermal stamp data returned to user (not stored in contract)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct StampExternal {
    user_id: AccountId,
    provider: ProviderExternal,
    validated_at_ms: TimestampMs,
}

#[near_bindgen]
impl Contract {
    /// Add a stamp for a user
    // TODO: consider adding force_refresh param that defaults to false, which if present won't trigger an error if the user already has this stamp
    #[payable]
    pub fn add_stamp(&mut self, provider_id: ProviderId) -> Promise {
        let user_id = env::signer_account_id();
        let attached_deposit = env::attached_deposit();
        // get provider, verify it exists
        let provider = Provider::from(
            self.providers_by_id
                .get(&provider_id)
                .expect("Provider does not exist"),
        );
        // verify that provider is active
        assert!(
            provider.status == ProviderStatus::Active,
            "Provider is not active"
        );
        // verify against provider, using custom gas if specified
        let (contract_id, method_name) = provider_id.decompose();
        let gas = Gas(provider.gas.unwrap_or(XCC_GAS_DEFAULT));

        // Create a HashMap and insert the dynamic account_id_arg_name and value
        let mut args_map = std::collections::HashMap::new();
        args_map.insert(provider.account_id_arg_name.clone(), user_id.to_string());

        // Serialize the HashMap to JSON string and then to bytes
        let args = near_sdk::serde_json::to_string(&args_map)
            .expect("Failed to serialize args")
            .into_bytes();

        Promise::new(AccountId::new_unchecked(contract_id.clone()))
            .function_call(method_name.clone(), args, NO_DEPOSIT, gas)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(gas)
                    .verify_stamp_callback(user_id, provider_id, provider, attached_deposit),
            )
    }

    #[private]
    pub fn verify_stamp_callback(
        &mut self,
        user_id: AccountId,
        provider_id: ProviderId,
        mut provider: Provider,
        attached_deposit: Balance,
        #[callback_result] call_result: Result<near_sdk::serde_json::Value, PromiseError>,
    ) -> Option<StampExternal> {
        match call_result {
            Ok(val) => {
                if let Some(is_valid) = val.as_bool() {
                    // provider returned a bool; so far so good
                    if !is_valid {
                        // provider returned false (user not verified); refund deposit
                        log!("User not verified; refunding deposit");
                        Promise::new(user_id).transfer(attached_deposit);
                        return None;
                    } else {
                        // provider returned true (user verified); create stamp
                        log!(format!("User verified; creating stamp",));
                        let stamp_id = StampId::new(user_id.clone(), provider_id.clone());
                        let stamp = Stamp {
                            validated_at_ms: env::block_timestamp_ms(),
                        };

                        // update state
                        let initial_storage_usage = env::storage_usage();
                        self.insert_stamp_record(
                            stamp_id.clone(),
                            stamp.clone(),
                            provider_id.clone(),
                            user_id.clone(),
                        );

                        provider.stamp_count += 1;
                        self.providers_by_id
                            .insert(&provider_id, &VersionedProvider::Current(provider.clone()));

                        // calculate storage cost
                        let required_deposit =
                            calculate_required_storage_deposit(initial_storage_usage);
                        // refund any unused deposit
                        if attached_deposit > required_deposit {
                            Promise::new(user_id.clone())
                                .transfer(attached_deposit - required_deposit);
                        } else if attached_deposit < required_deposit {
                            env::panic_str(&format!(
                                "Must attach {} yoctoNEAR to cover storage",
                                required_deposit
                            ));
                        }

                        // return stamp
                        return Some(StampExternal {
                            user_id,
                            provider: ProviderExternal::from_provider_id(&provider_id.0, provider),
                            validated_at_ms: stamp.validated_at_ms,
                        });
                    }
                } else {
                    // Response type is incorrect. Refund deposit.
                    log!(
                        "Received invalid response type for stamp verification. Returning deposit."
                    );
                    Promise::new(user_id).transfer(attached_deposit);
                    return None;
                }
            }
            Err(_) => {
                // Error occurred in cross-contract call. Refund deposit.
                log!("Error occurred while verifying stamp; refunding deposit");
                Promise::new(user_id).transfer(attached_deposit);
                return None;
            }
        }
    }

    pub(crate) fn insert_stamp_record(
        &mut self,
        stamp_id: StampId,
        stamp: Stamp,
        provider_id: ProviderId,
        user_id: AccountId,
    ) {
        // insert base stamp record
        self.stamps_by_id
            .insert(&stamp_id, &VersionedStamp::Current(stamp));

        // add to provider_ids_for_user mapping
        let mut provider_ids_for_user_set =
            if let Some(provider_ids_for_user_set) = self.provider_ids_for_user.get(&user_id) {
                provider_ids_for_user_set
            } else {
                UnorderedSet::new(StorageKey::ProviderIdsForUserInner {
                    user_id: user_id.clone(),
                })
            };
        provider_ids_for_user_set.insert(&provider_id);
        self.provider_ids_for_user
            .insert(&user_id, &provider_ids_for_user_set);

        // add to user_ids_for_provider mapping
        let mut user_ids_for_provider_set =
            if let Some(user_ids_for_provider_set) = self.user_ids_for_provider.get(&provider_id) {
                user_ids_for_provider_set
            } else {
                UnorderedSet::new(StorageKey::UserIdsForProviderInner {
                    provider_id: provider_id.clone(),
                })
            };
        user_ids_for_provider_set.insert(&user_id);
        self.user_ids_for_provider
            .insert(&provider_id, &user_ids_for_provider_set);
    }

    pub(crate) fn delete_stamp_record(
        &mut self,
        stamp_id: StampId,
        stamp: Stamp,
        provider_id: ProviderId,
        user_id: AccountId,
    ) {
        // delete base stamp record
        self.stamps_by_id.remove(&stamp_id);

        // remove from provider_ids_for_user mapping
        let mut provider_ids_for_user_set = self
            .provider_ids_for_user
            .get(&user_id)
            .expect("No provider IDs for user");
        provider_ids_for_user_set.remove(&provider_id);
        self.provider_ids_for_user
            .insert(&user_id, &provider_ids_for_user_set);

        // remove from user_ids_for_provider mapping
        let mut user_ids_for_provider_set = self
            .user_ids_for_provider
            .get(&provider_id)
            .expect("No user Ids for provider");
        user_ids_for_provider_set.remove(&user_id);
        self.user_ids_for_provider
            .insert(&provider_id, &user_ids_for_provider_set);
    }

    pub fn delete_stamp(&mut self, provider_id: ProviderId) {
        let user_id = env::signer_account_id();
        let stamp_id = StampId::new(user_id.clone(), provider_id.clone());
        let stamp = Stamp::from(
            self.stamps_by_id
                .get(&stamp_id)
                .expect("Stamp does not exist"),
        );
        let mut provider = Provider::from(
            self.providers_by_id
                .get(&provider_id)
                .expect("Provider does not exist"),
        );

        // update state
        let attached_deposit = env::attached_deposit();
        let initial_storage_usage = env::storage_usage();
        self.delete_stamp_record(
            stamp_id.clone(),
            stamp.clone(),
            provider_id.clone(),
            user_id.clone(),
        );

        provider.stamp_count -= 1;
        self.providers_by_id
            .insert(&provider_id, &VersionedProvider::Current(provider.clone()));

        // refund user for freed storage
        let storage_freed = initial_storage_usage - env::storage_usage();
        log!(format!("Storage freed: {} bytes", storage_freed));
        let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
        Promise::new(user_id.clone()).transfer(cost_freed + attached_deposit);
    }

    pub fn get_stamps_for_account_id(
        &self,
        account_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<StampExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        if let Some(account_id_stamp_set) = self.provider_ids_for_user.get(&account_id) {
            assert!(
                (account_id_stamp_set.len() as u128) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
            assert_ne!(limit, 0, "Cannot provide limit of 0.");
            account_id_stamp_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|provider_id| {
                    let stamp_id = StampId::new(account_id.clone(), provider_id.clone());
                    let stamp = Stamp::from(
                        self.stamps_by_id
                            .get(&stamp_id)
                            .expect("Stamp does not exist"),
                    );
                    StampExternal {
                        user_id: account_id.clone(),
                        provider: ProviderExternal::from_provider_id(
                            &provider_id.0,
                            Provider::from(
                                self.providers_by_id
                                    .get(&provider_id)
                                    .expect("Provider does not exist"),
                            ),
                        ),
                        validated_at_ms: stamp.validated_at_ms,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_users_for_stamp(
        &self,
        provider_id: ProviderId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<AccountId> {
        let start_index: u128 = from_index.unwrap_or_default();
        if let Some(provider_id_user_set) = self.user_ids_for_provider.get(&provider_id) {
            assert!(
                (provider_id_user_set.len() as u128) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
            assert_ne!(limit, 0, "Cannot provide limit of 0.");
            provider_id_user_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_providers_submitted_by_user(
        &self,
        account_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<ProviderExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        if let Some(providers_for_submitter_set) = self.provider_ids_for_submitter.get(&account_id)
        {
            assert!(
                (providers_for_submitter_set.len() as u128) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
            assert_ne!(limit, 0, "Cannot provide limit of 0.");
            providers_for_submitter_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|provider_id| {
                    ProviderExternal::from_provider_id(
                        &provider_id.0,
                        Provider::from(
                            self.providers_by_id
                                .get(&provider_id)
                                .expect("Provider does not exist"),
                        ),
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }
}

// impl StampExternal {
// TODO: WIP
//     pub fn new(user_id: AccountId, provider_id: ProviderExternal, validated_at: TimestampMs) -> Self {
//         Self {
//             user_id,
//             provider,
//             validated_at,
//         }
//     }
// }

// stamp added for user
// -> stamp added to stamps_by_id (indexed at user_id#provider_id)
// -> provider ID added to provider_ids_for_user set
// stamps for user
// -> provider_ids_for_user set
// -> fetch stamp from UnorderedMap using user_id#provider_id

// fetch all users with given stamp (provider ID)
// -> user_ids_for_provider sets (Lookupmap -> UnorderedSet)

// fetch providers that a user has submitted (e.g. if user has submitted one malicious provider, they are likely to submit more and you'll want to be able to fetch these or filter them out of results)
// -> submitter_ids_for_provider sets (Lookupmap -> UnorderedSet)
