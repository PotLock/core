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

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FtReceiverMsg {
    pub recipient_id: AccountId,
    pub referrer_id: Option<AccountId>,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum TransferType {
    DonationTransfer,
    ProtocolFeeTransfer,
    ReferrerFeeTransfer,
}

#[near_bindgen]
impl Contract {
    /// FT equivalent of donate, for use with FTs that implement NEP-144
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let ft_id = env::predecessor_account_id();
        let msg_json: FtReceiverMsg = near_sdk::serde_json::from_str(&msg)
            .expect("Invalid msg string. Must implement FtReceiverMsg.");
        log!(format!(
            "Recipient ID {:?}, Referrer ID {:?}, Amount {}, Message {:?}",
            msg_json.recipient_id, msg_json.referrer_id, amount.0, msg_json.message
        ));

        // calculate amounts
        let (protocol_fee, referrer_fee, remainder) =
            self.calculate_fees_and_remainder(amount.0, msg_json.referrer_id.clone());

        // create and insert donation record
        let initial_storage_usage = env::storage_usage();
        let donation = self.create_and_insert_donation_record(
            sender_id.clone(),
            amount,
            ft_id.clone(),
            msg_json.message.clone(),
            env::block_timestamp_ms(),
            msg_json.recipient_id.clone(),
            U128::from(protocol_fee),
            msg_json.referrer_id.clone(),
            referrer_fee,
        );

        // verify and update storage balance for FT donation
        self.verify_and_update_storage_balance(sender_id.clone(), initial_storage_usage);

        // transfer donation
        log!(format!(
            "Transferring donation {} ({}) to {}",
            remainder, ft_id, msg_json.recipient_id
        ));
        self.handle_transfer_donation(
            msg_json.recipient_id.clone(),
            U128(remainder),
            remainder,
            donation.clone(),
        );

        // NB: fees will be transferred in transfer_funds_callback after successful transfer of donation

        // return # unused tokens as per NEP-144 standard
        PromiseOrValue::Value(U128(0))
    }

    #[payable]
    pub fn donate(
        &mut self,
        recipient_id: AccountId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
    ) -> Donation {
        // calculate amounts
        let amount = env::attached_deposit();
        let (protocol_fee, referrer_fee, mut remainder) =
            self.calculate_fees_and_remainder(amount.clone(), referrer_id.clone());

        // create and insert donation record
        let initial_storage_usage = env::storage_usage();
        let donation = self.create_and_insert_donation_record(
            env::predecessor_account_id(),
            U128::from(amount),
            AccountId::new_unchecked("near".to_string()),
            message,
            env::block_timestamp_ms(),
            recipient_id.clone(),
            U128::from(protocol_fee),
            referrer_id.clone(),
            referrer_fee,
        );

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

        // transfer donation
        log!(format!(
            "Transferring donation {} to {}",
            remainder, recipient_id
        ));
        self.handle_transfer_donation(
            recipient_id.clone(),
            U128(remainder),
            remainder,
            donation.clone(),
        );

        // NB: fees will be transferred in transfer_funds_callback after successful transfer of donation

        // return donation
        donation
    }

    pub(crate) fn calculate_fees_and_remainder(
        &self,
        amount: u128,
        referrer_id: Option<AccountId>,
    ) -> (u128, Option<U128>, u128) {
        // calculate protocol fee
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

        (protocol_fee, referrer_fee, remainder)
    }

    pub(crate) fn create_and_insert_donation_record(
        &mut self,
        donor_id: AccountId,
        total_amount: U128,
        ft_id: AccountId,
        message: Option<String>,
        donated_at_ms: TimestampMs,
        recipient_id: AccountId,
        protocol_fee: U128,
        referrer_id: Option<AccountId>,
        referrer_fee: Option<U128>,
    ) -> Donation {
        let donation = Donation {
            id: self.next_donation_id,
            donor_id,
            total_amount,
            ft_id,
            message,
            donated_at_ms,
            recipient_id,
            protocol_fee,
            referrer_id,
            referrer_fee,
        };

        // increment next_donation_id
        self.next_donation_id += 1;

        // insert mapping records
        self.insert_donation_record_internal(&donation);

        donation
    }

    pub(crate) fn verify_and_update_storage_balance(
        &mut self,
        sender_id: AccountId,
        initial_storage_usage: u64,
    ) {
        // verify that deposit is sufficient to cover storage
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        let storage_balance = self.storage_balance_of(&sender_id);
        assert!(
            storage_balance.0 >= required_deposit,
            "{} must add storage deposit of at least {} yoctoNEAR to cover Donation storage",
            sender_id,
            required_deposit
        );

        log!("Old storage balance: {}", storage_balance.0);
        // deduct storage deposit from user's balance
        let new_storage_balance = storage_balance.0 - required_deposit;
        self.storage_deposits
            .insert(&sender_id, &new_storage_balance);
        log!("New storage balance: {}", new_storage_balance);
        log!(format!(
            "Deducted {} yoctoNEAR from {}'s storage balance to cover storage",
            required_deposit, sender_id
        ));
    }

    pub(crate) fn handle_transfer(
        &self,
        recipient_id: AccountId,
        amount: U128,
        remainder: Balance,
        donation: Donation,
        transfer_type: TransferType,
    ) {
        if donation.ft_id == AccountId::new_unchecked("near".to_string()) {
            Promise::new(recipient_id).transfer(amount.0).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                    .transfer_funds_callback(remainder, donation.clone(), transfer_type),
            );
        } else {
            let ft_transfer_args = json!({ "receiver_id": recipient_id, "amount": amount })
                .to_string()
                .into_bytes();
            Promise::new(donation.ft_id.clone())
                .function_call(
                    "ft_transfer".to_string(),
                    ft_transfer_args,
                    ONE_YOCTO,
                    Gas(XCC_GAS_DEFAULT),
                )
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas(XCC_GAS_DEFAULT))
                        .transfer_funds_callback(remainder, donation.clone(), transfer_type),
                );
        }
    }

    pub(crate) fn handle_transfer_donation(
        &self,
        recipient_id: AccountId,
        amount: U128,
        remainder: Balance,
        donation: Donation,
    ) {
        self.handle_transfer(
            recipient_id,
            amount,
            remainder,
            donation,
            TransferType::DonationTransfer,
        );
    }

    pub(crate) fn handle_transfer_protocol_fee(
        &self,
        recipient_id: AccountId,
        amount: U128,
        remainder: Balance,
        donation: Donation,
    ) {
        self.handle_transfer(
            recipient_id,
            amount,
            remainder,
            donation,
            TransferType::ProtocolFeeTransfer,
        );
    }

    pub(crate) fn handle_transfer_referrer_fee(
        &self,
        recipient_id: AccountId,
        amount: U128,
        remainder: Balance,
        donation: Donation,
    ) {
        self.handle_transfer(
            recipient_id,
            amount,
            remainder,
            donation,
            TransferType::ReferrerFeeTransfer,
        );
    }

    /// Verifies whether donation & fees have been paid out for a given donation
    #[private]
    pub fn transfer_funds_callback(
        &mut self,
        remainder: Balance,
        mut donation: Donation,
        transfer_type: TransferType,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        let is_ft_transfer = donation.ft_id != AccountId::new_unchecked("near".to_string());
        if call_result.is_err() {
            // ERROR CASE HANDLING
            // 1. If donation transfer failed, delete Donation record and return all funds to donor. NB: fees have not been transferred yet.
            // 2. If protocol fee transfer failed, update donation record to indicate protocol fee of "0". NB: donation has already been transferred to recipient and this cannot be reversed.
            // 3. If referrer fee transfer failed, update donation record to indicate referrer fee of "0". NB: donation has already been transferred to recipient and this cannot be reversed.
            match transfer_type {
                TransferType::DonationTransfer => {
                    log!(format!(
                        "Error transferring donation {:?} to {}. Returning funds to donor.",
                        donation.total_amount, donation.recipient_id
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.total_amount.clone() })
                                .to_string()
                                .into_bytes();
                        Promise::new(AccountId::new_unchecked(donation.ft_id.to_string()))
                            .function_call(
                                "ft_transfer".to_string(),
                                donation_transfer_args,
                                ONE_YOCTO,
                                Gas(XCC_GAS_DEFAULT),
                            );
                    } else {
                        Promise::new(donation.donor_id.clone()).transfer(donation.total_amount.0);
                    }
                    // delete donation record, and refund freed storage cost to donor's storage balance
                    let initial_storage_usage = env::storage_usage();
                    self.remove_donation_record_internal(&donation);
                    let storage_freed = initial_storage_usage - env::storage_usage();
                    let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
                    let storage_balance = self.storage_balance_of(&donation.donor_id);
                    let new_storage_balance = storage_balance.0 + cost_freed;
                    log!("Old storage balance: {}", storage_balance.0);
                    log!("New storage balance: {}", new_storage_balance);
                    self.storage_deposits
                        .insert(&donation.donor_id, &new_storage_balance); // TODO: check if this is hackable, e.g. if user can withdraw all their storage before this callback runs and therefore get a higher refund
                    log!(format!(
                        "Refunded {} yoctoNEAR to {}'s storage balance for freed storage",
                        cost_freed, donation.donor_id
                    ));
                }
                TransferType::ProtocolFeeTransfer => {
                    log!(format!(
                        "Error transferring protocol fee {:?} to {}. Returning funds to donor.",
                        donation.protocol_fee, self.protocol_fee_recipient_account
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.protocol_fee })
                                .to_string()
                                .into_bytes();
                        Promise::new(AccountId::new_unchecked(donation.ft_id.to_string()))
                            .function_call(
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
                }
                TransferType::ReferrerFeeTransfer => {
                    log!(format!(
                        "Error transferring referrer fee {:?} to {:?}. Returning funds to donor.",
                        donation.referrer_fee, donation.referrer_id
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.referrer_fee })
                                .to_string()
                                .into_bytes();
                        Promise::new(AccountId::new_unchecked(donation.ft_id.to_string()))
                            .function_call(
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
            }
        } else {
            // SUCCESS CASE HANDLING
            if transfer_type == TransferType::DonationTransfer {
                log!(format!(
                    "Successfully transferred donation {} to {}!",
                    remainder, donation.recipient_id
                ));

                // transfer protocol fee
                log!(format!(
                    "Transferring protocol fee {:?} ({}) to {}",
                    donation.protocol_fee, donation.ft_id, self.protocol_fee_recipient_account
                ));
                self.handle_transfer_protocol_fee(
                    self.protocol_fee_recipient_account.clone(),
                    donation.protocol_fee.clone(),
                    remainder,
                    donation.clone(),
                );

                // transfer referrer fee
                if let (Some(referrer_fee), Some(referrer_id)) =
                    (donation.referrer_fee.clone(), donation.referrer_id.clone())
                {
                    log!(format!(
                        "Transferring referrer fee {:?} ({}) to {}",
                        referrer_fee.clone(),
                        donation.ft_id,
                        referrer_id
                    ));
                    self.handle_transfer_referrer_fee(
                        referrer_id.clone(),
                        referrer_fee.clone(),
                        remainder,
                        donation.clone(),
                    );
                }

                // log event indicating successful donation/transfer!
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

    pub(crate) fn insert_donation_record_internal(&mut self, donation: &Donation) {
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

    pub(crate) fn remove_donation_record_internal(&mut self, donation: &Donation) {
        self.donations_by_id.remove(&donation.id);
        // remove from donations-by-recipient mapping
        let mut donation_ids_by_recipient_set = self
            .donation_ids_by_recipient_id
            .get(&donation.recipient_id)
            .unwrap();
        donation_ids_by_recipient_set.remove(&donation.id);
        self.donation_ids_by_recipient_id
            .insert(&donation.recipient_id, &donation_ids_by_recipient_set);

        // remove from donations-by-donor mapping
        let mut donation_ids_by_donor_set = self
            .donation_ids_by_donor_id
            .get(&donation.donor_id)
            .unwrap();
        donation_ids_by_donor_set.remove(&donation.id);
        self.donation_ids_by_donor_id
            .insert(&donation.donor_id, &donation_ids_by_donor_set);

        // remove from donations-by-ft mapping
        let mut donation_ids_by_ft_set = self.donation_ids_by_ft_id.get(&donation.ft_id).unwrap();
        donation_ids_by_ft_set.remove(&donation.id);
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
