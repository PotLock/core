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

            // get donation count, which will be incremented to create the unique donation ID
            let donation_count = self.donations_by_id.len();

            // format donation record
            let donation = Donation {
                id: (donation_count + 1) as DonationId,
                donor_id: env::predecessor_account_id(),
                total_amount: amount,
                ft_id: ft_id.clone(),
                message,
                donated_at_ms: env::block_timestamp_ms(),
                recipient_id: recipient_id.clone(),
                protocol_fee: U128::from(protocol_fee),
                referrer_id: referrer_id.clone(),
                referrer_fee,
            };

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
            // TODO: validate that this further transfer can be made potentially in the same block where ft_resolve_transfer is called on FT contract. This account may not have enough balance to cover the transfer at that point.
            log!(format!(
                "Transferring protocol fee {} ({}) to {}",
                protocol_fee, ft_id, self.protocol_fee_recipient_account
            ));
            let protocol_fee_transfer_args: Vec<u8> = json!({ "receiver_id": self.protocol_fee_recipient_account.clone(), "amount": U128(protocol_fee) })
                .to_string()
                .into_bytes();
            Promise::new(AccountId::new_unchecked(ft_id.to_string())).function_call(
                "ft_transfer".to_string(),
                protocol_fee_transfer_args,
                ONE_YOCTO,
                Gas(XCC_GAS_DEFAULT),
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
                Promise::new(AccountId::new_unchecked(ft_id.to_string())).function_call(
                    "ft_transfer".to_string(),
                    referrer_fee_transfer_args,
                    ONE_YOCTO,
                    Gas(XCC_GAS_DEFAULT),
                );
            }

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

        // get donation count, which will be incremented to create the unique donation ID
        let donation_count = self.donations_by_id.len();

        // format donation record
        let donation = Donation {
            id: (donation_count + 1) as DonationId,
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
        Promise::new(self.protocol_fee_recipient_account.clone()).transfer(protocol_fee);

        // transfer referrer fee
        if let (Some(referrer_fee), Some(referrer_id)) = (referrer_fee, referrer_id) {
            log!(format!(
                "Transferring referrer fee {} to {}",
                referrer_fee.0, referrer_id
            ));
            Promise::new(referrer_id).transfer(referrer_fee.0);
        }

        // transfer donation
        log!(format!(
            "Transferring donation {} to {}",
            remainder, recipient_id
        ));
        Promise::new(recipient_id).transfer(remainder);

        // log event
        log_donation_event(&donation);

        // return donation
        donation
    }

    pub(crate) fn calculate_protocol_fee(&self, amount: u128) -> u128 {
        let total_basis_points = 10_000u128;
        let amount_per_basis_point = amount / total_basis_points;
        self.protocol_fee_basis_points as u128 * amount_per_basis_point
    }

    pub(crate) fn calculate_referrer_fee(&self, amount: u128) -> u128 {
        let total_basis_points = 10_000u128;
        let amount_per_basis_point = amount / total_basis_points;
        self.referral_fee_basis_points as u128 * amount_per_basis_point
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
