use crate::*;

impl Contract {
    pub(crate) fn assert_at_least_one_yocto(&self) {
        assert!(
            env::attached_deposit() >= 1,
            "At least one yoctoNEAR must be attached"
        );
    }

    pub(crate) fn assert_list_owner(&self, list_id: &ListId) {
        let list = ListInternal::from(self.lists_by_id.get(list_id).expect("List does not exist"));
        assert_eq!(
            env::predecessor_account_id(),
            list.owner,
            "List owner-only action"
        );
        // require owner to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    pub(crate) fn is_caller_list_admin_or_greater(&self, list_id: &ListId) -> bool {
        let predecessor_account_id = env::predecessor_account_id();
        let list = ListInternal::from(self.lists_by_id.get(list_id).expect("List does not exist"));
        let list_admins = self
            .list_admins_by_list_id
            .get(list_id)
            .expect("List admins do not exist");
        list.owner == predecessor_account_id || list_admins.contains(&predecessor_account_id)
    }

    pub(crate) fn assert_list_admin_or_greater(&self, list_id: &ListId) {
        assert!(
            self.is_caller_list_admin_or_greater(list_id),
            "List admin-only action"
        );
        // require caller to attach at least one yoctoNEAR for security purposes
        self.assert_at_least_one_yocto();
    }

    // pub(crate) fn assert_project_exists(&self, project_id: &AccountId) {
    //     assert!(
    //         self.projects_by_id.get(project_id).is_some(),
    //         "Project does not exist"
    //     );
    // }

    // pub(crate) fn assert_project_does_not_exist(&self, project_id: &AccountId) {
    //     assert!(
    //         !self.projects_by_id.get(project_id).is_some(),
    //         "Project already exists"
    //     );
    // }
}
