use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatus {
    Submitted,
    InReview,
    Approved,
    Rejected,
    Graylisted,
    Blacklisted,
}

// ProjectInternal is the data structure that is stored within the contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectInternal {
    pub id: ProjectId,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedProjectInternal {
    Current(ProjectInternal),
}

impl From<VersionedProjectInternal> for ProjectInternal {
    fn from(project_internal: VersionedProjectInternal) -> Self {
        match project_internal {
            VersionedProjectInternal::Current(current) => current,
        }
    }
}

// Ephemeral data structure used for view methods, not stored within contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectExternal {
    pub id: ProjectId,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn register(&mut self, _project_id: Option<AccountId>) -> ProjectExternal {
        let initial_storage_usage = env::storage_usage();

        // _project_id can only be specified by admin; otherwise, it is the caller
        let project_id = if let Some(_project_id) = _project_id {
            self.assert_admin();
            _project_id
        } else {
            env::predecessor_account_id()
        };

        // make sure project doesn't already exist at this Project ID
        self.assert_project_does_not_exist(&project_id);

        // create project
        let project_internal = ProjectInternal {
            id: project_id.clone(),
            status: ProjectStatus::Approved, // approved by default - TODO: double-check that this is desired functionality
            submitted_ms: env::block_timestamp_ms(),
            updated_ms: env::block_timestamp_ms(),
            review_notes: None,
        };

        // update mappings
        self.project_ids.insert(&project_id);
        self.projects_by_id.insert(
            &project_id,
            &VersionedProjectInternal::Current(project_internal.clone()),
        );

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        // return formatted project
        self.format_project(project_internal)
    }

    pub fn get_projects(&self) -> Vec<ProjectExternal> {
        self.project_ids
            .iter()
            .map(|project_id| {
                self.format_project(ProjectInternal::from(
                    self.projects_by_id.get(&project_id).expect("No project"),
                ))
            })
            .collect()
    }

    pub fn get_project_by_id(&self, project_id: ProjectId) -> ProjectExternal {
        self.format_project(ProjectInternal::from(
            self.projects_by_id.get(&project_id).expect("No project"),
        ))
    }

    pub(crate) fn format_project(&self, project_internal: ProjectInternal) -> ProjectExternal {
        ProjectExternal {
            id: project_internal.id.clone(),
            status: project_internal.status,
            submitted_ms: project_internal.submitted_ms,
            updated_ms: project_internal.updated_ms,
            review_notes: project_internal.review_notes,
        }
    }

    #[payable]
    pub fn admin_set_project_status(
        &mut self,
        project_id: ProjectId,
        status: ProjectStatus,
        review_notes: Option<String>,
    ) {
        self.assert_admin();
        self.assert_project_exists(&project_id);
        let mut project =
            ProjectInternal::from(self.projects_by_id.get(&project_id).expect("No project"));
        project.status = status;
        project.review_notes = review_notes;
        project.updated_ms = env::block_timestamp_ms();
        self.projects_by_id
            .insert(&project_id, &VersionedProjectInternal::Current(project));
    }
}
