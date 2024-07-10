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
    pub recipient_id: AccountId,
    /// Amount to be paid out
    pub amount: u128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
    /// Memo field for payout notes
    pub memo: Option<String>,
    /// E.g. true if a return of funds to sponsors
    pub is_redistribution: bool,
}

/// Ephemeral-only; used for setting payouts
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutInput {
    pub amount: U128,
    pub recipient_id: ProjectId,
    pub is_redistribution: bool, // TODO: Could also determine this based on recipient being an approved project in this round
    pub memo: Option<String>,
}

/// Ephemeral-only
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutExternal {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// ID of the application receiving the payout
    pub recipient_id: AccountId,
    /// Amount to be paid out
    pub amount: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
    /// Memo field for payout notes
    pub memo: Option<String>,
    /// Used for tracking whether it was a matching pool payout or something else, e.g. a return of funds to sponsors
    pub is_redistribution: bool,
}

impl Payout {
    pub fn to_external(&self) -> PayoutExternal {
        PayoutExternal {
            id: self.id.clone(),
            recipient_id: self.recipient_id.clone(),
            amount: U128(self.amount),
            paid_at: self.paid_at,
            memo: self.memo.clone(),
            is_redistribution: self.is_redistribution,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutsChallenge {
    /// Timestamp when the payout challenge was made
    pub created_at: TimestampMs,
    /// Reason for the challenge
    pub reason: String,
    /// Notes from admin/owner
    pub admin_notes: Option<String>,
    /// Whether the challenge has been resolved
    pub resolved: bool,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedPayoutsChallenge {
    Current(PayoutsChallenge),
}

impl From<VersionedPayoutsChallenge> for PayoutsChallenge {
    fn from(payouts_challenge: VersionedPayoutsChallenge) -> Self {
        match payouts_challenge {
            VersionedPayoutsChallenge::Current(current) => current,
        }
    }
}

/// Ephemeral-only
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutsChallengeExternal {
    /// Account that made the challenge
    pub challenger_id: AccountId,
    /// Timestamp when the payout challenge was made
    pub created_at: TimestampMs,
    /// Reason for the challenge
    pub reason: String,
    /// Notes from admin/owner
    pub admin_notes: Option<String>,
    /// Whether the challenge has been resolved
    pub resolved: bool,
}

impl PayoutsChallenge {
    pub fn to_external(&self, challenger_id: AccountId) -> PayoutsChallengeExternal {
        PayoutsChallengeExternal {
            challenger_id,
            created_at: self.created_at,
            reason: self.reason.clone(),
            admin_notes: self.admin_notes.clone(),
            resolved: self.resolved,
        }
    }
}

#[near_bindgen]
impl Contract {
    // set_payouts (callable by chef or admin)
    #[payable]
    pub fn chef_set_payouts(&mut self, payouts: Vec<PayoutInput>, clear_existing: bool) {
        self.assert_chef_or_greater();
        // verify that the round has closed
        self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.all_paid_out == false,
            "Payouts have already been processed"
        );
        // clear any existing payouts if requested
        if clear_existing {
            for application_id in self.approved_application_ids.iter() {
                // if there are payouts for the project...
                if let Some(payout_ids_for_recipient) =
                    self.payout_ids_by_recipient_id.get(&application_id)
                {
                    // ...remove them
                    for payout_id in payout_ids_for_recipient.iter() {
                        self.payouts_by_id.remove(&payout_id);
                    }
                    // ...and remove the set of payout IDs for the project
                    let removed = self.payout_ids_by_recipient_id.remove(&application_id);
                    if let Some(mut removed) = removed {
                        removed.clear();
                    }
                }
            }
        }
        // get down to business
        let mut running_total: u128 = 0;
        // for each payout:
        for payout in payouts.iter() {
            // verify that the project exists and is approved
            self.assert_approved_application(&payout.recipient_id);
            // TODO: check that the project is not owner, admin or chef
            // add amount to running total
            running_total += payout.amount.0;
            // set cooldown_end to now + cooldown period ms
            self.cooldown_end_ms
                .set(&(env::block_timestamp_ms() + &self.cooldown_period_ms));
            // if compliance_period_ms is set, set compliance_end to now + compliance period ms
            if let Some(compliance_period_ms) = self.compliance_period_ms.get() {
                self.compliance_end_ms
                    .set(&(env::block_timestamp_ms() + compliance_period_ms));
            }
            // add payout to payouts
            let mut payout_ids_for_recipient = self
                .payout_ids_by_recipient_id
                .get(&payout.recipient_id)
                .unwrap_or(UnorderedSet::new(StorageKey::PayoutIdsByRecipientIdInner {
                    recipient_id: payout.recipient_id.clone(),
                }));
            let payout_id = format!(
                "{}{}{}",
                payout.recipient_id,
                PAYOUT_ID_DELIMITER,
                payout_ids_for_recipient.len() + 1
            );
            let payout = Payout {
                id: payout_id.clone(),
                amount: payout.amount.0,
                recipient_id: payout.recipient_id.clone(),
                paid_at: None,
                memo: payout.memo.clone(),
                is_redistribution: payout.is_redistribution,
            };
            payout_ids_for_recipient.insert(&payout_id);
            self.payout_ids_by_recipient_id
                .insert(&payout.recipient_id, &payout_ids_for_recipient);
            self.payouts_by_id.insert(&payout_id, &payout);
        }
        // error if running total is more than matching pool balance
        assert!(
            running_total <= self.matching_pool_balance,
            "Total payouts ({}) must not be greater than matching pool balance ({})",
            running_total,
            self.matching_pool_balance
        );
    }

    pub fn get_payouts(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<PayoutExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.payouts_by_id.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.unwrap_or(usize::MAX as u64);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.payouts_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .map(|(_payout_id, payout)| Payout::from(payout).to_external())
            .collect()
    }

    #[payable]
    /// Processes any payouts that have been set but not yet paid out. Takes optional vec of account IDs to process payouts for.
    pub fn admin_process_payouts(&mut self, project_ids: Option<Vec<ProjectId>>) {
        self.assert_admin_or_greater();
        // verify that the round has closed
        self.assert_round_closed();
        // // verify that payouts have not already been processed
        // TODO: verify that this should be removed
        // assert!(
        //     self.all_paid_out == false,
        //     "Payouts have already been processed"
        // );
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // verify that any challenges have been resolved
        self.assert_all_payouts_challenges_resolved();
        // pay out each project
        // for each approved project...
        // loop through self.approved_application_ids set
        for project_id in self.approved_application_ids.iter() {
            // if project_ids is Some, skip if project_id is not in project_ids
            if let Some(ref project_ids) = project_ids {
                if !project_ids.contains(&project_id) {
                    continue;
                }
            }
            // get application
            let application = Application::from(
                self.applications_by_id
                    .get(&project_id)
                    .expect("no application"),
            );
            // check that the project is not owner, admin or chef
            if self.is_owner_or_admin(Some(&project_id)) || self.is_chef(Some(&project_id)) {
                log!("Skipping payout for project {} as it is owner, admin or chef and not eligible for payouts.", project_id);
            } else {
                // ...if there are payouts for the project...
                if let Some(payout_ids_for_project) =
                    self.payout_ids_by_recipient_id.get(&project_id)
                {
                    // TODO: handle milestones (for now just paying out all payouts)
                    for payout_id in payout_ids_for_project.iter() {
                        let mut payout =
                            Payout::from(self.payouts_by_id.get(&payout_id).expect("no payout"));
                        if payout.paid_at.is_none() {
                            // ...transfer funds...
                            Promise::new(application.project_id.clone())
                                .transfer(payout.amount)
                                .then(
                                    Self::ext(env::current_account_id())
                                        .with_static_gas(XCC_GAS)
                                        .transfer_payout_callback(payout.clone()),
                                );
                        }
                    }
                }
            }
        }
        // self.all_paid_out = true; // TODO: verify that this should be removed
    }

    #[payable]
    /// Distribute funds to a list of recipients, without a cooldown period. Does not enforce that recipients are approved projects.
    pub fn owner_distribute_funds(&mut self, distributions: Vec<PayoutInput>) {
        self.assert_owner();
        // verify that the round has closed
        self.assert_round_closed();
        // get down to business
        let mut running_total: u128 = 0;
        let mut payouts: Vec<Payout> = Vec::new();
        // for each payout:
        for payout_input in distributions.iter() {
            // add amount to running total
            running_total += payout_input.amount.0;
            // add payout to payouts
            let mut payout_ids_for_recipient = self
                .payout_ids_by_recipient_id
                .get(&payout_input.recipient_id)
                .unwrap_or(UnorderedSet::new(StorageKey::PayoutIdsByRecipientIdInner {
                    recipient_id: payout_input.recipient_id.clone(),
                }));
            let payout_id = format!(
                "{}{}{}",
                payout_input.recipient_id,
                PAYOUT_ID_DELIMITER,
                payout_ids_for_recipient.len() + 1
            );
            let payout = Payout {
                id: payout_id.clone(),
                amount: payout_input.amount.0,
                recipient_id: payout_input.recipient_id.clone(),
                paid_at: None,
                memo: payout_input.memo.clone(),
                is_redistribution: payout_input.is_redistribution,
            };
            payout_ids_for_recipient.insert(&payout_id);
            self.payout_ids_by_recipient_id
                .insert(&payout_input.recipient_id, &payout_ids_for_recipient);
            self.payouts_by_id.insert(&payout_id, &payout);
            payouts.push(payout);
        }
        // error if running total is more than matching pool balance
        assert!(
            running_total <= self.matching_pool_balance,
            "Total payouts ({}) must not be greater than matching pool balance ({})",
            running_total,
            self.matching_pool_balance
        );
        // pay out each recipient
        for payout in payouts.iter() {
            // ...transfer funds...
            Promise::new(payout.recipient_id.clone())
                .transfer(payout.amount)
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(XCC_GAS)
                        .transfer_payout_callback(payout.clone()),
                );
        }
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
                "Error paying out amount {:#?} to recipient {}",
                payout.amount, payout.recipient_id
            ));
            // update payout to indicate error transferring funds
            payout.paid_at = None;
            self.payouts_by_id.insert(&payout.id.clone(), &payout);
        } else {
            log!(format!(
                "Successfully paid out amount {:#?} to recipient {}",
                payout.amount, payout.recipient_id
            ));
            // decrement matching pool balance
            self.matching_pool_balance -= payout.amount;
            // update payout to indicate that funds transfer has been completed
            payout.paid_at = Some(env::block_timestamp_ms());
            self.payouts_by_id.insert(&payout.id.clone(), &payout);
        }
    }

    #[payable]
    pub fn challenge_payouts(&mut self, reason: String) {
        // anyone can challenge
        // verify that cooldown is in process
        self.assert_cooldown_period_in_process();
        // create challenge & store, charging user for storage
        let initial_storage_usage = env::storage_usage();
        let challenge = PayoutsChallenge {
            created_at: env::block_timestamp_ms(),
            reason,
            admin_notes: None,
            resolved: false,
        };
        // store challenge (overwriting any existing challenge for this user - only one challenge per user allowed)
        self.payouts_challenges.insert(
            &env::predecessor_account_id(),
            &VersionedPayoutsChallenge::Current(challenge),
        );
        refund_deposit(initial_storage_usage);
    }

    pub fn remove_payouts_challenge(&mut self) {
        // verify that cooldown is in process
        self.assert_cooldown_period_in_process();
        // if a payout challenge exists for this caller, remove it (if unresolved) & refund for freed storage
        if let Some(versioned_challenge) =
            self.payouts_challenges.get(&env::predecessor_account_id())
        {
            let challenge = PayoutsChallenge::from(versioned_challenge);
            if !challenge.resolved {
                let initial_storage_usage = env::storage_usage();
                self.payouts_challenges
                    .remove(&env::predecessor_account_id());
                refund_deposit(initial_storage_usage);
            } else {
                panic!("Payout challenge already resolved; cannot be removed");
            }
        }
    }

    pub fn get_payouts_challenges(
        &self,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<PayoutsChallengeExternal> {
        let start_index: u64 = from_index.unwrap_or_default();
        assert!(
            (self.payouts_challenges.len() as u64) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.unwrap_or(usize::MAX as u64);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.payouts_challenges
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .map(|(challenger_id, versioned_challenge)| {
                let challenge = PayoutsChallenge::from(versioned_challenge);
                challenge.to_external(challenger_id)
            })
            .collect()
    }
}
