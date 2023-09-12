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
    pub fn apply(&mut self, project_id: ProjectId) -> Application {
        // TODO: verify that the project ID exists as an approved project on Registry contract (could also require that the caller be the project_id)
        // TODO: verify that the caller has permission to take this action
        // for now, assume that the project_id is valid
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

    pub fn unapply(&mut self, application_id: ApplicationId) {
        // TODO: verify that the caller has permission to take this action
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
        self.application_id_by_project_id
            .remove(&application.project_id);
        // TODO: emit event?
    }

    pub fn chef_update_application_status(
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
        self.chef_update_application_status(application_id, ApplicationStatus::Approved, notes)
    }

    pub fn chef_mark_application_rejected(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_update_application_status(application_id, ApplicationStatus::Rejected, notes)
    }

    pub fn chef_mark_application_in_review(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_update_application_status(application_id, ApplicationStatus::InReview, notes)
    }

    pub fn chef_mark_application_pending(
        &mut self,
        application_id: ApplicationId,
        notes: String,
    ) -> Application {
        self.chef_update_application_status(application_id, ApplicationStatus::Pending, notes)
    }
}
