use crate::*;

// Donation is the data structure that is stored within the contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Donation {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// Campaign ID
    pub campaign_id: CampaignId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: u128,
    /// Net amount, after all fees & storage costs
    pub net_amount: u128,
    /// FT ID (if None, it's a native NEAR donation)
    pub ft_id: Option<AccountId>,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// Protocol fee
    pub protocol_fee: u128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<u128>,
    /// Whether the donation was returned to sender
    pub returned: bool,
    // /// When donation was actually paid
    // pub paid_at_ms: Option<TimestampMs>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VersionedDonation {
    Current(Donation),
}

impl From<VersionedDonation> for Donation {
    fn from(donation: VersionedDonation) -> Self {
        match donation {
            VersionedDonation::Current(current) => current,
        }
    }
}

/// Ephemeral-only (used in views)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct DonationExternal {
    /// Unique identifier for the donation
    pub id: DonationId,
    /// Campaign ID
    pub campaign_id: CampaignId,
    /// ID of the donor               
    pub donor_id: AccountId,
    /// Amount donated         
    pub total_amount: U128,
    /// Net amount, after all fees & storage costs
    pub net_amount: U128,
    /// FT ID (if None, it's a native NEAR donation)
    pub ft_id: Option<AccountId>,
    /// Optional message from the donor          
    pub message: Option<String>,
    /// Timestamp when the donation was made
    pub donated_at_ms: TimestampMs,
    /// Protocol fee
    pub protocol_fee: U128,
    /// Referrer ID
    pub referrer_id: Option<AccountId>,
    /// Referrer fee
    pub referrer_fee: Option<U128>,
    // /// When donation was actually paid
    // pub paid_at_ms: Option<TimestampMs>,
    /// Whether the donation was returned to sender
    pub returned: bool,
    /// Whether the donation is currently in escrow
    pub is_in_escrow: bool,
    /// ID of the account receiving the donation  
    pub recipient_id: AccountId,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FtReceiverMsg {
    pub campaign_id: CampaignId,
    pub referrer_id: Option<AccountId>,
    pub message: Option<String>,
    pub bypass_protocol_fee: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum TransferType {
    DonationTransfer,
    ProtocolFeeTransfer,
    ReferrerFeeTransfer,
    EscrowTransferDonation,
    EscrowTransferProtocolFee,
    EscrowTransferReferrerFee,
}

#[near_bindgen]
impl Contract {
    /// FT equivalent of donate, for use with FTs that implement NEP-144
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let ft_id = env::predecessor_account_id();
        let msg_json: FtReceiverMsg = near_sdk::serde_json::from_str(&msg)
            .expect("Invalid msg string. Must implement FtReceiverMsg.");
        log!(format!(
            "Campaign ID {:?}, Referrer ID {:?}, Amount {}, Message {:?}",
            msg_json.campaign_id, msg_json.referrer_id, amount.0, msg_json.message
        ));

        // calculate amounts
        let (protocol_fee, referrer_fee, amount_after_fees) = self.calculate_fees_and_remainder(
            amount.0,
            msg_json.campaign_id.clone(),
            msg_json.referrer_id.clone(),
            msg_json.bypass_protocol_fee,
        );

        let net_amount = amount.0 - protocol_fee - referrer_fee.unwrap_or(0);

        // if min_amount present & has not been reached, donation is accepted but not paid out (goes into escrow)
        let mut campaign = Campaign::from(
            self.campaigns_by_id
                .get(&msg_json.campaign_id)
                .expect("Campaign not found"),
        );
        let should_escrow = campaign.net_raised_amount < campaign.min_amount.unwrap_or(0);

        // create and insert donation record
        let initial_storage_usage = env::storage_usage();
        let mut donation = Donation {
            id: self.next_donation_id,
            campaign_id: msg_json.campaign_id.clone(),
            donor_id: env::predecessor_account_id(),
            total_amount: amount.0,
            net_amount,
            ft_id: Some(ft_id.clone()),
            message: msg_json.message,
            donated_at_ms: env::block_timestamp_ms(),
            protocol_fee,
            referrer_id: msg_json.referrer_id.clone(),
            referrer_fee,
            returned: false,
            // paid_at_ms: if should_escrow {
            //     None
            // } else {
            //     Some(env::block_timestamp_ms())
            // },
        };
        self.next_donation_id += 1;
        self.internal_insert_donation_record(&donation, should_escrow);

        // verify and update storage balance for FT donation
        self.verify_and_update_storage_balance(sender_id.clone(), initial_storage_usage);

        if should_escrow {
            log!(format!(
                "Donation {} ({}) accepted but not paid out (escrowed) for campaign {}",
                amount_after_fees, ft_id, msg_json.campaign_id
            ));
            // update campaign (raised_amount & escrow_balance)
            campaign.total_raised_amount += amount.0;
            campaign.net_raised_amount += net_amount;
            campaign.escrow_balance += net_amount;
            self.campaigns_by_id.insert(
                &msg_json.campaign_id,
                &VersionedCampaign::Current(campaign.clone()),
            );
            // log event
            log_donation_event(&self.format_donation(&donation));
            // return # unused tokens as per NEP-144 standard
            return PromiseOrValue::Value(U128(0));
        } else {
            // transfer donation
            log!(format!(
                "Transferring donation {} to {}",
                amount_after_fees,
                campaign.recipient.clone()
            ));
            self.handle_transfer_donation(self.format_donation(&donation), amount_after_fees);
            // * NB: fees will be transferred in transfer_funds_callback after successful transfer of donation

            // return # unused tokens as per NEP-144 standard
            PromiseOrValue::Value(U128(0))
        }
    }

    #[payable]
    pub fn donate(
        &mut self,
        campaign_id: CampaignId,
        message: Option<String>,
        referrer_id: Option<AccountId>,
        bypass_protocol_fee: Option<bool>,
    ) -> PromiseOrValue<DonationExternal> {
        /*
        DONATION SCENARIOS
        1. Campaign is not live (before start or after end, or max_amount reached), donate call fails
        2. Campaign is live, min_amount present & has not been reached, donation is accepted but not paid out (goes into escrow)
        3. Campaign is live, no min_amount or min_amount has been reached, donation is accepted and paid out

         */
        self.assert_campaign_live(&campaign_id);
        // calculate amounts
        let amount = env::attached_deposit();
        let (protocol_fee, referrer_fee, mut amount_after_fees) = self
            .calculate_fees_and_remainder(
                amount.clone(),
                campaign_id.clone(),
                referrer_id.clone(),
                bypass_protocol_fee,
            );
        let net_amount = amount - protocol_fee - referrer_fee.unwrap_or(0);

        // if min_amount present & has not been reached, donation is accepted but not paid out (goes into escrow)
        let mut campaign = Campaign::from(
            self.campaigns_by_id
                .get(&campaign_id)
                .expect("Campaign not found"),
        );
        let should_escrow = campaign.net_raised_amount < campaign.min_amount.unwrap_or(0);

        // create and insert donation record
        let initial_storage_usage = env::storage_usage();
        let mut donation = Donation {
            id: self.next_donation_id,
            campaign_id,
            donor_id: env::predecessor_account_id(),
            total_amount: amount,
            net_amount,
            ft_id: None,
            message,
            donated_at_ms: env::block_timestamp_ms(),
            protocol_fee,
            referrer_id,
            referrer_fee,
            returned: false,
            // paid_at_ms: if should_escrow {
            //     None
            // } else {
            //     Some(env::block_timestamp_ms())
            // },
        };
        self.next_donation_id += 1;
        self.internal_insert_donation_record(&donation, should_escrow);

        // assert that donation after fees > storage cost
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        require!(
            amount_after_fees > required_deposit,
            format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            )
        );
        amount_after_fees -= required_deposit;

        // update net_amount with storage taken out
        donation.net_amount = net_amount - required_deposit;
        self.donations_by_id
            .insert(&donation.id, &VersionedDonation::Current(donation.clone()));

        if should_escrow {
            log!(format!(
                "Donation {} accepted but not paid out (escrowed) for campaign {}",
                amount_after_fees, campaign_id
            ));
            // update campaign (raised_amount & escrow_balance)
            campaign.total_raised_amount += amount;
            campaign.net_raised_amount += net_amount;
            campaign.escrow_balance += net_amount;
            self.campaigns_by_id
                .insert(&campaign_id, &VersionedCampaign::Current(campaign.clone()));
            // log event
            log_donation_event(&self.format_donation(&donation));
            return PromiseOrValue::Value(self.format_donation(&donation));
        } else {
            // transfer donation
            log!(format!(
                "Transferring donation {} to {}",
                amount_after_fees,
                campaign.recipient.clone()
            ));
            self.handle_transfer_donation(self.format_donation(&donation), amount_after_fees)
            // * NB: fees will be transferred in transfer_funds_callback after successful transfer of donation
        }
    }

    pub(crate) fn calculate_fees_and_remainder(
        &self,
        amount: u128,
        campaign_id: CampaignId,
        referrer_id: Option<AccountId>,
        bypass_protocol_fee: Option<bool>,
    ) -> (u128, Option<u128>, u128) {
        // calculate protocol fee
        let mut remainder = amount;
        let protocol_fee = if bypass_protocol_fee.unwrap_or(false) {
            0
        } else {
            self.calculate_protocol_fee(amount)
        };

        remainder -= protocol_fee;

        // calculate referrer fee, if applicable
        let mut referrer_fee = None;
        if let Some(_referrer_id) = referrer_id.clone() {
            let campaign = Campaign::from(
                self.campaigns_by_id
                    .get(&campaign_id)
                    .expect("Campaign not found"),
            );
            let referrer_amount =
                self.calculate_referrer_fee(amount, campaign.referral_fee_basis_points);
            remainder -= referrer_amount;
            referrer_fee = Some(referrer_amount);
        }

        (protocol_fee, referrer_fee, remainder)
    }

    pub(crate) fn verify_and_update_storage_balance(
        &mut self,
        sender_id: AccountId,
        initial_storage_usage: u64,
    ) {
        // verify that deposit is sufficient to cover storage
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        let storage_balance = self.storage_balance_of(&sender_id);
        assert!(
            storage_balance.0 >= required_deposit,
            "{} must add storage deposit of at least {} yoctoNEAR to cover Donation storage",
            sender_id,
            required_deposit
        );

        log!("Old storage balance: {}", storage_balance.0);
        // deduct storage deposit from user's balance
        let new_storage_balance = storage_balance.0 - required_deposit;
        self.storage_deposits
            .insert(&sender_id, &new_storage_balance);
        log!("New storage balance: {}", new_storage_balance);
        log!(format!(
            "Deducted {} yoctoNEAR from {}'s storage balance to cover storage",
            required_deposit, sender_id
        ));
    }

    pub(crate) fn handle_transfer(
        &self,
        donation: DonationExternal,
        amount_after_fees: Balance,
        transfer_type: TransferType,
    ) -> PromiseOrValue<DonationExternal> {
        if let Some(ft_id) = donation.ft_id {
            // FT donation
            let ft_transfer_args =
                json!({ "receiver_id": donation.recipient_id, "amount": U128(amount_after_fees) })
                    .to_string()
                    .into_bytes();
            PromiseOrValue::Promise(
                Promise::new(ft_id.clone())
                    .function_call(
                        "ft_transfer".to_string(),
                        ft_transfer_args,
                        ONE_YOCTO,
                        Gas(XCC_GAS_DEFAULT),
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(XCC_GAS_DEFAULT))
                            .transfer_funds_callback(
                                amount_after_fees,
                                donation.clone(),
                                transfer_type,
                                None,
                            ),
                    ),
            )
        } else {
            // native NEAR donation
            PromiseOrValue::Promise(
                Promise::new(donation.recipient_id)
                    .transfer(amount_after_fees)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(Gas(XCC_GAS_DEFAULT))
                            .transfer_funds_callback(
                                amount_after_fees,
                                donation.clone(),
                                transfer_type,
                                None,
                            ),
                    ),
            )
        }
    }

    pub(crate) fn handle_transfer_donation(
        &self,
        donation: DonationExternal,
        amount_after_fees: Balance,
    ) -> PromiseOrValue<DonationExternal> {
        self.handle_transfer(donation, amount_after_fees, TransferType::DonationTransfer)
    }

    pub(crate) fn handle_transfer_protocol_fee(
        &self,
        donation: DonationExternal,
        amount_after_fees: Balance,
    ) -> PromiseOrValue<DonationExternal> {
        self.handle_transfer(
            donation,
            amount_after_fees,
            TransferType::ProtocolFeeTransfer,
        )
    }

    pub(crate) fn handle_transfer_referrer_fee(
        &self,
        donation: DonationExternal,
        amount_after_fees: Balance,
    ) -> PromiseOrValue<DonationExternal> {
        self.handle_transfer(
            donation,
            amount_after_fees,
            TransferType::ReferrerFeeTransfer,
        )
    }

    /// Verifies whether donation & fees have been paid out for a given donation
    #[private]
    pub fn transfer_funds_callback(
        &mut self,
        amount: Balance,
        mut donation: DonationExternal,
        transfer_type: TransferType,
        donation_ids: Option<Vec<DonationId>>,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Option<DonationExternal> {
        let is_ft_transfer = donation.ft_id.is_some();
        if call_result.is_err() {
            // ERROR CASE HANDLING
            // TODO: HANDLE REVERT PAID_AT_MS ON DONATION
            // 1. If donation transfer failed, delete Donation record and return all funds to donor. NB: fees have not been transferred yet.
            // 2. If protocol fee transfer failed, update donation record to indicate protocol fee of "0". NB: donation has already been transferred to recipient and this cannot be reversed.
            // 3. If referrer fee transfer failed, update donation record to indicate referrer fee of "0". NB: donation has already been transferred to recipient and this cannot be reversed.
            match transfer_type {
                TransferType::DonationTransfer => {
                    log!(format!(
                        "Error transferring donation {:?} to {}. Returning funds to donor.",
                        donation.total_amount, donation.recipient_id
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.total_amount.clone() })
                                .to_string()
                                .into_bytes();
                        Promise::new(donation.ft_id.unwrap()).function_call(
                            "ft_transfer".to_string(),
                            donation_transfer_args,
                            ONE_YOCTO,
                            Gas(XCC_GAS_DEFAULT),
                        );
                    } else {
                        Promise::new(donation.donor_id.clone()).transfer(donation.total_amount.0);
                    }
                    // delete donation record, and refund freed storage cost to donor's storage balance
                    let initial_storage_usage = env::storage_usage();
                    self.internal_remove_donation_record(&self.unformat_donation(&donation));
                    let storage_freed = initial_storage_usage - env::storage_usage();
                    let cost_freed = env::storage_byte_cost() * Balance::from(storage_freed);
                    let storage_balance = self.storage_balance_of(&donation.donor_id);
                    let new_storage_balance = storage_balance.0 + cost_freed;
                    log!("Old storage balance: {}", storage_balance.0);
                    log!("New storage balance: {}", new_storage_balance);
                    self.storage_deposits
                        .insert(&donation.donor_id, &new_storage_balance); // TODO: check if this is hackable, e.g. if user can withdraw all their storage before this callback runs and therefore get a higher refund
                    log!(format!(
                        "Refunded {} yoctoNEAR to {}'s storage balance for freed storage",
                        cost_freed, donation.donor_id
                    ));
                    None
                }
                TransferType::EscrowTransferDonation => {
                    // transfer of escrowed donations to recipient failed
                    // TODO: how to handle this case?
                    None
                }
                TransferType::ProtocolFeeTransfer => {
                    log!(format!(
                        "Error transferring protocol fee {:?} to {}. Returning funds to donor.",
                        donation.protocol_fee, self.protocol_fee_recipient_account
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.protocol_fee })
                                .to_string()
                                .into_bytes();
                        Promise::new(donation.ft_id.unwrap()).function_call(
                            "ft_transfer".to_string(),
                            donation_transfer_args,
                            ONE_YOCTO,
                            Gas(XCC_GAS_DEFAULT),
                        );
                    } else {
                        Promise::new(donation.donor_id.clone()).transfer(donation.protocol_fee.0);
                    }
                    // update fee on Donation record to indicate error transferring funds
                    donation.protocol_fee = U128(0);
                    self.donations_by_id.insert(
                        &donation.id.clone(),
                        &VersionedDonation::Current(self.unformat_donation(&donation)),
                    );
                    Some(donation)
                }
                TransferType::EscrowTransferProtocolFee => {
                    // transfer of protocol fee for escrowed donations failed
                    // TODO: how to handle this case?
                    None
                }
                TransferType::ReferrerFeeTransfer => {
                    log!(format!(
                        "Error transferring referrer fee {:?} to {:?}. Returning funds to donor.",
                        donation.referrer_fee, donation.referrer_id
                    ));
                    // return funds to donor
                    if is_ft_transfer {
                        let donation_transfer_args =
                            json!({ "receiver_id": donation.donor_id, "amount": donation.referrer_fee.unwrap() })
                                .to_string()
                                .into_bytes();
                        Promise::new(donation.ft_id.unwrap()).function_call(
                            "ft_transfer".to_string(),
                            donation_transfer_args,
                            ONE_YOCTO,
                            Gas(XCC_GAS_DEFAULT),
                        );
                    } else {
                        Promise::new(donation.donor_id.clone())
                            .transfer(donation.referrer_fee.unwrap().0);
                    }
                    // update fee on Donation record to indicate error transferring funds
                    donation.referrer_fee = Some(U128(0));
                    self.donations_by_id.insert(
                        &donation.id.clone(),
                        &VersionedDonation::Current(self.unformat_donation(&donation)),
                    );
                    Some(donation)
                }
                TransferType::EscrowTransferReferrerFee => {
                    // transfer of referrer fee for escrowed donations failed
                    // TODO: how to handle this case?
                    None
                }
            }
        } else {
            // SUCCESS CASE HANDLING
            if transfer_type == TransferType::EscrowTransferDonation {
                log!(format!(
                    "Successfully transferred donation {} out of escrow to {}!",
                    amount, donation.recipient_id
                ));
                // reduce campaign.escrow_balance by amount
                let mut campaign = Campaign::from(
                    self.campaigns_by_id
                        .get(&donation.campaign_id)
                        .expect("Campaign not found"),
                );
                campaign.escrow_balance -= amount;
                self.campaigns_by_id
                    .insert(&donation.campaign_id, &VersionedCampaign::Current(campaign));

                // remove donation IDs from escrowed set...
                let mut escrowed_donation_ids_by_campaign = self
                    .escrowed_donation_ids_by_campaign_id
                    .get(&donation.campaign_id)
                    .expect("Campaign not found");
                for donation_id in donation_ids.expect("No donation IDs provided") {
                    escrowed_donation_ids_by_campaign.remove(&donation_id);
                }
                self.escrowed_donation_ids_by_campaign_id
                    .insert(&donation.campaign_id, &escrowed_donation_ids_by_campaign);

                // ...and add them to unescrowed set
                let mut unescrowed_donation_ids_by_campaign = self
                    .unescrowed_donation_ids_by_campaign_id
                    .get(&donation.campaign_id)
                    .expect("Campaign not found");
                for donation_id in donation_ids.expect("No donation IDs provided") {
                    unescrowed_donation_ids_by_campaign.remove(&donation_id);
                }
                self.unescrowed_donation_ids_by_campaign_id
                    .insert(&donation.campaign_id, &unescrowed_donation_ids_by_campaign);

                // no need to transfer protocol fee or referrer fee for escrowed donations as this is handled separately

                Some(donation)
            } else if transfer_type == TransferType::DonationTransfer {
                log!(format!(
                    "Successfully transferred donation {} to {}!",
                    amount, donation.recipient_id
                ));

                // transfer protocol fee
                if donation.protocol_fee.0 > 0 {
                    log!(format!(
                        "Transferring protocol fee {:?} to {}",
                        donation.protocol_fee, self.protocol_fee_recipient_account
                    ));
                    self.handle_transfer_protocol_fee(donation.clone(), amount);
                }

                // transfer referrer fee
                if let (Some(referrer_fee), Some(referrer_id)) =
                    (donation.referrer_fee.clone(), donation.referrer_id.clone())
                {
                    if referrer_fee.0 > 0 {
                        log!(format!(
                            "Transferring referrer fee {:?} to {}",
                            referrer_fee, referrer_id
                        ));
                        self.handle_transfer_referrer_fee(donation.clone(), amount);
                    }
                }

                // if this donation is the one that meets or exceeds the min_amount for campaign, trigger payouts for all escrowed donations
                let mut campaign = Campaign::from(
                    self.campaigns_by_id
                        .get(&donation.campaign_id)
                        .expect("Campaign not found"),
                );
                if campaign.net_raised_amount < campaign.min_amount.unwrap_or(0)
                    && campaign.net_raised_amount + amount >= campaign.min_amount.unwrap()
                {
                    log!(format!(
                        "Campaign {} has reached min_amount! Triggering payouts for all escrowed donations.",
                        donation.campaign_id
                    ));
                    // self.trigger_payouts_for_campaign(&donation.campaign_id);
                    // get donation IDs for campaign
                    // consolidate recipients and amounts into HashMap (campaign recipient, protocol fee recipient, referrer)
                    // transfer funds to recipients
                    // update donation records
                    // log events
                    let donation_ids = self
                        .escrowed_donation_ids_by_campaign_id
                        .get(&donation.campaign_id)
                        .expect("Campaign not found"); // NB: this set will include the current donation
                                                       // let mut recipients: HashMap<AccountId, Balance> = HashMap::new();
                    let mut recipient_balance: Balance = 0;
                    let mut protocol_fee_recipient_balance: Balance = 0;
                    let mut referrers_balances: HashMap<AccountId, Balance> = HashMap::new();
                    for donation_id in donation_ids.iter() {
                        let donation = Donation::from(
                            self.donations_by_id
                                .get(&donation_id)
                                .expect("Donation not found"),
                        );
                        let amount = donation.net_amount;
                        recipient_balance += amount;
                        if donation.protocol_fee > 0 {
                            protocol_fee_recipient_balance += donation.protocol_fee;
                        }
                        if let Some(referrer_id) = donation.referrer_id {
                            if let Some(referrer_fee) = donation.referrer_fee {
                                referrers_balances
                                    .entry(referrer_id)
                                    .and_modify(|v| *v += referrer_fee)
                                    .or_insert(referrer_fee);
                            }
                        }
                    }
                    // transfer funds to recipients
                    if recipient_balance > 0 {
                        log!(format!(
                            "Transferring {} to campaign recipient {} for campaign {}",
                            recipient_balance, campaign.recipient, donation.campaign_id
                        ));
                        Promise::new(campaign.recipient.clone())
                            .transfer(recipient_balance)
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                                    .transfer_funds_callback(
                                        recipient_balance,
                                        donation.clone(),
                                        TransferType::EscrowTransferDonation,
                                        Some(donation_ids.to_vec()),
                                    ),
                            );
                    }
                    if protocol_fee_recipient_balance > 0 {
                        log!(format!(
                            "Transferring {} to protocol fee recipient {} for campaign {}",
                            protocol_fee_recipient_balance,
                            self.protocol_fee_recipient_account,
                            donation.campaign_id
                        ));
                        Promise::new(self.protocol_fee_recipient_account.clone())
                            .transfer(protocol_fee_recipient_balance)
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                                    .transfer_funds_callback(
                                        protocol_fee_recipient_balance,
                                        donation.clone(),
                                        TransferType::EscrowTransferProtocolFee,
                                        None,
                                    ),
                            );
                    }
                    for (referrer, amount) in referrers_balances.iter() {
                        if amount > &0 {
                            log!(format!(
                                "Transferring {} to referrer {} for campaign {}",
                                amount, referrer, donation.campaign_id
                            ));
                            Promise::new(referrer.clone()).transfer(*amount).then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(Gas(XCC_GAS_DEFAULT))
                                    .transfer_funds_callback(
                                        *amount,
                                        donation.clone(),
                                        TransferType::EscrowTransferReferrerFee,
                                        None,
                                    ),
                            );
                        }
                    }
                }

                // log event indicating successful donation/transfer!
                log_donation_event(&donation);

                // return donation
                Some(donation)
            } else {
                None
            }
        }
    }

    pub(crate) fn calculate_protocol_fee(&self, amount: u128) -> u128 {
        let total_basis_points = 10_000u128;
        let fee_amount = (self.protocol_fee_basis_points as u128).saturating_mul(amount);
        // Round up
        fee_amount.div_ceil(total_basis_points)
    }

    pub(crate) fn calculate_referrer_fee(
        &self,
        amount: u128,
        referral_fee_basis_points: u32,
    ) -> u128 {
        let total_basis_points = 10_000u128;
        let fee_amount = (referral_fee_basis_points as u128).saturating_mul(amount);
        // Round down
        fee_amount / total_basis_points
    }

    pub(crate) fn internal_insert_donation_record(&mut self, donation: &Donation, escrow: bool) {
        // add to donations-by-id mapping
        self.donations_by_id
            .insert(&donation.id, &VersionedDonation::Current(donation.clone()));

        // add to appropriate donations-by-campaign mapping, according to whether donation is escrowed or not
        if escrow {
            let mut donation_ids_by_campaign_set = if let Some(donation_ids_by_campaign_set) = self
                .escrowed_donation_ids_by_campaign_id
                .get(&donation.campaign_id)
            {
                donation_ids_by_campaign_set
            } else {
                UnorderedSet::new(StorageKey::EscrowedDonationIdsByCampaignIdInner {
                    campaign_id: donation.campaign_id.clone(),
                })
            };
            donation_ids_by_campaign_set.insert(&donation.id);
            self.escrowed_donation_ids_by_campaign_id
                .insert(&donation.campaign_id, &donation_ids_by_campaign_set);
        } else {
            let mut donation_ids_by_campaign_set = if let Some(donation_ids_by_campaign_set) = self
                .unescrowed_donation_ids_by_campaign_id
                .get(&donation.campaign_id)
            {
                donation_ids_by_campaign_set
            } else {
                UnorderedSet::new(StorageKey::UnescrowedDonationIdsByCampaignIdInner {
                    campaign_id: donation.campaign_id.clone(),
                })
            };
            donation_ids_by_campaign_set.insert(&donation.id);
            self.unescrowed_donation_ids_by_campaign_id
                .insert(&donation.campaign_id, &donation_ids_by_campaign_set);
        }

        // add to donations-by-donor mapping
        let mut donation_ids_by_donor_set = if let Some(donation_ids_by_donor_set) =
            self.donation_ids_by_donor_id.get(&donation.donor_id)
        {
            donation_ids_by_donor_set
        } else {
            UnorderedSet::new(StorageKey::DonationIdsByDonorIdInner {
                donor_id: donation.donor_id.clone(),
            })
        };
        donation_ids_by_donor_set.insert(&donation.id);
        self.donation_ids_by_donor_id
            .insert(&donation.donor_id, &donation_ids_by_donor_set);
    }

    pub(crate) fn internal_remove_donation_record(&mut self, donation: &Donation) {
        // remove from donations-by-id mapping
        self.donations_by_id.remove(&donation.id);

        // remove from donations-by-campaign mappings
        let mut escrowed_donation_ids_by_campaign_set = self
            .escrowed_donation_ids_by_campaign_id
            .get(&donation.campaign_id)
            .expect("Campaign not found");
        escrowed_donation_ids_by_campaign_set.remove(&donation.id);
        self.escrowed_donation_ids_by_campaign_id.insert(
            &donation.campaign_id,
            &escrowed_donation_ids_by_campaign_set,
        );

        let mut unescrowed_donation_ids_by_campaign_set = self
            .unescrowed_donation_ids_by_campaign_id
            .get(&donation.campaign_id)
            .expect("Campaign not found");
        unescrowed_donation_ids_by_campaign_set.remove(&donation.id);
        self.unescrowed_donation_ids_by_campaign_id.insert(
            &donation.campaign_id,
            &unescrowed_donation_ids_by_campaign_set,
        );

        // remove from donations-by-donor mapping
        let mut donation_ids_by_donor_set = self
            .donation_ids_by_donor_id
            .get(&donation.donor_id)
            .expect("Donor not found");
        donation_ids_by_donor_set.remove(&donation.id);
        self.donation_ids_by_donor_id
            .insert(&donation.donor_id, &donation_ids_by_donor_set);
    }

    // GETTERS
    // get_donations
    // get_matching_pool_balance
    pub fn get_donations(
        &self,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        self.donations_by_id
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|(_, v)| self.format_donation(&Donation::from(v)))
            .collect()
    }

    pub fn get_donation_by_id(&self, donation_id: DonationId) -> Option<DonationExternal> {
        self.donations_by_id
            .get(&donation_id)
            .map(|v| self.format_donation(&Donation::from(v)))
    }

    pub fn get_donations_for_campaign(
        &self,
        campaign_id: CampaignId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        assert!(
            (self.donations_by_id.len() as u128) >= start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let escrowed_donation_ids_by_campaign_set =
            self.escrowed_donation_ids_by_campaign_id.get(&campaign_id);
        let escrowed_donation_ids_vec = if let Some(escrowed_donation_ids_by_campaign_set) =
            escrowed_donation_ids_by_campaign_set
        {
            escrowed_donation_ids_by_campaign_set.to_vec()
        } else {
            vec![]
        };
        let unescrowed_donation_ids_by_campaign_set = self
            .unescrowed_donation_ids_by_campaign_id
            .get(&campaign_id);
        let unescrowed_donation_ids_vec = if let Some(unescrowed_donation_ids_by_campaign_set) =
            unescrowed_donation_ids_by_campaign_set
        {
            unescrowed_donation_ids_by_campaign_set.to_vec()
        } else {
            vec![]
        };

        // combine both vecs
        let mut donation_ids_by_campaign = escrowed_donation_ids_vec;
        donation_ids_by_campaign.extend(unescrowed_donation_ids_vec);
        donation_ids_by_campaign
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .map(|donation_id| {
                self.format_donation(&Donation::from(
                    self.donations_by_id.get(&donation_id).unwrap(),
                ))
            })
            .collect()
    }

    pub fn get_donations_for_donor(
        &self,
        donor_id: AccountId,
        from_index: Option<u128>,
        limit: Option<u64>,
    ) -> Vec<DonationExternal> {
        let start_index: u128 = from_index.unwrap_or_default();
        // TODO: ADD BELOW BACK IN
        // assert!(
        //     (self.donations_by_id.len() as u128) >= start_index,
        //     "Out of bounds, please use a smaller from_index."
        // );
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let donation_ids_by_donor_set = self.donation_ids_by_donor_id.get(&donor_id);
        if let Some(donation_ids_by_donor_set) = donation_ids_by_donor_set {
            donation_ids_by_donor_set
                .iter()
                .skip(start_index as usize)
                .take(limit)
                .map(|donation_id| {
                    self.format_donation(&Donation::from(
                        self.donations_by_id.get(&donation_id).unwrap(),
                    ))
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub(crate) fn format_donation(&self, donation: &Donation) -> DonationExternal {
        let campaign = Campaign::from(
            self.campaigns_by_id
                .get(&donation.campaign_id)
                .expect("Campaign not found"),
        );
        DonationExternal {
            id: donation.id,
            campaign_id: donation.campaign_id.clone(),
            donor_id: donation.donor_id.clone(),
            recipient_id: campaign.recipient.clone(),
            total_amount: U128(donation.total_amount),
            net_amount: U128(donation.net_amount),
            ft_id: donation.ft_id.clone(),
            message: donation.message.clone(),
            donated_at_ms: donation.donated_at_ms,
            protocol_fee: U128(donation.protocol_fee),
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.map(|v| U128(v)),
            // paid_at_ms: donation.paid_at_ms,
            // returned_at: donation.returned_at,
            returned: donation.returned,
            is_in_escrow: self
                .escrowed_donation_ids_by_campaign_id
                .get(&donation.campaign_id)
                .map(|v| v.contains(&donation.id))
                .unwrap(),
        }
    }

    pub(crate) fn unformat_donation(&self, donation: &DonationExternal) -> Donation {
        Donation {
            id: donation.id,
            campaign_id: donation.campaign_id.clone(),
            donor_id: donation.donor_id.clone(),
            total_amount: donation.total_amount.0,
            net_amount: 0,
            ft_id: donation.ft_id.clone(),
            message: donation.message.clone(),
            donated_at_ms: donation.donated_at_ms,
            protocol_fee: donation.protocol_fee.0,
            referrer_id: donation.referrer_id.clone(),
            referrer_fee: donation.referrer_fee.map(|v| v.0),
            // paid_at_ms: donation.paid_at_ms,
            // returned_at: donation.returned_at,
            returned: donation.returned,
        }
    }
}
