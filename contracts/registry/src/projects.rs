use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatusV1 {
    Submitted,
    InReview,
    Approved,
    Rejected,
    Graylisted,
    Blacklisted,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProjectStatus {
    Pending,
    Approved,
    Rejected,
    Graylisted,
    Blacklisted,
}

// OLD (v1) - ProjectInternal is the data structure that is stored within the contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectInternalV1 {
    pub id: ProjectId,
    pub status: ProjectStatusV1,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

// CURRENT - ProjectInternal is the data structure that is stored within the contract
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
    V1(ProjectInternalV1),
    Current(ProjectInternal),
}

impl From<VersionedProjectInternal> for ProjectInternal {
    fn from(project_internal: VersionedProjectInternal) -> Self {
        match project_internal {
            VersionedProjectInternal::V1(v1) => ProjectInternal {
                id: v1.id,
                status: match v1.status {
                    ProjectStatusV1::Submitted => ProjectStatus::Pending,
                    ProjectStatusV1::InReview => ProjectStatus::Pending,
                    ProjectStatusV1::Approved => ProjectStatus::Approved,
                    ProjectStatusV1::Rejected => ProjectStatus::Rejected,
                    ProjectStatusV1::Graylisted => ProjectStatus::Rejected,
                    ProjectStatusV1::Blacklisted => ProjectStatus::Rejected,
                },
                submitted_ms: v1.submitted_ms,
                updated_ms: v1.updated_ms,
                review_notes: v1.review_notes,
            },
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
            status: self.default_project_status.clone(),
            submitted_ms: env::block_timestamp_ms(),
            updated_ms: env::block_timestamp_ms(),
            review_notes: None,
        };

        // update mappings
        // self.project_ids.insert(&project_id);
        self.projects_by_id.insert(
            &project_id,
            &VersionedProjectInternal::Current(project_internal.clone()),
        );
        match project_internal.status {
            ProjectStatus::Pending => {
                self.pending_project_ids.insert(&project_id);
            }
            ProjectStatus::Approved => {
                self.approved_project_ids.insert(&project_id);
            }
            ProjectStatus::Rejected => {
                self.rejected_project_ids.insert(&project_id);
            }
            ProjectStatus::Graylisted => {
                self.graylisted_project_ids.insert(&project_id);
            }
            ProjectStatus::Blacklisted => {
                self.blacklisted_project_ids.insert(&project_id);
            }
        }

        // refund any unused deposit
        refund_deposit(initial_storage_usage);

        // return formatted project
        self.format_project(project_internal)
    }

    pub fn get_projects(
        &self,
        status: Option<ProjectStatus>,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<ProjectExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        if let Some(status) = status {
            match status {
                ProjectStatus::Pending => {
                    assert!(
                        (self.pending_project_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.pending_project_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|project_id| {
                            self.format_project(ProjectInternal::from(
                                self.projects_by_id.get(&project_id).expect("No project"),
                            ))
                        })
                        .collect()
                }
                ProjectStatus::Approved => {
                    assert!(
                        (self.approved_project_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.approved_project_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|project_id| {
                            self.format_project(ProjectInternal::from(
                                self.projects_by_id.get(&project_id).expect("No project"),
                            ))
                        })
                        .collect()
                }
                ProjectStatus::Rejected => {
                    assert!(
                        (self.rejected_project_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.rejected_project_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|project_id| {
                            self.format_project(ProjectInternal::from(
                                self.projects_by_id.get(&project_id).expect("No project"),
                            ))
                        })
                        .collect()
                }
                ProjectStatus::Graylisted => {
                    assert!(
                        (self.graylisted_project_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.graylisted_project_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|project_id| {
                            self.format_project(ProjectInternal::from(
                                self.projects_by_id.get(&project_id).expect("No project"),
                            ))
                        })
                        .collect()
                }
                ProjectStatus::Blacklisted => {
                    assert!(
                        (self.blacklisted_project_ids.len() as u64) >= start_index,
                        "Out of bounds, please use a smaller from_index."
                    );
                    self.blacklisted_project_ids
                        .iter()
                        .skip(start_index as usize)
                        .take(limit)
                        .map(|project_id| {
                            self.format_project(ProjectInternal::from(
                                self.projects_by_id.get(&project_id).expect("No project"),
                            ))
                        })
                        .collect()
                }
            }
        } else {
            assert!(
                (self.projects_by_id.len() as u64) >= start_index,
                "Out of bounds, please use a smaller from_index."
            );
            self.projects_by_id
                .iter()
                .skip(start_index as usize)
                .take(limit.try_into().unwrap())
                .map(|(_project_id, project_internal)| {
                    self.format_project(ProjectInternal::from(project_internal))
                })
                .collect()
        }
    }

    pub fn get_project_by_id(&self, project_id: ProjectId) -> ProjectExternal {
        self.format_project(ProjectInternal::from(
            self.projects_by_id.get(&project_id).expect("No project"),
        ))
    }

    pub fn is_registered(&self, account_id: ProjectId) -> bool {
        self.projects_by_id.get(&account_id).is_some()
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
}
