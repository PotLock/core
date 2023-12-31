use crate::*;

// PAYOUTS

pub const PAYOUT_ID_DELIMITER: &str = ":";
pub type PayoutId = String; // concatenation of application_id + PAYOUT_ID_DELIMITER + incrementing integer per-project

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// ID of the application receiving the payout
    pub project_id: ProjectId,
    /// Amount to be paid out
    pub amount: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedPayout {
    Current(Payout),
}

impl From<VersionedPayout> for Payout {
    fn from(payout: VersionedPayout) -> Self {
        match payout {
            VersionedPayout::Current(current) => current,
        }
    }
}

/// Ephemeral-only; used for setting payouts
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutInput {
    pub amount: U128,
    pub project_id: ProjectId,
}

#[near_bindgen]
impl Contract {
    // set_payouts (callable by chef or admin)
    pub fn chef_set_payouts(&mut self, payouts: Vec<PayoutInput>) {
        self.assert_chef_or_greater();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.all_paid_out == false,
            "Payouts have already been processed"
        );
        // clear any existing payouts (in case this is a reset, e.g. fixing an error)
        for application_id in self.approved_application_ids.iter() {
            if let Some(payout_ids_for_application) =
                self.payout_ids_by_project_id.get(&application_id)
            {
                for payout_id in payout_ids_for_application.iter() {
                    self.payouts_by_id.remove(&payout_id);
                }
                self.payout_ids_by_project_id.remove(&application_id);
            }
        }
        // get down to business
        let mut running_total: u128 = 0;
        // for each payout:
        for payout in payouts.iter() {
            // verify that the project exists and is approved
            self.assert_approved_application(&payout.project_id);
            // TODO: check that the project is not owner, admin or chef
            // add amount to running total
            running_total += payout.amount.0;
            // set cooldown_end to now + 1 week (?)
            self.cooldown_end_ms
                .set(&(env::block_timestamp_ms() + ONE_WEEK_MS)); // TODO: remove hardcoding to one week, allow owner/admin to configure
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
                amount: U128::from(payout.amount.0),
                project_id: payout.project_id.clone(),
                paid_at: None,
            };
            payout_ids_for_application.insert(&payout_id);
            self.payout_ids_by_project_id
                .insert(&payout.project_id, &payout_ids_for_application);
            self.payouts_by_id
                .insert(&payout_id, &VersionedPayout::Current(payout));
        }
        // error if running total is not equal to matching_pool_balance (NB: this logic will change once milestones are supported)
        assert!(
            running_total == self.matching_pool_balance.0,
            "Total payouts must equal matching pool balance"
        );
    }

    pub fn get_payouts(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Payout> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.applications_by_id.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.unwrap_or(usize::MAX as u64);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.payouts_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .map(|(_payout_id, payout)| Payout::from(payout))
            .collect()
    }

    pub fn admin_process_payouts(&mut self) {
        self.assert_admin_or_greater();
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
        for (project_id, v_app) in self.applications_by_id.iter() {
            // TODO: update this to only go through approved applications mapping
            self.assert_approved_application(&project_id);
            // TODO: check that the project is not owner, admin or chef
            let application = Application::from(v_app);
            // ...if there are payouts for the project...
            if let Some(payout_ids_for_project) = self.payout_ids_by_project_id.get(&project_id) {
                // TODO: handle milestones (for now just paying out all payouts)
                for payout_id in payout_ids_for_project.iter() {
                    let payout =
                        Payout::from(self.payouts_by_id.get(&payout_id).expect("no payout"));
                    if payout.paid_at.is_none() {
                        // ...transfer funds...
                        Promise::new(application.project_id.clone())
                            .transfer(payout.amount.0)
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(XCC_GAS)
                                    .transfer_payout_callback(payout),
                            );
                    }
                }
            }
        }
        self.all_paid_out = true;
    }

    /// Verifies whether payout transfer completed successfully & updates payout record accordingly
    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_payout_callback(
        &mut self,
        mut payout: Payout,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            log!(format!(
                "Error paying out amount {:#?} to project {}",
                payout.amount, payout.project_id
            ));
        } else {
            log!(format!(
                "Successfully paid out amount {:#?} to project {}",
                payout.amount, payout.project_id
            ));
            // update payout to indicate that funds have been transferred
            payout.paid_at = Some(env::block_timestamp_ms());
            let payout_id = payout.id.clone();
            self.payouts_by_id
                .insert(&payout_id, &VersionedPayout::Current(payout));
        }
    }

    // challenge_payouts (callable by anyone on ReFi Council)
}
