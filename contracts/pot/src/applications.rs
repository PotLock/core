use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ApplicationStatus {
    Pending,
    Accepted,
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
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn apply(&mut self, project_id: ProjectId) -> Application {
        // TODO: verify that the project ID exists as an approved project on Registry contract
        // for now, assume that the project_id is valid
        // check that application doesn't already exist for this project
        if self.application_id_by_project_id.get(&project_id).is_some() {
            // application already exists
            env::panic_str("Application already exists for this project");
        }
        // check that application period is open
        self.assert_application_period_open();
        // add application
        let application = Application {
            id: self.applications_by_id.len() + 1 as ApplicationId,
            project_id,
            status: ApplicationStatus::Pending,
            submitted_at: env::block_timestamp_ms(),
            updated_at: None,
        };
        // update mappings
        self.applications_by_id
            .insert(&application.id, &application);
        self.application_id_by_project_id
            .insert(&application.project_id, &application.id);
        self.pending_application_ids.insert(&application.id);
        // return application
        application
    }
}
