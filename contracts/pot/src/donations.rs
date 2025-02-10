use crate::*;

pub type DonationId = u64;

/// Donation (treasury pool or public round)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// ID of the donor
    pub donor_id: AccountId,
    /// Amount donated
    pub total_amount: u128,
    /// Amount after all fees/expenses (incl. storage)
    pub net_amount: u128,
    /// Optional message from the donor
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<u128>,
    /// Protocol fee
    pub protocol_fee: u128,

    /// pool donation
    pub treasury_pool: bool,
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
    /// Amount after all fees/expenses (incl. storage)
    pub net_amount: U128,
    /// Optional message from the donor
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Indicates whether this is treasury pool donation
    pub treasury_pool: bool,
}

pub const DONATION_ID_DELIMITER: &str = ":";

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
            .map(|(id, _v)| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_spending_pool_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.spending_pool_donation_ids.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.spending_pool_donation_ids
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|id| {
                self.format_donation(&Donation::from(self.donations_by_id.get(&id).unwrap()), id)
            })
            .collect()
    }

    pub fn get_treasury_pool_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.treasury_pool_donation_ids.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.treasury_pool_donation_ids
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
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        if let Some(donation_ids_by_project_set) = self.donation_ids_by_project_id.get(&project_id)
        {
            assert!(
                (donation_ids_by_project_set.len() as u128) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            donation_ids_by_project_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|id| {
                    self.format_donation(
                        &Donation::from(self.donations_by_id.get(&id).unwrap()),
                        id,
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        if let Some(donation_ids_by_donor_set) = self.donation_ids_by_donor_id.get(&donor_id) {
            assert!(
                (donation_ids_by_donor_set.len() as u64) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            donation_ids_by_donor_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|id| {
                    self.format_donation(
                        &Donation::from(self.donations_by_id.get(&id).unwrap()),
                        id,
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub(crate) fn calculate_fee(&self, amount: u128, basis_points: u32, is_protocol: bool) -> u128 {
        let total_basis_points = 10_000u128;
        let fee_amount = (basis_points as u128).saturating_mul(amount);
        if !is_protocol {
            // round down
            fee_amount / total_basis_points
        } else {
            // round up
            fee_amount.div_ceil(total_basis_points)
        }
    }

    pub(crate) fn calculate_referrer_fee(&self, amount: u128, treasury_pool: bool) -> u128 {
        let multiplier = if treasury_pool {
            self.referral_fee_treasury_pool_basis_points
        } else {
            self.referral_fee_spending_pool_basis_points
        };
        let referrer_amount = self.calculate_fee(amount, multiplier, false);
        referrer_amount
    }

    pub fn get_blacklisted_donors(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<AccountId> {
        if self.blacklisted_donors.is_empty() {
            return vec![];
        }
        let start_index = std::cmp::min(
            from_index.unwrap_or_default(),
            self.blacklisted_donors.len() - 1 as u64,
        );
        let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE as u64);
        self.blacklisted_donors
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .collect()
    }

    // WRITE METHODS

    #[payable]
    pub fn donate(
        &mut self,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: Option<bool>,
        bypass_protocol_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {
        let is_treasury_pool = treasury_pool.unwrap_or(false);
        self.assert_pot_active();
        let deposit = env::attached_deposit();
        self.assert_caller_can_donate(
            deposit,
            message,
            referrer_id,
            is_treasury_pool,
            bypass_protocol_fee,
        )
    }

    pub(crate) fn assert_caller_can_donate(
        &mut self,
        deposit: Balance,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: bool,
        bypass_protocol_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {
        let donor_id = env::predecessor_account_id();
        assert!(
            deposit >= self.min_treasury_donation_amount,
            "treasury pool donations must be at least {} yoctoNEAR",
            self.min_treasury_donation_amount
        );
        // treasury pool donations not subject to sybil checks, so move on to protocol fee handler
        self.handle_protocol_fee(
            deposit,
            donor_id,
            message.clone(),
            referrer_id.clone(),
            treasury_pool,
            bypass_protocol_fee,
        )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn sybil_callback(
        &mut self,
        deposit: Balance,
        donor_id: AccountId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: bool,
        bypass_protocol_fee: Option<bool>,
        #[callback_result] call_result: Result<bool, PromiseError>,
    ) -> PromiseOrValue<DonationExternal> {
        if call_result.is_err() {
            log!(format!(
                "Error verifying sybil check; returning donation {} to donor {}",
                deposit, donor_id
            ));
            Promise::new(donor_id).transfer(deposit);
            env::panic_str(
                "There was an error querying sybil check. Donation has been returned to donor.",
            );
        }
        let is_human: bool = call_result.unwrap();
        if !is_human {
            log!(format!(
                "Sybil provider wrapper check returned false; returning donation {} to donor {}",
                deposit, donor_id
            ));
            Promise::new(donor_id).transfer(deposit);
            env::panic_str(
                "Sybil provider wrapper check returned false. Donation has been returned to donor.",
            );
        } else {
            self.handle_protocol_fee(
                deposit,
                donor_id,
                message,
                referrer_id,
                treasury_pool,
                bypass_protocol_fee,
            )
        }
    }

    #[private]
    pub fn handle_protocol_fee(
        &mut self,
        deposit: Balance,
        donor_id: AccountId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: bool,
        bypass_protocol_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {
        if bypass_protocol_fee.unwrap_or(false) {
            // bypass protocol fee
            PromiseOrValue::Value(self.process_donation(
                deposit,
                donor_id,
                0,
                None,
                message.clone(),
                referrer_id.clone(),
                treasury_pool,
            ))
        } else if let Some(protocol_config_provider) = self.protocol_config_provider.get() {
            let (contract_id, method_name) = protocol_config_provider.decompose();
            let args = json!({}).to_string().into_bytes();
            PromiseOrValue::Promise(
                Promise::new(AccountId::new_unchecked(contract_id.clone()))
                    .function_call(method_name.clone(), args, 0, XCC_GAS)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(XCC_GAS)
                            .handle_protocol_fee_callback(
                                deposit,
                                donor_id,
                                message,
                                referrer_id,
                                treasury_pool,
                            ),
                    ),
            )
        } else {
            // bypass protocol fee
            PromiseOrValue::Value(self.process_donation(
                deposit,
                donor_id,
                0,
                None,
                message.clone(),
                referrer_id.clone(),
                treasury_pool,
            ))
        }
    }

    // calculate protocol fee callback
    #[private]
    pub fn handle_protocol_fee_callback(
        &mut self,
        deposit: Balance,
        donor_id: AccountId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: bool,
        #[callback_result] call_result: Result<ProtocolConfigProviderResult, PromiseError>,
    ) -> DonationExternal {
        if call_result.is_err() {
            log!(format!(
                "Error getting protocol fee; continuing with donation",
            ));
            self.process_donation(
                deposit,
                donor_id,
                0,
                None,
                message,
                referrer_id,
                treasury_pool,
            )
        } else {
            let protocol_config_provider_result = call_result.unwrap();
            let protocol_fee_basis_points = std::cmp::min(
                protocol_config_provider_result.basis_points,
                MAX_PROTOCOL_FEE_BASIS_POINTS,
            );
            let protocol_fee_recipient_account = protocol_config_provider_result.account_id;
            // calculate protocol fee (don't transfer yet)
            let protocol_fee = self.calculate_fee(deposit, protocol_fee_basis_points, true);
            self.process_donation(
                deposit,
                donor_id,
                protocol_fee,
                Some(protocol_fee_recipient_account),
                message,
                referrer_id,
                treasury_pool,
            )
        }
    }

    #[private]
    pub fn process_donation(
        &mut self,
        deposit: Balance,
        donor_id: AccountId,
        protocol_fee: u128,
        protocol_fee_recipient_account: Option<AccountId>,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        treasury_pool: bool,
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
            let referrer_fee_amount = self.calculate_referrer_fee(remainder, treasury_pool);
            referrer_fee = Some(U128::from(referrer_fee_amount));
            remainder = remainder.checked_sub(referrer_fee_amount).expect(&format!(
                "Overflow occurred when calculating remainder ({} - {})",
                remainder, referrer_fee_amount,
            ));
        }

        // insert mappings
        let donation_id = (self.donations_by_id.len() + 1) as DonationId;
        let mut donation = Donation {
            donor_id: donor_id.clone(),
            total_amount: deposit,
            net_amount: 0, // this will be updated in a moment after storage cost is subtracted
            message,
            donated_at: env::block_timestamp_ms(),
            protocol_fee,
            referrer_id: referrer_id.clone(),
            referrer_fee: referrer_fee.map(|v| v.0),
            treasury_pool,
        };
        self.insert_donation_record(&donation_id, &donation, treasury_pool);

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

        // update donation with net amount
        donation.net_amount = remainder;

        // update donation with net amount
        self.donations_by_id.insert(&donation_id, &donation);

        // update totals
        if treasury_pool {
            self.total_treasury_pool_donations =
                self.total_treasury_pool_donations
                    .checked_add(remainder)
                    .expect(&format!(
                        "Overflow occurred when calculating self.total_treasury_pool_donations ({} + {})",
                        self.total_treasury_pool_donations, remainder,
                    ));
            self.treasury_balance = self
                .treasury_balance
                .checked_add(remainder)
                .expect(&format!(
                    "Overflow occurred when calculating self.treasury_pool_balance ({} + {})",
                    self.treasury_balance, remainder,
                ));
        } else {
            self.total_spending_pool_donations = self
                .total_spending_pool_donations
                .checked_add(remainder)
                .expect(&format!(
                    "Overflow occurred when calculating self.total_public_donations ({} + {})",
                    self.total_spending_pool_donations, remainder,
                ));

            self.spending_balance = self
                .spending_balance
                .checked_add(remainder)
                .expect(&format!(
                    "Overflow occurred when calculating self.spending_pool_balance ({} + {})",
                    self.spending_balance, remainder,
                ));
        }

        // transfer protocol fee
        if let Some(protocol_fee_recipient_account) = protocol_fee_recipient_account {
            Promise::new(protocol_fee_recipient_account.clone()).transfer(protocol_fee);
        }

        // transfer referrer fee
        if let Some(referrer_fee) = referrer_fee {
            // it has already been established that referrer_id is Some
            Promise::new(referrer_id.expect("no referrer ID")).transfer(referrer_fee.0);
        }

        // return formatted donation
        self.format_donation(&donation, donation_id)
    }

    pub(crate) fn insert_donation_record(
        &mut self,
        donation_id: &DonationId,
        donation: &Donation,
        treasury_pool: bool,
    ) {
        // insert base donation record
        self.donations_by_id.insert(donation_id, &donation);

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

        // add to public round or treasury pool donation ids
        if treasury_pool {
            self.treasury_pool_donation_ids.insert(donation_id);
        } else {
            self.spending_pool_donation_ids.insert(donation_id);
        }
    }

    pub fn format_donation(&self, donation: &Donation, id: DonationId) -> DonationExternal {
        DonationExternal {
            id,
            donor_id: donation.donor_id.clone(),
            total_amount: U128(donation.total_amount),
            net_amount: U128(donation.net_amount),
            message: donation.message.clone(),
            donated_at: donation.donated_at,
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.map(U128),
            protocol_fee: U128(donation.protocol_fee),
            treasury_pool: self.treasury_pool_donation_ids.contains(&id),
        }
    }
}
