use crate::*;

impl Contract {
    pub(crate) fn assert_chef(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.chef_id,
            "Only the chef can call this method"
        );
    }

    pub(crate) fn assert_round_closed(&self) {
        assert!(
            env::block_timestamp_ms() >= self.end_time,
            "Round is still open"
        );
    }

    pub(crate) fn assert_round_open(&self) {
        assert!(env::block_timestamp_ms() < self.end_time, "Round is closed");
    }

    pub(crate) fn assert_approved_project(&self, project_id: &ProjectId) {
        let project_exists = self.approved_project_ids.contains(project_id);
        assert!(project_exists, "Project does not exist");
    }
}
