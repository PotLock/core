use crate::*;

// Donation is the data structure that is stored within the contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// FT id (e.g. "near")
    pub ft_id: AccountId,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// ID of the account receiving the donation  
    pub recipient_id: AccountId,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
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

#[near_bindgen]
impl Contract {
    pub fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> String {
        let ft_id = env::predecessor_account_id();
        // deconstruct msg, should contain recipient_id and optional referrer_id
        let mut recipient_id = None;
        let mut referrer_id = None;
        let mut message = None;
        let mut parts = msg.split("|");
        if let Some(recipient_id_str) = parts.next() {
            if recipient_id_str != "" {
                recipient_id = Some(AccountId::new_unchecked(recipient_id_str.to_string()));
            }
        }
        if let Some(referrer_id_str) = parts.next() {
            if referrer_id_str != "" {
                referrer_id = Some(AccountId::new_unchecked(referrer_id_str.to_string()));
            }
        }
        if let Some(message_str) = parts.next() {
            if message_str != "" {
                message = Some(message_str.to_string());
            }
        }
        log!(format!(
            "Recipient ID {:?}, Referrer ID {:?}, Amount {}, Message {:?}",
            recipient_id, referrer_id, amount.0, message
        ));

        if let Some(recipient_id) = recipient_id {
            // store donation to see how much storage was used
            // check that user has paid enough to cover storage
            // panicking will cancel the transfer on the FT contract
            let initial_storage_usage = env::storage_usage();
            // calculate fees
            // calculate protocol fee
            let mut remainder = amount.0;
            let protocol_fee = self.calculate_protocol_fee(amount.0);
            remainder -= protocol_fee;

            // calculate referrer fee, if applicable
            let mut referrer_fee = None;
            if referrer_id.is_some() {
                let referrer_amount = self.calculate_referrer_fee(amount.0);
                remainder -= referrer_amount;
                referrer_fee = Some(U128::from(referrer_amount));
            }

            // format donation record
            let donation = Donation {
                id: self.next_donation_id,
                donor_id: sender_id.clone(),
                total_amount: amount,
                ft_id: ft_id.clone(),
                message,
                donated_at_ms: env::block_timestamp_ms(),
                recipient_id: recipient_id.clone(),
                protocol_fee: U128::from(protocol_fee),
                referrer_id: referrer_id.clone(),
                referrer_fee,
            };

            // increment next_donation_id
            self.next_donation_id += 1;

            // insert mapping records
            self.insert_donation_record(&donation);

            // check that user has enough storage deposit
            let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
            let storage_balance = self.storage_balance_of(&sender_id);
            assert!(
                storage_balance.0 >= required_deposit,
                "{} must add storage deposit of at least {} yoctoNEAR to cover Donation storage",
                sender_id,
                required_deposit
            );

            // deduct storage deposit from user's balance
            let new_storage_balance = storage_balance.0 - required_deposit;
            self.storage_deposits
                .insert(&sender_id, &new_storage_balance);

            // transfer fees
            // transfer protocol fee
            log!(format!(
                "Transferring protocol fee {} ({}) to {}",
                protocol_fee, ft_id, self.protocol_fee_recipient_account
            ));
            let protocol_fee_transfer_args: Vec<u8> = json!({ "receiver_id": self.protocol_fee_recipient_account.clone(), "amount": U128(protocol_fee) })
                .to_string()
                .into_bytes();
            Promise::new(AccountId::new_unchecked(ft_id.to_string()))
                .function_call(
                    "ft_transfer".to_string(),
                    protocol_fee_transfer_args,
                    ONE_YOCTO,
                    Gas(XCC_GAS_DEFAULT),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas(XCC_GAS_DEFAULT))
                        .transfer_funds_callback(
                            remainder,
                            donation.clone(),
                            Some(ft_id.clone()),
                            false,
                            true,
                            false,
                        ),
                );

            // transfer referrer fee
            if let Some(referrer_id) = referrer_id {
                log!(format!(
                    "Transferring referrer fee {:?} ({}) to {}",
                    referrer_fee.clone().unwrap(),
                    ft_id,
                    referrer_id
                ));
                let referrer_fee_transfer_args =
                    json!({ "receiver_id": referrer_id, "amount": referrer_fee })
                        .to_string()
                        .into_bytes();
                Promise::new(AccountId::new_unchecked(ft_id.to_string()))
                    .function_call(
                        "ft_transfer".to_string(),
                        referrer_fee_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(XCC_GAS_DEFAULT))
                            .transfer_funds_callback(
                                remainder,
                                donation.clone(),
                                Some(ft_id.clone()),
                                false,
                                false,
                                true,
                            ),
                    );
            }

