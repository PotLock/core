use crate::*;

// PAYOUTS

pub const PAYOUT_ID_DELIMITER: &str = ":";
pub type PayoutId = String; // concatenation of application_id + PAYOUT_ID_DELIMITER + incrementing integer per-project

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutBatch {
    pub batch_id: u32,
    pub created_at: TimestampMs,
    pub total_amount: u128,
    pub status: PayoutBatchStatus,
    pub cooldown_end_ms: TimestampMs,
    pub memo: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PayoutBatchStatus {
    InCooldown,
    ReadyToProcess,
    Processing,
    Completed,
    Failed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    /// Unique identifier for the payout
    pub id: PayoutId,
    /// batch id
    pub batch_id: u32,
    /// ID of the application receiving the payout
    pub recipient_id: ProjectId,
    /// Amount to be paid out
    pub amount: u128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
    /// Memo field for payout notes
    pub memo: Option<String>,
}

/// Ephemeral-only; used for setting payouts
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutInput {
    pub amount: U128,
    pub recipient_id: ProjectId,
    pub memo: Option<String>,
}

/// Ephemeral-only
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PayoutExternal {
    /// Unique identifier for the payout
    pub id: PayoutId,
    pub batch_id: u32,
    /// ID of the application receiving the payout
    pub recipient_id: ProjectId,
    /// Amount to be paid out
    pub amount: U128,
    /// Timestamp when the payout was made. None if not yet paid out.
    pub paid_at: Option<TimestampMs>,
    /// Memo field for payout notes
    pub memo: Option<String>,
}

impl Payout {
    pub fn to_external(&self) -> PayoutExternal {
        PayoutExternal {
            id: self.id.clone(),
            batch_id: self.batch_id,
            recipient_id: self.recipient_id.clone(),
            amount: U128(self.amount),
            paid_at: self.paid_at,
            memo: self.memo.clone(),
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
    /// id of batch being challenged
    pub batch_id: u32,
    /// id of payout being challenged, if any.
    pub payout_id: Option<PayoutId>,
    /// Notes from admin/owner
    pub admin_notes: Option<String>,
    /// Whether the challenge has been resolved
    pub resolved: bool,
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
    pub batch_id: u32,
    pub payout_id: Option<PayoutId>,
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
            batch_id: self.batch_id,
            payout_id: self.payout_id.clone(),
            admin_notes: self.admin_notes.clone(),
            resolved: self.resolved,
        }
    }
}

#[near_bindgen]
impl Contract {
    // set_payouts (callable by chef or admin)
    // allow each payout batch have it's own cooldown setting(wether to allow, and length)
    #[payable]
    pub fn chef_set_payouts(
        &mut self,
        payouts: Vec<PayoutInput>,
        clear_existing: bool,
        batch_memo: Option<String>,
    ) {
        self.assert_admin_or_greater();
        // verify that the round has closed
        // self.assert_round_closed();
        // verify that payouts have not already been processed
        assert!(
            self.all_paid_out == false,
            "Payouts have already been processed"
        );
        let batch_id = self.current_payout_batch_id + 1;
        let now = env::block_timestamp_ms();
        let mut batch = PayoutBatch {
            batch_id,
            created_at: now,
            total_amount: 0,
            status: PayoutBatchStatus::InCooldown,
            cooldown_end_ms: now + self.cooldown_period_ms,
            memo: batch_memo,
        };
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
        let mut payout_ids_for_batch =
            self.payout_ids_by_batch_id
                .get(&batch_id)
                .unwrap_or(UnorderedSet::new(StorageKey::PayoutIdsByBatchIdInner {
                    batch_id,
                }));
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
                "{:?}{}{}",
                payout.recipient_id,
                PAYOUT_ID_DELIMITER,
                payout_ids_for_recipient.len() + 1
            );
            let payout = Payout {
                id: payout_id.clone(),
                amount: payout.amount.0,
                recipient_id: payout.recipient_id.clone(),
                batch_id,
                paid_at: None,
                memo: payout.memo.clone(),
            };
            payout_ids_for_recipient.insert(&payout_id);
            payout_ids_for_batch.insert(&payout_id);
            self.payout_ids_by_recipient_id
                .insert(&payout.recipient_id, &payout_ids_for_recipient);
            self.payouts_by_id.insert(&payout_id, &payout);
        }
        // error if running total is more than matching pool balance
        assert!(
            running_total <= self.spending_balance,
            "Total payouts ({}) must not be greater than matching pool balance ({})",
            running_total,
            self.spending_balance
        );
        batch.total_amount = running_total;
        self.payout_batch_by_id.insert(&batch_id, &batch);
        self.payout_ids_by_batch_id
            .insert(&batch_id, &payout_ids_for_batch);
        self.current_payout_batch_id = batch_id;
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
    pub fn admin_process_payouts(&mut self, batch_id: u32, project_ids: Option<Vec<ProjectId>>) {
        self.assert_admin_or_greater();
        // verify that the cooldown period has passed
        self.assert_cooldown_period_complete();
        // verify that any challenges have been resolved
        self.assert_all_payouts_challenges_resolved();
        let mut batch = self
            .payout_batch_by_id
            .get(&batch_id)
            .expect("No payout batch found");

        assert!(
            batch.status == PayoutBatchStatus::InCooldown,
            "must be in cooldown"
        );
        batch.status = PayoutBatchStatus::ReadyToProcess;
        self.payout_batch_by_id.insert(&batch_id, &batch);
        let batch_payouts = self
            .payout_ids_by_batch_id
            .get(&batch_id)
            .expect("NO payout for Batch");
        // payout batch
        for payout_id in batch_payouts.iter() {
            // batch to avoid OOG?
            let mut payout = self.payouts_by_id.get(&payout_id).expect("missing payout");
            let project_account_id = match &payout.recipient_id {
                ProjectId::Project(acct_id) => acct_id,
                ProjectId::Social { platform, handle } => {
                    &AccountId::new_unchecked("gg.near".to_string())
                }
            };
            if self.is_owner_or_admin(Some(&project_account_id)) {
                log!("Skipping payout for project {} as it is owner, admin or chef and not eligible for payouts.", project_account_id);
            }
            if payout.paid_at.is_none() {
                Promise::new(project_account_id.clone())
                    .transfer(payout.amount)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(XCC_GAS)
                            .transfer_payout_callback(payout, batch_id),
                    );
            }
        }
        // pay out each project
        // for each approved project...
        // loop through self.approved_application_ids set
    }

    // #[payable]
    // pub fn admin_redistribute_matching_pool(&mut self, memo: Option<String>) {
    //     self.assert_admin_or_greater();
    //     // verify that the cooldown period has passed
    //     self.assert_cooldown_period_complete();
    //     // verify that any challenges have been resolved
    //     self.assert_all_payouts_challenges_resolved();
    //     // verify that compliance period has passed
    //     self.assert_compliance_period_complete();
    //     // verify that redistribution is allowed
    //     if !self.allow_remaining_funds_redistribution {
    //         panic!("Redistribution of matching pool is not allowed");
    //     }
    //     // verify that there is a redistribution recipient set
    //     if self.remaining_funds_redistribution_recipient.is_none() {
    //         panic!("No redistribution recipient set");
    //     }
    //     let redistribution_recipient = self.remaining_funds_redistribution_recipient.get().unwrap();
    //     // update matching pool balance (this will be reverted in callback on failure)
    //     let amount = self.matching_pool_balance;
    //     self.matching_pool_balance = 0;
    //     // send matching pool balance to redistribution recipient
    //     Promise::new(redistribution_recipient.clone())
    //         .transfer(amount)
    //         .then(
    //             Self::ext(env::current_account_id()).redistribute_matching_pool_callback(
    //                 amount,
    //                 redistribution_recipient.clone(),
    //                 memo,
    //             ),
    //         );
    // }

    // /// Verifies whether redistribution was successful; reverts matching pool balance if not
    // #[private]
    // pub fn redistribute_matching_pool_callback(
    //     &mut self,
    //     amount: u128,
    //     redistribution_recipient: AccountId,
    //     memo: Option<String>,
    //     #[callback_result] call_result: Result<(), PromiseError>,
    // ) {
    //     if call_result.is_err() {
    //         log!(format!(
    //             "Error redistributing matching pool ({:#?} to recipient {}). Reverting matching pool balance...",
    //             amount, redistribution_recipient
    //         ));
    //         // revert matching pool balance
    //         self.matching_pool_balance += amount;
    //     } else {
    //         log!(format!(
    //             "Successfully redistributed matching pool ({:#?} to recipient {})",
    //             amount, redistribution_recipient
    //         ));
    //         // set remaining_funds_redistributed_at_ms to now
    //         self.remaining_funds_redistributed_at_ms
    //             .set(&env::block_timestamp_ms());
    //         // add memo
    //         if let Some(memo) = memo {
    //             self.remaining_funds_redistribution_memo.set(&memo);
    //         }
    //         // set all_paid_out to true
    //     }
    // }

    // have set person who gets refund
    // refund settings
    // pre-determined entity
    // refunds_available
    // once matching period is over
    // pays out whatever is left after compliance cooldown

    /// Verifies whether payout transfer completed successfully & updates payout record accordingly
    #[private] // Public - but only callable by env::current_account_id()
    pub fn transfer_payout_callback(
        &mut self,
        mut payout: Payout,
        batch_id: u32,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) {
        if call_result.is_err() {
            log!(format!(
                "Error paying out amount {:#?} to recipient {:?}",
                payout.amount, payout.recipient_id
            ));
            // update payout to indicate error transferring funds
            payout.paid_at = None;
            self.payouts_by_id.insert(&payout.id.clone(), &payout);
        } else {
            log!(format!(
                "Successfully paid out amount {:#?} to recipient {:?}",
                payout.amount, payout.recipient_id
            ));
            // decrement matching pool balance
            self.spending_balance -= payout.amount;
            // update payout to indicate that funds transfer has been completed
            payout.paid_at = Some(env::block_timestamp_ms());
            self.payouts_by_id.insert(&payout.id.clone(), &payout);
            self.check_batch_completion(batch_id);
        }
    }

    #[private]
    fn check_batch_completion(&mut self, batch_id: u32) {
        let mut all_processed = true;

        let batch_payouts = self
            .payout_ids_by_batch_id
            .get(&batch_id)
            .expect("NO payout for Batch");

        // Check status of all payouts in batch
        for payout_id in batch_payouts.iter() {
            let payout = self.payouts_by_id.get(&payout_id);
            if payout.is_none() {
                all_processed = false;
                break;
            }
            if payout.unwrap().paid_at.is_none() {
                all_processed = false;
                break;
            }
        }

        // If all payouts are processed, update batch status,
        // status==complete even when all not successful?
        if all_processed {
            let mut batch = self
                .payout_batch_by_id
                .get(&batch_id)
                .expect("Batch not found");
            batch.status = PayoutBatchStatus::Completed;

            self.payout_batch_by_id.insert(&batch_id, &batch);

            log!(
                "Batch {} processing completed with status {:?}",
                batch_id,
                batch.status
            );
        }
    }

    #[payable]
    pub fn challenge_payouts(
        &mut self,
        reason: String,
        batch_id: u32,
        payout_id: Option<PayoutId>,
    ) {
        // anyone can challenge
        // verify that cooldown is in process
        let batch = self
            .payout_batch_by_id
            .get(&batch_id)
            .expect("Batch does not exist?");

        assert!(
            env::block_timestamp_ms() <= batch.cooldown_end_ms,
            "Cooldown period is over"
        );

        // self.assert_cooldown_period_in_process();
        // create challenge & store, charging user for storage
        let initial_storage_usage = env::storage_usage();
        let challenge = PayoutsChallenge {
            created_at: env::block_timestamp_ms(),
            reason,
            batch_id,
            payout_id,
            admin_notes: None,
            resolved: false,
        };

        // store challenge (overwriting any existing challenge for this user - only one challenge per user allowed)
        self.payouts_challenges
            .insert(&env::predecessor_account_id(), &challenge);
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
