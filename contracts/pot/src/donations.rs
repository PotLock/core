use crate::*;
/// Could be an end-user donation (must include a project_id in this case) or a matching pool donation (may include a referrer_id in this case)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub amount: U128,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at: TimestampMs,
    /// ID of the project receiving the donation        
    pub project_id: ProjectId, // TODO: change to application_id
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
}

pub const DONATION_ID_DELIMETER: &str = ":";

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_donation_requirement(&mut self, donation_requirement: Option<SBTRequirement>) {
        self.assert_chef();
        self.donation_requirement = donation_requirement;
    }

    #[payable]
    pub fn donate(&mut self, project_id: ProjectId, message: Option<String>) -> Promise {
        // TODO: add referrer_id
        self.assert_caller_can_donate(project_id, message)
    }

    pub(crate) fn assert_caller_can_donate(
        &mut self,
        project_id: ProjectId,
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
                    .assert_can_donate_callback(project_id, message),
            )
        } else {
            // no donation requirement. always allow
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(XXC_GAS))
                .always_allow_callback(project_id, message)
        }
    }

    pub(crate) fn handle_donation(
        &mut self,
        project_id: ProjectId,
        message: Option<String>,
    ) -> Donation {
        let donation_count_for_project = if let Some(donation_ids_by_project_set) =
            self.donation_ids_by_project_id.get(&project_id)
        {
            donation_ids_by_project_set.len()
        } else {
            0
        };
        let donation = Donation {
            id: donation_count_for_project + 1 as DonationId,
            donor_id: env::predecessor_account_id(),
            amount: U128::from(env::attached_deposit()),
            message,
            donated_at: env::block_timestamp(),
            project_id,
            referrer_id: None,
        };
        self.insert_donation_record(&donation);
        // TODO: TAKE OUT PROTOCOL FEE & ANY OTHER FEES
        donation
        // Promise::new()
    }

    pub(crate) fn insert_donation_record(&mut self, donation: &Donation) {
        self.donations_by_id.insert(&donation.id, &donation);
        self.add_donation_for_project(&donation);
        self.add_donation_for_donor(&donation);
    }

    pub(crate) fn add_donation_for_project(&mut self, donation: &Donation) {
        let mut donation_ids_by_project_set = if let Some(donation_ids_by_project_set) =
            self.donation_ids_by_project_id.get(&donation.project_id)
        {
            donation_ids_by_project_set
        } else {
            UnorderedSet::new(StorageKey::DonationIdsByProjectIdInner {
                project_id: donation.project_id,
            })
        };
        donation_ids_by_project_set.insert(&donation.id);
        self.donation_ids_by_project_id
            .insert(&donation.project_id, &donation_ids_by_project_set);
    }

    pub(crate) fn add_donation_for_donor(&mut self, donation: &Donation) {
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
        self.donation_ids_by_project_id
            .insert(&donation.project_id, &donation_ids_by_donor_set);
    }

    // CALLBACKS

    #[private]
    pub fn always_allow_callback(
        &mut self,
        project_id: ProjectId,
        message: Option<String>,
    ) -> Donation {
        self.handle_donation(project_id, message)
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn assert_can_donate_callback(
        &mut self,
        project_id: ProjectId,
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
            self.handle_donation(project_id, message)
        } else {
            env::panic_str("You don't have the required SBTs in order to donate.");
            // TODO: add details of required SBTs to error string
        }
    }
}
