use crate::*;

impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Owner-only action"
        );
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_admin(&self) {
        assert!(
            self.admins.contains(&env::predecessor_account_id()),
            "Admin-only action"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn assert_project_exists(&self, project_id: &AccountId) {
        assert!(
            self.projects_by_id.get(project_id).is_some(),
            "Project does not exist"
        );
    }

    pub(crate) fn assert_project_does_not_exist(&self, project_id: &AccountId) {
        assert!(
            !self.projects_by_id.get(project_id).is_some(),
            "Project already exists"
        );
    }
}
