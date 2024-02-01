use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn owner_add_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin in admins {
            self.admins.insert(&admin);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>) {
        self.assert_owner();
        let initial_storage_usage = env::storage_usage();
        for admin in admins {
            self.admins.remove(&admin);
        }
        refund_deposit(initial_storage_usage);
    }

    #[payable]
    pub fn admin_set_default_project_status(&mut self, status: ProjectStatus) {
        self.assert_admin();
        let initial_storage_usage = env::storage_usage();
        self.default_project_status = status;
        refund_deposit(initial_storage_usage);
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
        let old_status = project.status.clone();
        project.status = status.clone();
        project.review_notes = review_notes;
        project.updated_ms = env::block_timestamp_ms();
        self.projects_by_id
            .insert(&project_id, &VersionedProjectInternal::Current(project));
        // add to status-specific set & remove from old status-specific set
        match status {
            ProjectStatus::Pending => {
                self.pending_project_ids.insert(&project_id);
            }
            ProjectStatus::Approved => {
                self.approved_project_ids.insert(&project_id);
            }
            ProjectStatus::Rejected => {
                self.rejected_project_ids.insert(&project_id);
            }
        }
        match old_status {
            ProjectStatus::Pending => {
                self.pending_project_ids.remove(&project_id);
            }
            ProjectStatus::Approved => {
                self.approved_project_ids.remove(&project_id);
            }
            ProjectStatus::Rejected => {
                self.rejected_project_ids.remove(&project_id);
            }
        }
    }
}
