use crate::*;
/// End-user donation
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub amount: u128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation        
    pub application_id: ApplicationId,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
}

/// Matching pool / patron donation
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PatronDonation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub amount: u128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
}

pub const DONATION_ID_DELIMETER: &str = ":";

#[near_bindgen]
impl Contract {
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
            .map(|(_, v)| v)
            .collect()
    }

    pub fn get_donations_for_application(
        &self,
        application_id: ApplicationId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_application_set = self
            .donation_ids_by_application_id
            .get(&application_id)
            .unwrap();
        donation_ids_by_application_set
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|donation_id| self.donations_by_id.get(&donation_id).unwrap())
            .collect()
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<Donation> {
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
            .map(|donation_id| self.donations_by_id.get(&donation_id).unwrap())
            .collect()
    }

    pub fn get_matching_pool_balance(&self) -> U128 {
        self.matching_pool_balance
    }

    pub fn get_donations_balance(&self) -> U128 {
        self.donations_balance
    }

    #[payable]
    pub fn set_donation_requirement(&mut self, donation_requirement: Option<SBTRequirement>) {
        self.assert_chef();
        self.donation_requirement = donation_requirement;
    }

    #[payable]
    pub fn donate(&mut self, application_id: ApplicationId, message: Option<String>) -> Promise {
        // TODO: add referrer_id
        self.assert_caller_can_donate(application_id, message)
    }

    /// Adds attached deposit to matching pool, adds mappings & returns PatronDonation
    #[payable]
    pub fn patron_donate_to_matching_pool(&mut self, message: Option<String>) -> PatronDonation {
        let deposit = env::attached_deposit();
        self.matching_pool_balance = U128::from(
            self.matching_pool_balance
                .0
                .checked_add(deposit)
                .expect(&format!(
                    "Overflow occurred when calculating self.matching_pool_balance ({} + {})",
                    self.matching_pool_balance.0, deposit,
                )),
        );
        let patron_donation_count = self.patron_donation_ids.len();
        let patron_donation = PatronDonation {
            id: patron_donation_count + 1 as DonationId,
            donor_id: env::predecessor_account_id(),
            amount: deposit,
            message,
            donated_at: env::block_timestamp(),
            referrer_id: None, // TODO: handle referrer
        };
        self.patron_donations_by_id
            .insert(&patron_donation.id, &patron_donation);
        self.patron_donation_ids.insert(&patron_donation.id);
        patron_donation
    }

    pub(crate) fn assert_caller_can_donate(
        &mut self,
        application_id: ApplicationId,
        message: Option<String>,
    ) -> Promise {
        // TODO: verify that the project exists & donation window is open
        if let Some(donation_requirement) = &self.donation_requirement {
            let promise = sbt_registry::ext(donation_requirement.registry_id.clone())
                .with_static_gas(Gas(XXC_GAS))
                .sbt_tokens_by_owner(
                    env::predecessor_account_id(),
                    Some(donation_requirement.issuer_id.clone()),
                    Some(donation_requirement.class_id),
                );

            promise.then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(XXC_GAS))
                    .assert_can_donate_callback(application_id, message),
            )
        } else {
            // no donation requirement. always allow
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(XXC_GAS))
                .always_allow_callback(application_id, message)
        }
    }

    pub(crate) fn handle_donation(
        &mut self,
        application_id: ApplicationId,
        message: Option<String>,
    ) -> Donation {
        let donation_count_for_project = if let Some(donation_ids_by_project_set) =
            self.donation_ids_by_application_id.get(&application_id)
        {
            donation_ids_by_project_set.len()
        } else {
            0
        };
        let amount = env::attached_deposit();
        let donation = Donation {
            id: donation_count_for_project + 1 as DonationId,
            donor_id: env::predecessor_account_id(),
            amount,
            message,
            donated_at: env::block_timestamp(),
            application_id,
            referrer_id: None,
        };
        self.insert_donation_record(&donation);
        self.donations_balance = U128::from(self.donations_balance.0.checked_add(amount).expect(
            &format!(
                "Overflow occurred when calculating self.donations_balance ({} + {})",
                self.donations_balance.0, amount,
            ),
        ));
        // TODO: TAKE OUT PROTOCOL FEE & ANY OTHER FEES
        donation
        // Promise::new()
    }

    pub(crate) fn insert_donation_record(&mut self, donation: &Donation) {
        self.donations_by_id.insert(&donation.id, &donation);
        // add to donations-by-application mapping
        let mut donation_ids_by_application_set = if let Some(donation_ids_by_application_set) =
            self.donation_ids_by_application_id
                .get(&donation.application_id)
        {
            donation_ids_by_application_set
        } else {
            UnorderedSet::new(StorageKey::DonationIdsByApplicationIdInner {
                application_id: donation.application_id.clone(),
            })
        };
        donation_ids_by_application_set.insert(&donation.id);
        self.donation_ids_by_application_id
            .insert(&donation.application_id, &donation_ids_by_application_set);
        // add to donations-by-donor mapping
        // self.add_donation_for_donor(&donation);
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
    }

    // CALLBACKS

    #[private]
    pub fn always_allow_callback(
        &mut self,
        application_id: ApplicationId,
        message: Option<String>,
    ) -> Donation {
        self.handle_donation(application_id, message)
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn assert_can_donate_callback(
        &mut self,
        application_id: ApplicationId,
        message: Option<String>,
        #[callback_result] call_result: Result<SbtTokensByOwnerResult, PromiseError>,
    ) -> Donation {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
            env::panic_str("There was an error querying SBTs");
        }
        let tokens: Vec<(AccountId, Vec<OwnedToken>)> = call_result.unwrap();
        if tokens.len() > 0 {
            // user holds the required SBT(s)
            self.handle_donation(application_id, message)
        } else {
            env::panic_str("You don't have the required SBTs in order to donate.");
            // TODO: add details of required SBTs to error string
        }
    }
}
