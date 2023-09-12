use crate::*;

// PAYOUTS

pub const PAYOUT_ID_DELIMITER: &str = ":";
pub type PayoutId = String; // concatenation of application_id + PAYOUT_ID_DELIMITER + incrementing integer per-project

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// ID of the application receiving the payout
    pub application_id: ApplicationId,
    // /// ID of the project receiving the payout
    // pub project_id: ProjectId,
    /// Amount paid out
    pub amount: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
}

pub struct PayoutInput {
    pub amount: U128,
    pub application_id: ApplicationId,
}

impl Contract {
    // set_payouts (only callable by chef)
    pub fn chef_set_payouts(&mut self, payouts: Vec<PayoutInput>) {
        self.assert_chef();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.paid_out == false,
            "Payouts have already been processed"
        );
        // clear any existing payouts (in case this is a reset, e.g. fixing an error)
        for application_id in self.application_ids.iter() {
            if let Some(payout_ids_for_application) =
                self.payout_ids_by_application_id.get(&application_id)
            {
                for payout_id in payout_ids_for_application.iter() {
                    self.payouts_by_id.remove(&payout_id);
                }
                self.payout_ids_by_application_id.remove(&application_id);
            }
        }
        // get down to business
        let mut running_total: u128 = 0;
        let balance_available = self
            .total_matching_pool_funds
            .0
            .checked_add(self.total_donations_funds.0)
            .expect(&format!(
                "Overflow occurred when calculating balance available ({} + {})",
                self.total_matching_pool_funds.0, self.total_donations_funds.0,
            ));
        // for each payout:
        for payout in payouts.iter() {
            // 1. verify that the project exists and is approved
            self.assert_approved_application(&payout.application_id);
            // 2. verify that the project is not already paid out
            let existing_payout = self
                .payout_ids_by_application_id
                .get(&payout.application_id);
            assert!(
                existing_payout.is_none(),
                "Project has already been paid out"
            );
            // 3. add amount to running total
            running_total += payout.amount.0;
            // error if running total exceeds round total
            assert!(
                running_total <= balance_available,
                "Payouts exceed available balance"
            );
            // set cooldown_end to now + 1 week (?)
            self.cooldown_end_ms = Some(env::block_timestamp_ms() + ONE_WEEK_MS);
            // add payout to payouts
            let mut payout_ids_for_application = self
                .payout_ids_by_application_id
                .get(&payout.application_id)
                .unwrap_or(UnorderedSet::new(
                    StorageKey::PayoutIdsByApplicationIdInner {
                        application_id: payout.application_id.clone(),
                    },
                ));
            let payout_id = format!(
                "{}{}{}",
                payout.application_id,
                PAYOUT_ID_DELIMITER,
                payout_ids_for_application.len() + 1
            );
            let payout = Payout {
                id: payout_id.clone(),
                amount: payout.amount,
                application_id: payout.application_id.clone(),
                paid_at: None,
            };
            payout_ids_for_application.insert(&payout_id);
            self.payout_ids_by_application_id
                .insert(&payout.application_id, &payout_ids_for_application);
            self.payouts_by_id.insert(&payout_id, &payout);
        }
    }

    pub fn chef_process_payouts(&mut self) {
        self.assert_chef();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.paid_out == false,
            "Payouts have already been processed"
        );
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // pay out each project
        // for each approved project...
        for application_id in self.application_ids.iter() {
            self.assert_approved_application(&application_id);
            // ...if there are payouts for the project...
            if let Some(payout_ids_for_project) =
                self.payout_ids_by_application_id.get(&application_id)
            {
                // TODO: handle milestones (for now just paying out all payouts)
                for payout_id in payout_ids_for_project.iter() {
                    let mut payout = self.payouts_by_id.get(&payout_id).expect("no payout");
                    if payout.paid_at.is_none() {
                        // ...transfer funds...
                        let payout_to = self
                            .applications_by_id
                            .get(&payout.application_id)
                            .expect("no application")
                            .project_id; // TODO: consider adding payout_to to Payout struct
                                         // TODO: what happens if this fails?
                        Promise::new(payout_to).transfer(payout.amount.0);
                        // ...and update payout to indicate that funds have been transferred
                        // TODO: handle via Promise callback?
                        payout.paid_at = Some(env::block_timestamp_ms());
                        self.payouts_by_id.insert(&payout_id, &payout);
                    }
                }
            }
        }
    }

    pub fn get_payouts(&self) -> Vec<Payout> {
        // could add pagination but not necessary initially
        self.payouts_by_id.values().collect()
    }

    // challenge_payouts (callable by anyone on ReFi Council)
}
