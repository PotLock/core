use crate::*;

#[near_bindgen]
impl Contract {
    // CHANGE OWNER
    pub fn owner_change_owner(&mut self, new_owner: AccountId) {
        self.assert_owner();
        self.owner = new_owner;
    }

    // ADD ADMINS
    pub fn owner_add_admins(&mut self, new_admins: Vec<AccountId>) {
        self.assert_owner();
        for new_admin in new_admins.iter() {
            self.admins.insert(new_admin);
        }
    }

    // REMOVE ADMINS
    pub fn owner_remove_admins(&mut self, admins_to_remove: Vec<AccountId>) {
        self.assert_owner();
        for admin_to_remove in admins_to_remove.iter() {
            self.admins.remove(admin_to_remove);
        }
    }

    // APPLICATION START
    pub fn admin_set_application_start_ms(&mut self, application_start_ms: u64) {
        self.assert_owner_or_admin();
        self.application_start_ms = application_start_ms;
    }

    // APPLICATION END
    pub fn admin_set_application_end_ms(&mut self, application_end_ms: u64) {
        self.assert_owner_or_admin();
        self.application_end_ms = application_end_ms;
    }

    // CHEF
    pub fn admin_set_chef(&mut self, chef: AccountId) {
        self.assert_owner_or_admin();
        self.chef.set(&chef);
    }

    pub fn admin_set_chef_fee_basis_points(&mut self, chef_fee_basis_points: u32) {
        self.assert_owner_or_admin();
        self.chef_fee_basis_points = chef_fee_basis_points;
    }

    // ROUND
    pub fn admin_set_round_open(&mut self, public_round_end_ms: TimestampMs) {
        self.assert_owner_or_admin();
        self.public_round_start_ms = env::block_timestamp_ms();
        self.public_round_end_ms = public_round_end_ms;
    }

    pub fn admin_close_round(&mut self) {
        self.assert_owner_or_admin();
        self.public_round_end_ms = env::block_timestamp_ms();
    }

    // PAYOUTS
    pub fn admin_process_payouts(&mut self) {
        self.assert_owner_or_admin();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.all_paid_out == false,
            "Payouts have already been processed"
        );
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // pay out each project
        // for each approved project...
        for (project_id, application) in self.applications_by_project_id.iter() {
            self.assert_approved_application(&project_id);
            // ...if there are payouts for the project...
            if let Some(payout_ids_for_project) = self.payout_ids_by_project_id.get(&project_id) {
                // TODO: handle milestones (for now just paying out all payouts)
                for payout_id in payout_ids_for_project.iter() {
                    let mut payout = self.payouts_by_id.get(&payout_id).expect("no payout");
                    if payout.paid_at.is_none() {
                        // ...transfer funds...
                        Promise::new(application.project_id.clone())
                            .transfer(payout.amount_total.0); // TODO: what happens if this fails?
                                                              // ...and update payout to indicate that funds have been transferred
                                                              // TODO: handle via Promise callback?
                        payout.paid_at = Some(env::block_timestamp_ms());
                        self.payouts_by_id.insert(&payout_id, &payout);
                    }
                }
            }
        }
        self.all_paid_out = true;
    }

    pub fn admin_set_cooldown_period_complete(&mut self) {
        self.assert_owner_or_admin();
        self.cooldown_end_ms.set(&env::block_timestamp_ms());
    }
}
