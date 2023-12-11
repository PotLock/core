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
    // functions as unique identifier for application, since projects can only apply once per round
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
        let project_id = env::predecessor_account_id(); // TODO: consider renaming to "applicant_id" to make it less opinionated
        if let Some(registry_provider) = self.registry_provider.get() {
            // decompose registry provider
            let (contract_id, method_name) = registry_provider.decompose();
            // call registry provider
            let args = json!({ "account_id": project_id }).to_string().into_bytes();
            Promise::new(AccountId::new_unchecked(contract_id.clone()))
                .function_call(method_name.clone(), args, 0, XCC_GAS)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(XCC_GAS)
                        .assert_can_apply_callback(project_id.clone()),
                )
        } else {
            Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .apply_always_allow_callback(project_id.clone())
        }
    }

    #[private] // Only callable by env::current_account_id()
    pub fn apply_always_allow_callback(&mut self, project_id: ProjectId) -> Application {
        self.handle_apply(project_id)
    }

    #[private] // Only callable by env::current_account_id()
    pub fn assert_can_apply_callback(
        &mut self,
        project_id: ProjectId,
        #[callback_result] call_result: Result<bool, PromiseError>,
    ) -> Application {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() || !call_result.unwrap() {
            env::panic_str(&format!(
                "Project is not registered on {:#?}",
                self.registry_provider.get().unwrap()
            ));
        }
        self.handle_apply(project_id)
    }

    pub(crate) fn handle_apply(&mut self, project_id: ProjectId) -> Application {
        // check that application doesn't already exist for this project
        if self.applications_by_project_id.get(&project_id).is_some() {
            // application already exists
            env::panic_str("Application already exists for this project");
        }
        // check that application period is open
        self.assert_application_period_open();
        // check that max_projects hasn't been reached
        self.assert_max_projects_not_reached();
        // add application
        let application = Application {
            project_id,
            status: ApplicationStatus::Pending,
            submitted_at: env::block_timestamp_ms(),
            updated_at: None,
            review_notes: None,
        };
        // update mappings
        self.applications_by_project_id
            .insert(&application.project_id, &application);
        // return application
        application
    }

    pub fn unapply(&mut self) {
        let project_id = env::predecessor_account_id();
        let application = self
            .applications_by_project_id
            .get(&project_id)
            .expect("Application does not exist for calling project");
        // verify that application is pending
        // TODO: consider removing this check
        assert_eq!(
            application.status,
            ApplicationStatus::Pending,
            "Application status is {:?}. Only pending applications can be removed",
            application.status
        );
        // remove from mappings
        self.applications_by_project_id.remove(&project_id);
        // TODO: emit event?
    }

    pub fn get_applications(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Application> {
        let start_index: u128 = from_index.map(From::from).unwrap_or_default();
        assert!(
            (self.applications_by_project_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.applications_by_project_id
            .iter()
            .skip(start_index as usize)
            .take(limit.try_into().unwrap())
            .map(|(_account_id, application)| application)
            .collect()
    }

    pub fn get_application_by_project_id(&self, project_id: ProjectId) -> Application {
        self.applications_by_project_id
            .get(&project_id)
            .expect("Application does not exist")
    }

    pub fn chef_set_application_status(
        &mut self,
        project_id: ProjectId,
        status: ApplicationStatus,
        notes: String,
    ) -> Application {
        self.assert_chef_or_greater();
        // verify that the application exists
        let mut application = self
            .applications_by_project_id
            .get(&project_id)
            .expect("Application does not exist");
        // verify that the application is pending
        application.status = status;
        application.updated_at = Some(env::block_timestamp_ms());
        application.review_notes = Some(notes);
        // update mapping
        self.applications_by_project_id
            .insert(&project_id, &application);
        application
    }

    // TODO: consider removing convenience methods below

    pub fn chef_mark_application_approved(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Approved, notes)
    }

    pub fn chef_mark_application_rejected(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Rejected, notes)
    }

    pub fn chef_mark_application_in_review(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::InReview, notes)
    }

    pub fn chef_mark_application_pending(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Pending, notes)
    }
}
