use crate::*;

impl Contract {
    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Owner-only action"
        );
    }

    pub(crate) fn assert_admin(&self) {
        assert!(
            self.admins.contains(&env::predecessor_account_id()),
            "Admin-only action"
        );
    }

    pub(crate) fn assert_project_exists(&self, project_id: &AccountId) {
        assert!(
            self.project_ids.contains(project_id),
            "Project does not exist"
        );
    }

    pub(crate) fn assert_project_does_not_exist(&self, project_id: &AccountId) {
        assert!(
            !self.project_ids.contains(project_id),
            "Project already exists"
        );
    }
}