            // transfer donation
            log!(format!(
                "Transferring donation {} ({}) to {}",
                remainder, ft_id, recipient_id
            ));
            let donation_transfer_args =
                json!({ "receiver_id": recipient_id, "amount": U128(remainder) })
                    .to_string()
                    .into_bytes();
            Promise::new(AccountId::new_unchecked(ft_id.to_string()))
                .function_call(
                    "ft_transfer".to_string(),
                    donation_transfer_args,
                    ONE_YOCTO,
                    Gas(XCC_GAS_DEFAULT),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas(XCC_GAS_DEFAULT))
                        .transfer_funds_callback(
                            remainder,
                            donation.clone(),
                            Some(ft_id.clone()),
                            true,
                            false,
                            false,
                        ),
                );

            // return # unused tokens as per NEP-144 standard
            "0".to_string()
        } else {
            panic!("Must provide recipient ID in msg");
        }
    }

    #[payable]
    pub fn donate(
        &mut self,
        recipient_id: AccountId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
    ) -> Donation {
        // user has to pay for storage
        let initial_storage_usage = env::storage_usage();

        // calculate fees
        // calculate protocol fee
        let amount = env::attached_deposit();
        let mut remainder = amount;
        let protocol_fee = self.calculate_protocol_fee(amount);
        remainder -= protocol_fee;

        // calculate referrer fee, if applicable
        let mut referrer_fee = None;
        if let Some(_referrer_id) = referrer_id.clone() {
            let referrer_amount = self.calculate_referrer_fee(amount);
            remainder -= referrer_amount;
            referrer_fee = Some(U128::from(referrer_amount));
        }

        // format donation record
        let donation = Donation {
            id: self.next_donation_id,
            donor_id: env::predecessor_account_id(),
            total_amount: U128::from(amount),
            ft_id: AccountId::new_unchecked("near".to_string()), // for now, only NEAR is supported
            message,
            donated_at_ms: env::block_timestamp_ms(),
            recipient_id: recipient_id.clone(),
            protocol_fee: U128::from(protocol_fee),
            referrer_id: referrer_id.clone(),
            referrer_fee,
        };

        // insert mapping records
        self.insert_donation_record(&donation);

        // assert that donation after fees > storage cost
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        require!(
            remainder > required_deposit,
            format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            )
        );
        remainder -= required_deposit;

        // transfer fees
        // transfer protocol fee
        log!(format!(
            "Transferring protocol fee {} to {}",
            protocol_fee, self.protocol_fee_recipient_account
        ));
        Promise::new(self.protocol_fee_recipient_account.clone())
            .transfer(protocol_fee)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                    .transfer_funds_callback(remainder, donation.clone(), None, false, true, false),
            );

        // transfer referrer fee
        if let (Some(referrer_fee), Some(referrer_id)) = (referrer_fee, referrer_id) {
            log!(format!(
                "Transferring referrer fee {} to {}",
                referrer_fee.0, referrer_id
            ));
            Promise::new(referrer_id).transfer(referrer_fee.0).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                    .transfer_funds_callback(remainder, donation.clone(), None, false, false, true),
            );
        }

        // transfer donation
        log!(format!(
            "Transferring donation {} to {}",
            remainder, recipient_id
        ));
        Promise::new(recipient_id).transfer(remainder).then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(XCC_GAS_DEFAULT))
                .transfer_funds_callback(remainder, donation.clone(), None, true, false, false),
        );

        // return donation
        donation
    }

    /// Verifies whether donation & fees have been paid out for a given donation
    #[private]
    pub fn transfer_funds_callback(
        &mut self,
        remainder: Balance,
        mut donation: Donation,
        ft_id: Option<AccountId>,
        is_donation_transfer: bool,
        is_protocol_fee_transfer: bool,
        is_referrer_fee_transfer: bool,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            if is_donation_transfer {
                log!(format!(
                    "Error transferring donation {} to {}. Returning funds to donor.",
                    remainder, donation.recipient_id
                ));
                // return funds to donor
                if let Some(ft_id) = ft_id {
                    let donation_transfer_args =
                        json!({ "receiver_id": donation.donor_id, "amount": U128(remainder) })
                            .to_string()
                            .into_bytes();
                    Promise::new(AccountId::new_unchecked(ft_id.to_string())).function_call(
                        "ft_transfer".to_string(),
                        donation_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    );
                } else {
                    Promise::new(donation.donor_id.clone()).transfer(remainder);
                }
                // delete donation record, and refund freed storage cost to donor's storage balance
                let initial_storage_usage = env::storage_usage();
                self.donations_by_id.remove(&donation.id);
                let storage_freed = initial_storage_usage - env::storage_usage();
                let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
                let storage_balance = self.storage_balance_of(&donation.donor_id);
                let new_storage_balance = storage_balance.0 + cost_freed;
                self.storage_deposits
                    .insert(&donation.donor_id, &new_storage_balance); // TODO: check if this is hackable, e.g. if user can withdraw all their storage before this callback runs and therefore get a higher refund
            } else if is_protocol_fee_transfer {
                log!(format!(
                    "Error transferring protocol fee {:?} to {}. Returning funds to donor.",
                    donation.protocol_fee, self.protocol_fee_recipient_account
                ));
                // return funds to donor
                if let Some(ft_id) = ft_id {
                    let donation_transfer_args =
                        json!({ "receiver_id": donation.donor_id, "amount": donation.protocol_fee })
                            .to_string()
                            .into_bytes();
                    Promise::new(AccountId::new_unchecked(ft_id.to_string())).function_call(
                        "ft_transfer".to_string(),
                        donation_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    );
                } else {
                    Promise::new(donation.donor_id.clone()).transfer(donation.protocol_fee.0);
                }
                // update fee on Donation record to indicate error transferring funds
                donation.protocol_fee = U128(0);
                self.donations_by_id
                    .insert(&donation.id.clone(), &VersionedDonation::Current(donation));
            } else if is_referrer_fee_transfer {
                log!(format!(
                    "Error transferring referrer fee {:?} to {:?}. Returning funds to donor.",
                    donation.referrer_fee, donation.referrer_id
                ));
                // return funds to donor
                if let Some(ft_id) = ft_id {
                    let donation_transfer_args =
                        json!({ "receiver_id": donation.donor_id, "amount": donation.referrer_fee })
                            .to_string()
                            .into_bytes();
                    Promise::new(AccountId::new_unchecked(ft_id.to_string())).function_call(
                        "ft_transfer".to_string(),
                        donation_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    );
                } else {
                    Promise::new(donation.donor_id.clone())
                        .transfer(donation.referrer_fee.unwrap().0);
                }
                // update fee on Donation record to indicate error transferring funds
                donation.referrer_fee = Some(U128(0));
                self.donations_by_id
                    .insert(&donation.id.clone(), &VersionedDonation::Current(donation));
            }
        } else {
            if is_donation_transfer {
                // log event
                log_donation_event(&donation);
            }
        }
    }

    pub(crate) fn calculate_protocol_fee(&self, amount: u128) -> u128 {
        let total_basis_points = 10_000u128;
        let fee_amount = self.protocol_fee_basis_points as u128 * amount;
        // Round up
        fee_amount.div_ceil(total_basis_points)
    }

    pub(crate) fn calculate_referrer_fee(&self, amount: u128) -> u128 {
        let total_basis_points = 10_000u128;
        let fee_amount = self.referral_fee_basis_points as u128 * amount;
        // Round down
        fee_amount / total_basis_points
    }

    pub(crate) fn insert_donation_record(&mut self, donation: &Donation) {
        self.donations_by_id
            .insert(&donation.id, &VersionedDonation::Current(donation.clone()));
        // add to donations-by-recipient mapping
        let mut donation_ids_by_recipient_set = if let Some(donation_ids_by_recipient_set) = self
            .donation_ids_by_recipient_id
            .get(&donation.recipient_id)
        {
            donation_ids_by_recipient_set
        } else {
            UnorderedSet::new(StorageKey::DonationIdsByRecipientIdInner {
                recipient_id: donation.recipient_id.clone(),
            })
        };
        donation_ids_by_recipient_set.insert(&donation.id);
        self.donation_ids_by_recipient_id
            .insert(&donation.recipient_id, &donation_ids_by_recipient_set);

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
        donation_ids_by_donor_set.insert(&donation.id);
        self.donation_ids_by_donor_id
            .insert(&donation.donor_id, &donation_ids_by_donor_set);

        // add to donations-by-ft mapping
        let mut donation_ids_by_ft_set =
            if let Some(donation_ids_by_ft_set) = self.donation_ids_by_ft_id.get(&donation.ft_id) {
                donation_ids_by_ft_set
            } else {
                UnorderedSet::new(StorageKey::DonationIdsByFtIdInner {
                    ft_id: donation.ft_id.clone(),
                })
            };
        donation_ids_by_ft_set.insert(&donation.id);
        self.donation_ids_by_ft_id
            .insert(&donation.ft_id, &donation_ids_by_ft_set);
    }

    // GETTERS
    // get_donations
    // get_matching_pool_balance
    pub fn get_donations(&self, from_index: Option<u128>, limit: Option<u64>) -> Vec<Donation> {
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
            .map(|(_, v)| Donation::from(v))
            .collect()
    }

    pub fn get_donation_by_id(&self, donation_id: DonationId) -> Option<Donation> {
        self.donations_by_id
            .get(&donation_id)
            .map(|v| Donation::from(v))
    }

    pub fn get_donations_for_recipient(
        &self,
        recipient_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation> {
        let start_index: u128 = from_index.unwrap_or_default();
        // TODO: ADD BELOW BACK IN
        // assert!(
        //     (self.donations_by_id.len() as u128) >= start_index,
        //     "Out of bounds, please use a smaller from_index."
        // );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_recipient_set = self.donation_ids_by_recipient_id.get(&recipient_id);
        log!("got set"); // TODO: REMOVE
        if let Some(donation_ids_by_recipient_set) = donation_ids_by_recipient_set {
            donation_ids_by_recipient_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|donation_id| Donation::from(self.donations_by_id.get(&donation_id).unwrap()))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation> {
        let start_index: u128 = from_index.unwrap_or_default();
        // TODO: ADD BELOW BACK IN
        // assert!(
        //     (self.donations_by_id.len() as u128) >= start_index,
        //     "Out of bounds, please use a smaller from_index."
        // );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_donor_set = self.donation_ids_by_donor_id.get(&donor_id);
        log!("got set"); // TODO: REMOVE
        if let Some(donation_ids_by_donor_set) = donation_ids_by_donor_set {
            donation_ids_by_donor_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|donation_id| Donation::from(self.donations_by_id.get(&donation_id).unwrap()))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn get_donations_for_ft(
        &self,
        ft_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation> {
        let start_index: u128 = from_index.unwrap_or_default();
        // TODO: ADD BELOW BACK IN
        // assert!(
        //     (self.donations_by_id.len() as u128) >= start_index,
        //     "Out of bounds, please use a smaller from_index."
        // );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_ft_set = self.donation_ids_by_ft_id.get(&ft_id);
        log!("got set"); // TODO: REMOVE
        if let Some(donation_ids_by_ft_set) = donation_ids_by_ft_set {
            donation_ids_by_ft_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|donation_id| Donation::from(self.donations_by_id.get(&donation_id).unwrap()))
                .collect()
        } else {
            vec![]
        }
    }
}
