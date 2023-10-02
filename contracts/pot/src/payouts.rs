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
    pub project_id: ProjectId,
    // /// ID of the project receiving the payout
    // pub project_id: ProjectId,
    /// Amount paid out from the matching pool
    pub matching_pool_amount: U128,
    /// Amount paid out from the donations pool
    pub donations_amount: U128,
    /// Amount paid out in total
    pub amount_total: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutInput {
    pub matching_pool_amount: U128,
    pub donations_amount: U128,
    pub project_id: ProjectId,
}

#[near_bindgen]
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
        for (project_id, application) in self.applications_by_project_id.iter() {
            if let Some(payout_ids_for_application) = self.payout_ids_by_project_id.get(&project_id)
            {
                for payout_id in payout_ids_for_application.iter() {
                    self.payouts_by_id.remove(&payout_id);
                }
                self.payout_ids_by_project_id.remove(&project_id);
            }
        }
        // get down to business
        let mut running_total: u128 = 0;
        let balance_available = self
            .matching_pool_balance
            .0
            .checked_add(self.donations_balance.0)
            .expect(&format!(
                "Overflow occurred when calculating balance available ({} + {})",
                self.matching_pool_balance.0, self.donations_balance.0,
            ));
        // for each payout:
        for payout in payouts.iter() {
            // 1. verify that the project exists and is approved
            self.assert_approved_application(&payout.project_id);
            // 2. verify that the project is not already paid out
            let existing_payout = self.payout_ids_by_project_id.get(&payout.project_id);
            assert!(
                existing_payout.is_none(),
                "Project has already been paid out"
            );
            // 3. add amount to running total
            running_total += payout.matching_pool_amount.0;
            running_total += payout.donations_amount.0;
            // error if running total exceeds round total
            assert!(
                running_total <= balance_available,
                "Payouts exceed available balance"
            );
            // set cooldown_end to now + 1 week (?)
            self.cooldown_end_ms = Some(env::block_timestamp_ms() + ONE_WEEK_MS);
            // add payout to payouts
            let mut payout_ids_for_application = self
                .payout_ids_by_project_id
                .get(&payout.project_id)
                .unwrap_or(UnorderedSet::new(StorageKey::PayoutIdsByProjectIdInner {
                    project_id: payout.project_id.clone(),
                }));
            let payout_id = format!(
                "{}{}{}",
                payout.project_id,
                PAYOUT_ID_DELIMITER,
                payout_ids_for_application.len() + 1
            );
            let payout = Payout {
                id: payout_id.clone(),
                matching_pool_amount: payout.matching_pool_amount.clone(),
                donations_amount: payout.donations_amount.clone(),
                amount_total: U128::from(
                    payout
                        .matching_pool_amount
                        .0
                        .checked_add(payout.donations_amount.0)
                        .expect(&format!(
                            "Overflow occurred when calculating amount_total ({} + {})",
                            payout.matching_pool_amount.0, payout.donations_amount.0,
                        )),
                ),
                project_id: payout.project_id.clone(),
                paid_at: None,
            };
            payout_ids_for_application.insert(&payout_id);
            self.payout_ids_by_project_id
                .insert(&payout.project_id, &payout_ids_for_application);
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
    }

    pub fn get_payouts(&self) -> Vec<Payout> {
        // could add pagination but not necessary initially
        self.payouts_by_id.values().collect()
    }

    // challenge_payouts (callable by anyone on ReFi Council)
}
