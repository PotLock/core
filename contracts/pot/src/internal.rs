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
            env::block_timestamp_ms() >= self.round_end_time,
            "Round is still open"
        );
    }

    pub(crate) fn assert_approved_application(&self, application_id: &ApplicationId) {
        let application = self
            .applications_by_id
            .get(application_id)
            .expect("Application does not exist");
        assert!(
            application.status == ApplicationStatus::Approved,
            "Application is not approved"
        );
    }

    pub(crate) fn assert_cooldown_period_complete(&self) {
        assert!(
            self.cooldown_end_ms.is_some()
                && self.cooldown_end_ms.unwrap() < env::block_timestamp_ms(),
            "Cooldown period is not over"
        );
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
        let mut approved_applications_count = 0;
        for application_id in self.application_ids.iter() {
            let application = self
                .applications_by_id
                .get(&application_id)
                .expect("Application does not exist");
            if application.status == ApplicationStatus::Approved {
                approved_applications_count += 1;
            }
        }
        assert!(
            approved_applications_count < self.max_projects,
            "Max projects reached"
        );
    }
}
