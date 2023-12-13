use crate::*;

impl Contract {
    pub(crate) fn is_owner(&self) -> bool {
        env::predecessor_account_id() == self.owner
    }

    pub(crate) fn is_admin(&self) -> bool {
        self.admins.contains(&env::predecessor_account_id())
    }

    pub(crate) fn is_owner_or_admin(&self) -> bool {
        self.is_owner() || self.is_admin()
    }

    pub(crate) fn assert_owner(&self) {
        assert!(self.is_owner(), "Only contract owner can call this method");
    }

    pub(crate) fn assert_admin_or_greater(&self) {
        assert!(
            self.is_admin() || self.is_owner(),
            "Only contract admin or owner can call this method"
        );
    }

    pub(crate) fn assert_owner_or_admin(&self) {
        assert!(
            self.is_owner_or_admin(),
            "Only contract owner or admin can call this method"
        );
    }

    pub(crate) fn is_chef(&self) -> bool {
        if let Some(chef) = self.chef.get() {
            env::predecessor_account_id() == chef
        } else {
            false
        }
    }

    /// Asserts that caller is, at minimum, a chef (admin or owner also allowed)
    pub(crate) fn assert_chef_or_greater(&self) {
        assert!(
            self.is_chef() || self.is_admin() || self.is_owner(),
            "Only chef, admin or owner can call this method"
        );
    }

    // pub(crate) fn assert_pot_deployer_admin(&self) {
    //     assert!(
    //         self.pot_deployer_admins
    //             .contains(&env::predecessor_account_id()),
    //         "Only the pot deployer admin can call this method"
    //     );
    // }

    pub(crate) fn assert_round_closed(&self) {
        assert!(
            env::block_timestamp_ms() >= self.public_round_end_ms,
            "Round is still open"
        );
    }

    pub(crate) fn assert_round_not_closed(&self) {
        assert!(
            env::block_timestamp_ms() < self.public_round_end_ms,
            "Round is closed"
        );
    }

    pub(crate) fn assert_approved_application(&self, project_id: &ProjectId) {
        assert!(
            self.approved_application_ids.contains(project_id),
            "Approved application does not exist"
        );
    }

    pub(crate) fn assert_cooldown_period_complete(&self) {
        if let Some(cooldown_end_ms) = self.cooldown_end_ms.get() {
            assert!(
                cooldown_end_ms < env::block_timestamp_ms(),
                "Cooldown period is not over"
            );
        } else {
            panic!("Cooldown period is not set");
        }
    }

    pub(crate) fn is_application_period_open(&self) -> bool {
        let block_timestamp_ms = env::block_timestamp_ms();
        block_timestamp_ms >= self.application_start_ms
            && block_timestamp_ms < self.application_end_ms
    }

    pub(crate) fn assert_application_period_open(&self) {
        assert!(
            self.is_application_period_open(),
            "Application period is not open"
        );
    }

    pub(crate) fn assert_round_active(&self) {
        assert!(self.is_round_active(), "Round is not active");
    }

    pub(crate) fn assert_max_projects_not_reached(&self) {
        assert!(
            self.approved_application_ids.len() < self.max_projects.into(),
            "Max projects reached"
        );
    }
}
