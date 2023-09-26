use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
    InReview,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Application {
    /// Unique identifier for the application, increments from 1
    pub id: ApplicationId,
    pub project_id: ProjectId,
    // /// ID of the individual or group that submitted the application. TODO: MUST be on the Potlock Registry (registry.potluck.[NETWORK])
    // pub creator_id: AccountId,
    // /// Name of the individual or group that submitted the application
    // pub creator_name: Option<String>, // TODO: consider whether this should be required (currently optional)
    /// Account ID that should receive payout funds  
    // pub payout_to: AccountId, // TODO: consider whether this should exist here or on the registry contract, or on nearhorizons contract
    /// Status of the project application (Pending, Accepted, Rejected, InReview)
    pub status: ApplicationStatus,
    /// Timestamp for when the application was submitted
    pub submitted_at: TimestampMs,
    // /// Timestamp for when the application was reviewed (if applicable)
    // pub reviewed_at: Option<TimestampMs>,
    /// Timestamp for when the project was updated
    // TODO: should only be updateable before it is approved
    pub updated_at: Option<TimestampMs>,
    /// Notes to be added by Chef when reviewing the application
    pub review_notes: Option<String>,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn apply(&mut self) -> Promise {
        let project_id = env::predecessor_account_id();
        let promise = potlock_registry::ext(self.registry_contract_id.clone())
            .with_static_gas(Gas(XXC_GAS))
            .get_project_by_id(project_id.clone());

        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(XXC_GAS))
                .assert_can_apply_callback(project_id.clone()),
        )
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn assert_can_apply_callback(
        &mut self,
        project_id: ProjectId,
        #[callback_result] call_result: Result<PotlockRegistryProject, PromiseError>,
    ) -> Application {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
            env::panic_str(&format!(
                "Project is not registered on {}",
                self.registry_contract_id.clone()
            ));
        }
        // check that application doesn't already exist for this project
        if self.application_id_by_project_id.get(&project_id).is_some() {
            // application already exists
            env::panic_str("Application already exists for this project");
        }
        // check that application period is open
        self.assert_application_period_open();
        // check that max_projects hasn't been reached
        self.assert_max_projects_not_reached();
        // add application
        let application = Application {
            id: self.applications_by_id.len() + 1 as ApplicationId,
            project_id,
            status: ApplicationStatus::Pending,
            submitted_at: env::block_timestamp_ms(),
            updated_at: None,
            review_notes: None,
        };
        // update mappings
        self.applications_by_id
            .insert(&application.id, &application);
        self.application_id_by_project_id
            .insert(&application.project_id, &application.id);
        self.application_ids.insert(&application.id);
        // return application
        application
    }

    pub fn unapply(&mut self) {
        let application_id = self
            .application_id_by_project_id
            .get(&env::predecessor_account_id())
            .expect("Application does not exist for calling project");
        let application = self
            .applications_by_id
            .get(&application_id)
            .expect("Application does not exist");
        // verify that application is pending
        assert_eq!(
            application.status,
            ApplicationStatus::Pending,
            "Application status is {:?}. Only pending applications can be removed",
            application.status
        );
        // remove from mappings
        self.application_ids.remove(&application_id);
        self.applications_by_id.remove(&application_id);
        self.application_id_by_project_id
            .remove(&application.project_id);
        // TODO: emit event?
    }

    pub fn get_applications(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Application> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.application_ids.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.application_ids
            .iter()
            .skip(start_index as usize)
            .take(limit.try_into().unwrap())
            .map(|application_id| {
                self.applications_by_id
                    .get(&application_id)
                    .expect("Application does not exist")
            })
            .collect()
    }

    pub fn get_application_by_id(&self, application_id: ApplicationId) -> Application {
        self.applications_by_id
            .get(&application_id)
            .expect("Application does not exist")
    }

    pub fn chef_set_application_status(
        &mut self,
        application_id: ApplicationId,
        status: ApplicationStatus,
        notes: String,
    ) -> Application {
        self.assert_chef();
        // verify that the application exists
        let mut application = self
            .applications_by_id
            .get(&application_id)
            .expect("Application does not exist");
        // verify that the application is pending
        application.status = status;
        application.updated_at = Some(env::block_timestamp_ms());
        application.review_notes = Some(notes);
        // update mapping
        self.applications_by_id
            .insert(&application_id, &application);
        application
    }

    pub fn chef_mark_application_approved(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(application_id, ApplicationStatus::Approved, notes)
    }

    pub fn chef_mark_application_rejected(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(application_id, ApplicationStatus::Rejected, notes)
    }

    pub fn chef_mark_application_in_review(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(application_id, ApplicationStatus::InReview, notes)
    }

    pub fn chef_mark_application_pending(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(application_id, ApplicationStatus::Pending, notes)
    }
}
