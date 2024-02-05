use borsh::de;

use crate::*;

pub type ProjectId = AccountId;
pub type ApplicationId = ProjectId; // Applications are indexed by ProjectId

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
    InReview,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Application {
    /// functions as unique identifier for application, since projects can only apply once per round
    // Don't technically need this, since we use the project_id as the key in the applications_by_id mapping, but it's possible that we'll want to change that in the future, so keeping this for now
    pub project_id: ProjectId,
    /// Optional message to be included in application
    pub message: Option<String>,
    /// Status of the project application (Pending, Accepted, Rejected, InReview)
    pub status: ApplicationStatus,
    /// Timestamp for when the application was submitted
    pub submitted_at: TimestampMs,
    /// Timestamp for when the application was last updated (e.g. status changed)
    pub updated_at: Option<TimestampMs>,
    /// Notes to be added by Chef when reviewing the application
    pub review_notes: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedApplication {
    Current(Application),
}

// converts VersionedApplication to Application
impl From<VersionedApplication> for Application {
    fn from(application: VersionedApplication) -> Self {
        match application {
            VersionedApplication::Current(current) => current,
        }
    }
}

// converts &VersionedApplication to Application
impl From<&VersionedApplication> for Application {
    fn from(application: &VersionedApplication) -> Self {
        match application {
            VersionedApplication::Current(current) => current.to_owned(),
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn apply(&mut self, message: Option<String>) -> Promise {
        let project_id = env::predecessor_account_id(); // TODO: consider renaming to "applicant_id" to make it less opinionated (e.g. maybe developers are applying, and they are not exactly a "project")
                                                        // chef, admin & owner cannot apply
        assert!(
            !self.is_chef(Some(&project_id)) && !self.is_owner_or_admin(Some(&project_id)),
            "Chef, admin & owner cannot apply"
        );
        let deposit = env::attached_deposit();
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
                        .assert_can_apply_callback(project_id.clone(), message, deposit),
                )
        } else {
            Self::ext(env::current_account_id())
                .with_static_gas(XCC_GAS)
                .handle_apply(project_id.clone(), message, deposit)
        }
    }

    #[private] // Only callable by env::current_account_id()
    pub fn assert_can_apply_callback(
        &mut self,
        project_id: ProjectId,
        message: Option<String>,
        deposit: Balance,
        #[callback_result] call_result: Result<bool, PromiseError>,
    ) -> Application {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() || !call_result.unwrap() {
            env::panic_str(&format!(
                "Project is not registered on {:#?}",
                self.registry_provider.get().unwrap()
            ));
        }
        self.handle_apply(project_id, message, deposit)
    }

    #[private]
    pub fn handle_apply(
        &mut self,
        project_id: ProjectId,
        message: Option<String>,
        deposit: Balance,
    ) -> Application {
        // check that application doesn't already exist for this project
        if self.applications_by_id.get(&project_id).is_some() {
            // application already exists
            env::panic_str("Application already exists for this project");
        }
        // check that application period is open
        self.assert_application_period_open();
        // add application
        let application = Application {
            project_id,
            message,
            status: ApplicationStatus::Pending,
            submitted_at: env::block_timestamp_ms(),
            updated_at: None,
            review_notes: None,
        };
        // charge for storage
        let initial_storage_usage = env::storage_usage();
        // update mappings
        self.applications_by_id.insert(
            &application.project_id,
            &VersionedApplication::Current(application.clone()),
        );
        // refund excess deposit
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        if deposit > required_deposit {
            Promise::new(env::predecessor_account_id()).transfer(deposit - required_deposit);
        } else if deposit < required_deposit {
            env::panic_str(&format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            ));
        }

        // return application
        application
    }

    pub fn unapply(&mut self) {
        let project_id = env::predecessor_account_id();
        let application = Application::from(
            self.applications_by_id
                .get(&project_id)
                .expect("Application does not exist for calling project"),
        );
        // verify that application is pending
        // TODO: consider whether this check is necessary
        assert_eq!(
            application.status,
            ApplicationStatus::Pending,
            "Application status is {:?}. Only pending applications can be removed",
            application.status
        );
        // get current storage usage
        let initial_storage_usage = env::storage_usage();
        // remove from mappings
        self.applications_by_id.remove(&project_id);
        // refund for storage freed
        refund_deposit(initial_storage_usage);
    }

    pub fn get_applications(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
        status: Option<ApplicationStatus>,
    ) -> Vec<Application> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.applications_by_id.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.unwrap_or(usize::MAX as u64);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        if let Some(status) = status {
            self.applications_by_id
                .iter()
                .skip(start_index as usize)
                .take(limit.try_into().unwrap())
                .filter(|(_account_id, application)| {
                    Application::from(application).status == status
                })
                .map(|(_account_id, application)| Application::from(application))
                .collect()
        } else {
            self.applications_by_id
                .iter()
                .skip(start_index as usize)
                .take(limit.try_into().unwrap())
                .map(|(_account_id, application)| Application::from(application))
                .collect()
        }
    }

    pub fn get_approved_applications(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<Application> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.approved_application_ids.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.approved_application_ids
            .iter()
            .skip(start_index as usize)
            .take(limit.try_into().unwrap())
            .map(|project_id| {
                Application::from(
                    self.applications_by_id
                        .get(&project_id)
                        .expect("Application does not exist"),
                )
            })
            .collect()
    }

    pub fn get_application_by_project_id(&self, project_id: ProjectId) -> Application {
        Application::from(
            self.applications_by_id
                .get(&project_id)
                .expect("Application does not exist"),
        )
    }

    #[payable]
    pub fn chef_set_application_status(
        &mut self,
        project_id: ProjectId,
        status: ApplicationStatus,
        notes: String,
    ) -> Application {
        self.assert_chef_or_greater();
        // verify that the application exists
        let mut application = Application::from(
            self.applications_by_id
                .get(&project_id)
                .expect("Application does not exist"),
        );
        // update application
        let previous_status = application.status.clone();
        application.status = status;
        application.updated_at = Some(env::block_timestamp_ms());
        application.review_notes = Some(notes);
        // update mapping
        self.applications_by_id.insert(
            &project_id,
            &VersionedApplication::Current(application.clone()),
        );
        // insert into approved applications mapping if approved
        if application.status == ApplicationStatus::Approved {
            // check that max_projects hasn't been reached
            self.assert_max_projects_not_reached();
            self.approved_application_ids.insert(&project_id);
        } else {
            // setting application status as something other than Approved; if it was previously approved, remove from approved mapping
            if previous_status == ApplicationStatus::Approved {
                self.approved_application_ids.remove(&project_id);
            }
        }
        application
    }

    // TODO: consider removing convenience methods below

    #[payable]
    pub fn chef_mark_application_approved(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Approved, notes)
    }

    #[payable]
    pub fn chef_mark_application_rejected(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Rejected, notes)
    }

    #[payable]
    pub fn chef_mark_application_in_review(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::InReview, notes)
    }

    #[payable]
    pub fn chef_mark_application_pending(
        &mut self,
        project_id: ProjectId,
        notes: String,
    ) -> Application {
        self.chef_set_application_status(project_id, ApplicationStatus::Pending, notes)
    }
}
