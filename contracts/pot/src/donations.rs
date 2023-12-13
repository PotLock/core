use crate::*;
/// Donation (matching pool or public round)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation, if applicable (matching pool donations will contain `None`)
    pub project_id: Option<ProjectId>,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Protocol fee
    pub protocol_fee: U128,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedDonation {
    Current(Donation),
}

impl From<VersionedDonation> for Donation {
    fn from(donation: VersionedDonation) -> Self {
        match donation {
            VersionedDonation::Current(current) => current,
        }
    }
}

/// Ephemeral-only (used in views)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DonationExternal {
    /// ID of the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation, if applicable (matching pool donations will contain `None`)
    pub project_id: Option<ProjectId>,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Indicates whether this is matching pool donation
    pub matching_pool: bool,
}

pub const DONATION_ID_DELIMETER: &str = ":";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProtocolConfigProviderResult {
    pub basis_points: u32,
    pub account_id: AccountId,
}

#[near_bindgen]
impl Contract {
    // GETTERS

    pub fn get_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.donations_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(id, v)| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_public_round_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.public_round_donation_ids.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.public_round_donation_ids
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|id| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_matching_pool_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.matching_pool_donation_ids.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.matching_pool_donation_ids
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|id| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_donations_for_project(
        &self,
        project_id: ProjectId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_project_set = self.donation_ids_by_project_id.get(&project_id).unwrap();
        donation_ids_by_project_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|id| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_donor_set = self.donation_ids_by_donor_id.get(&donor_id).unwrap();
        donation_ids_by_donor_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|id| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_matching_pool_balance(&self) -> U128 {
        self.matching_pool_balance
    }

    pub fn get_total_donations(&self) -> U128 {
        self.total_donations
    }

    pub(crate) fn calculate_referrer_fee(&self, amount: u128, matching_pool: bool) -> u128 {
        let total_basis_points = 10_000u128;
        let amount_per_basis_point = amount / total_basis_points;
        let multiplier = if matching_pool {
            self.patron_referral_fee_basis_points
        } else {
            self.public_round_referral_fee_basis_points
        };
        let referrer_amount = multiplier as u128 * amount_per_basis_point;
        referrer_amount
    }

    // WRITE METHODS

    #[payable]
    pub fn donate(
        &mut self,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: Option<bool>,
    ) -> Promise {
        if let Some(project_id) = project_id.clone() {
            self.assert_approved_application(&project_id);
        };
        let is_matching_pool = matching_pool.unwrap_or(false);
        if !is_matching_pool {
            self.assert_round_active();
            // error if this is an end-user donation and no project_id is provided
            if project_id.is_none() {
                env::panic_str(
                    "project_id argument must be provided for public (non-matching pool) donations",
                );
            }
        }
        let deposit = env::attached_deposit();
        self.assert_caller_can_donate(deposit, project_id, message, referrer_id, is_matching_pool)
    }

    pub(crate) fn assert_caller_can_donate(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
    ) -> Promise {
        let always_allow_cb_promise = Self::ext(env::current_account_id())
            .with_static_gas(XCC_GAS)
            .sybil_always_allow_callback(
                deposit,
                project_id.clone(),
                message.clone(),
                referrer_id.clone(),
                matching_pool,
            );

        if matching_pool {
            assert!(
                deposit >= self.min_matching_pool_donation_amount.0,
                "Matching pool donations must be at least {} yoctoNEAR",
                self.min_matching_pool_donation_amount.0
            );
            // matching pool donations not subject to sybil checks, so go to always_allow callback
            always_allow_cb_promise
        } else {
            if let Some(sybil_wrapper_provider) = self.sybil_wrapper_provider.get() {
                let (contract_id, method_name) = sybil_wrapper_provider.decompose();
                let args = json!({ "account_id": env::predecessor_account_id() })
                    .to_string()
                    .into_bytes();
                Promise::new(AccountId::new_unchecked(contract_id.clone()))
                    .function_call(method_name, args, 0, XCC_GAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(XCC_GAS)
                            .sybil_callback(
                                deposit,
                                project_id.clone(),
                                message.clone(),
                                referrer_id.clone(),
                                matching_pool,
                            ),
                    )
            } else {
                // no sybil wrapper provider, so go to always_allow callback
                always_allow_cb_promise
            }
        }
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn sybil_always_allow_callback(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
    ) -> Promise {
        self.handle_protocol_fee(
            // caller_id,
            // amount,
            deposit,
            project_id,
            message,
            referrer_id,
            matching_pool,
        )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn sybil_callback(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
        #[callback_result] call_result: Result<bool, PromiseError>,
    ) -> Promise {
        let caller_id = env::predecessor_account_id();
        let amount = env::attached_deposit();
        if call_result.is_err() {
            log!(format!(
                "Error verifying sybil check; returning donation {} to donor {}",
                amount, caller_id
            ));
            Promise::new(caller_id).transfer(amount);
            env::panic_str(
                "There was an error querying sybil check. Donation has been returned to donor.",
            );
        }
        let is_human: bool = call_result.unwrap();
        if !is_human {
            log!(format!(
                "Sybil provider wrapper check returned false; returning donation {} to donor {}",
                amount, caller_id
            ));
            Promise::new(caller_id).transfer(amount);
            env::panic_str(
                "Sybil provider wrapper check returned false. Donation has been returned to donor.",
            );
        } else {
            self.handle_protocol_fee(
                // caller_id,
                // amount,
                deposit,
                project_id,
                message,
                referrer_id,
                matching_pool,
            )
        }
    }

    pub(crate) fn handle_protocol_fee(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
    ) -> Promise {
        let bypass_protocol_fee_promise = Self::ext(env::current_account_id())
            .with_static_gas(XCC_GAS)
            .bypass_protocol_fee(
                deposit,
                project_id.clone(),
                message.clone(),
                referrer_id.clone(),
                matching_pool,
            );
        if matching_pool {
            // protocol fee is only paid for matching pool donations
            if let Some(protocol_config_provider) = self.protocol_config_provider.get() {
                let (contract_id, method_name) = protocol_config_provider.decompose();
                let args = json!({}).to_string().into_bytes();
                Promise::new(AccountId::new_unchecked(contract_id.clone()))
                    .function_call(method_name.clone(), args, 0, XCC_GAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(XCC_GAS)
                            .handle_protocol_fee_callback(
                                deposit,
                                project_id,
                                message,
                                referrer_id,
                                matching_pool,
                            ),
                    )
            } else {
                // bypass protocol fee
                bypass_protocol_fee_promise
            }
        } else {
            // bypass protocol fee
            bypass_protocol_fee_promise
        }
    }

    pub(crate) fn calculate_fee(&self, amount: u128, basis_points: u32) -> u128 {
        let total_basis_points = 10_000u128;
        let amount_per_basis_point = amount / total_basis_points;
        basis_points as u128 * amount_per_basis_point
    }

    // calculate protocol fee callback
    #[private]
    pub fn handle_protocol_fee_callback(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
        #[callback_result] call_result: Result<ProtocolConfigProviderResult, PromiseError>,
    ) -> DonationExternal {
        if call_result.is_err() {
            log!(format!(
                "Error getting protocol fee; continuing with donation",
            ));
            self.process_donation(
                deposit,
                0,
                None,
                project_id,
                message,
                referrer_id,
                matching_pool,
            )
        } else {
            let protocol_config_provider_result = call_result.unwrap();
            let protocol_fee_basis_points = protocol_config_provider_result.basis_points;
            let protocol_fee_recipient_account = protocol_config_provider_result.account_id;
            // calculate protocol fee (don't transfer yet)
            let protocol_fee =
                self.calculate_fee(env::attached_deposit(), protocol_fee_basis_points);
            self.process_donation(
                deposit,
                protocol_fee,
                Some(protocol_fee_recipient_account),
                project_id,
                message,
                referrer_id,
                matching_pool,
            )
        }
    }

    #[private]
    pub fn bypass_protocol_fee(
        &mut self,
        deposit: Balance,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
    ) -> DonationExternal {
        self.process_donation(
            deposit,
            0,
            None,
            project_id,
            message,
            referrer_id,
            matching_pool,
        )
    }

    pub(crate) fn process_donation(
        &mut self,
        deposit: Balance,
        protocol_fee: u128,
        protocol_fee_recipient_account: Option<AccountId>,
        project_id: Option<ProjectId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        matching_pool: bool,
    ) -> DonationExternal {
        let initial_storage_usage = env::storage_usage();

        // subtract protocol fee
        let mut remainder = deposit.checked_sub(protocol_fee).expect(&format!(
            "Overflow occurred when calculating remainder ({} - {})",
            deposit, protocol_fee,
        ));

        // subtract referrer fee
        let mut referrer_fee: Option<U128> = None;
        if let Some(_referrer_id) = referrer_id.clone() {
            let referrer_fee_amount = self.calculate_referrer_fee(remainder, matching_pool);
            referrer_fee = Some(U128::from(referrer_fee_amount));
            remainder = remainder.checked_sub(referrer_fee_amount).expect(&format!(
                "Overflow occurred when calculating remainder ({} - {})",
                remainder, referrer_fee_amount,
            ));
        }

        // insert mappings
        let donation_id = (self.donations_by_id.len() + 1) as DonationId;
        let donation = Donation {
            donor_id: env::signer_account_id(),
            total_amount: U128::from(deposit),
            message,
            donated_at: env::block_timestamp(),
            project_id: project_id.clone(),
            protocol_fee: U128::from(protocol_fee),
            referrer_id: referrer_id.clone(),
            referrer_fee,
        };
        self.insert_donation_record(&donation_id, &donation, matching_pool);
        self.total_donations = U128::from(self.total_donations.0.checked_add(remainder).expect(
            &format!(
                "Overflow occurred when calculating self.donations_balance ({} + {})",
                self.total_donations.0, remainder,
            ),
        ));

        // assert that donation after fees > storage cost
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        require!(
            remainder > required_deposit,
            format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            )
        );

        // subtract storage cost
        remainder = remainder.checked_sub(required_deposit).expect(&format!(
            "Overflow occurred when calculating remainder ({} - {})",
            remainder, required_deposit,
        ));

        // transfer protocol fee
        if let Some(protocol_fee_recipient_account) = protocol_fee_recipient_account {
            Promise::new(protocol_fee_recipient_account.clone()).transfer(protocol_fee);
        }

        // transfer referrer fee
        if let Some(referrer_fee) = referrer_fee {
            // it has already been established that referrer_id is Some
            Promise::new(referrer_id.unwrap()).transfer(referrer_fee.0);
        }

        // transfer remainder to project
        if let Some(project_id) = project_id {
            Promise::new(project_id.clone()).transfer(remainder);
        }

        // return formatted donation
        self.format_donation(&donation, donation_id)
    }

    pub(crate) fn insert_donation_record(
        &mut self,
        donation_id: &DonationId,
        donation: &Donation,
        matching_pool: bool,
    ) {
        // insert base donation record
        self.donations_by_id
            .insert(donation_id, &VersionedDonation::Current(donation.clone()));

        // if donation has a project_id, add to relevant mappings
        if let Some(project_id) = donation.project_id.clone() {
            let mut donation_ids_by_project_set = if let Some(donation_ids_by_application_set) =
                self.donation_ids_by_project_id.get(&project_id)
            {
                donation_ids_by_application_set
            } else {
                UnorderedSet::new(StorageKey::DonationIdsByProjectIdInner {
                    project_id: project_id.clone(),
                })
            };
            donation_ids_by_project_set.insert(donation_id);
            self.donation_ids_by_project_id
                .insert(&project_id, &donation_ids_by_project_set);
        }

        // add to donations-by-donor mapping
        let mut donation_ids_by_donor_set = if let Some(donation_ids_by_donor_set) =
            self.donation_ids_by_donor_id.get(&donation.donor_id)
        {
            donation_ids_by_donor_set
        } else {
            UnorderedSet::new(StorageKey::DonationIdsByDonorIdInner {
                donor_id: donation.donor_id.clone(),
            })
        };
        donation_ids_by_donor_set.insert(donation_id);
        self.donation_ids_by_donor_id
            .insert(&donation.donor_id, &donation_ids_by_donor_set);

        // add to public round or matching pool donation ids
        // TODO: consider determining this based on Donation.project_id instead of matching_pool boolean
        if matching_pool {
            self.matching_pool_donation_ids.insert(donation_id);
        } else {
            self.public_round_donation_ids.insert(donation_id);
        }
    }

    pub fn format_donation(&self, donation: &Donation, id: DonationId) -> DonationExternal {
        DonationExternal {
            id,
            donor_id: donation.donor_id.clone(),
            total_amount: donation.total_amount,
            message: donation.message.clone(),
            donated_at: donation.donated_at,
            project_id: donation.project_id.clone(),
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.clone(),
            protocol_fee: donation.protocol_fee,
            matching_pool: self.matching_pool_donation_ids.contains(&id),
        }
    }
}
