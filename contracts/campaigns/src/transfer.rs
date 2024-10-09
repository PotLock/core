use crate::*;

#[near]
impl Contract {
    // pub(crate) fn handle_transfer_recipient_amount(
    //     &self,
    //     donation: DonationExternal,
    // ) -> PromiseOrValue<DonationExternal> {
    //     self.handle_transfer(donation, FundsReceiver::Recipient, donation.net_amount, )
    // }

    // pub(crate) fn handle_transfer_protocol_fee(
    //     &self,
    //     donation: DonationExternal,
    // ) -> PromiseOrValue<DonationExternal> {
    //     self.handle_transfer(donation, FundsReceiver::Protocol)
    // }

    // pub(crate) fn handle_transfer_referrer_fee(
    //     &self,
    //     donation: DonationExternal,
    // ) -> PromiseOrValue<DonationExternal> {
    //     self.handle_transfer(donation, FundsReceiver::Referrer)
    // }

    // pub(crate) fn handle_transfer_creator_fee(
    //     &self,
    //     donation: DonationExternal,
    // ) -> PromiseOrValue<DonationExternal> {
    //     self.handle_transfer(donation, FundsReceiver::Creator)
    // }

    /// Handles transfer of one component (e.g. recipient amount, protocol fee, referrer fee or creator fee) of a single donation
    pub(crate) fn handle_transfer(
        &self,
        donation: DonationExternal,
        receiver_type: FundsReceiver,
        amount: U128,
        recipient_id: AccountId,
    ) -> PromiseOrValue<DonationExternal> {
        PromiseOrValue::Promise(
            self.internal_transfer_amount(amount.into(), recipient_id, donation.ft_id.clone())
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(Gas::from_tgas(XCC_GAS_DEFAULT))
                        .transfer_funds_callback(amount.into(), donation.clone(), receiver_type),
                ),
        )
    }

    /// Verifies whether donation & fees have been paid out for a given donation
    #[private]
    pub fn transfer_funds_callback(
        &mut self,
        amount: Balance,
        mut donation: DonationExternal,
        receiver_type: FundsReceiver,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> Option<DonationExternal> {
        if call_result.is_err() {
            // * ERROR CASE HANDLING
            match receiver_type {
                FundsReceiver::Recipient => {
                    // Campain recipient transfer failed
                    // Remove donation record & transfer full donation amount back to donor
                    log!(
                        "{}",
                        format!(
                            "Error transferring donation {:?} to {}. Returning funds to donor.",
                            donation.total_amount, donation.recipient_id
                        )
                    );
                    // return funds to donor
                    self.internal_transfer_amount(
                        donation.total_amount.0,
                        donation.donor_id.clone(),
                        donation.ft_id.clone(),
                    );
                    // delete donation record, and refund freed storage cost to donor's storage balance
                    let initial_storage_usage = env::storage_usage();
                    self.internal_remove_donation_record(&self.unformat_donation(&donation));
                    // Refund cost of storage freed directly to donor. (No need to keep it in user's storage_deposit balance, this just creates an extra step for them to withdraw it.)
                    let storage_freed = initial_storage_usage - env::storage_usage();
                    let cost_freed =
                        env::storage_byte_cost().as_yoctonear() * Balance::from(storage_freed);
                    self.internal_transfer_amount(cost_freed, donation.donor_id.clone(), None);
                    None
                }
                FundsReceiver::Protocol => {
                    // Protocol fee transfer failed
                    // Return protocol fee to donor & update donation record to indicate protocol fee of 0
                    log!(
                        "{}",
                        format!(
                            "Error transferring protocol fee {:?} to {}. Returning funds to donor.",
                            donation.protocol_fee, self.protocol_fee_recipient_account
                        )
                    );
                    // return funds to donor
                    self.internal_transfer_amount(
                        donation.protocol_fee.0,
                        donation.donor_id.clone(),
                        donation.ft_id.clone(),
                    );
                    // update protocol fee on Donation record to indicate error transferring funds
                    donation.protocol_fee = U128(0);
                    self.donations_by_id.insert(
                        donation.id.clone(),
                        VersionedDonation::Current(self.unformat_donation(&donation)),
                    );
                    Some(donation)
                }
                FundsReceiver::Referrer => {
                    log!(
                        "{}",
                        format!(
                        "Error transferring referrer fee {:?} to {:?}. Returning funds to donor.",
                        donation.referrer_fee, donation.referrer_id
                    )
                    );
                    // return funds to donor
                    self.internal_transfer_amount(
                        donation.referrer_fee.unwrap().0,
                        donation.donor_id.clone(),
                        donation.ft_id.clone(),
                    );
                    // update referrer fee on Donation record to indicate error transferring funds
                    donation.referrer_fee = Some(U128(0));
                    self.donations_by_id.insert(
                        donation.id.clone(),
                        VersionedDonation::Current(self.unformat_donation(&donation)),
                    );
                    Some(donation)
                }
                FundsReceiver::Creator => {
                    log!(
                        "{}",
                        format!(
                            "Error transferring creator fee {:?} to {}. Returning funds to donor.",
                            donation.creator_fee, donation.recipient_id
                        )
                    );
                    // return funds to donor
                    self.internal_transfer_amount(
                        donation.creator_fee.0,
                        donation.donor_id.clone(),
                        donation.ft_id.clone(),
                    );
                    // update fee on Donation record to indicate error transferring funds
                    donation.creator_fee = U128(0);
                    self.donations_by_id.insert(
                        donation.id.clone(),
                        VersionedDonation::Current(self.unformat_donation(&donation)),
                    );
                    Some(donation)
                }
            }
        } else {
            // * SUCCESS CASE HANDLING
            // NB: escrow transfers are handled in transfer_escrowed_donations_callback, which is written to handle multiple donations
            if receiver_type == FundsReceiver::Recipient {
                log!(
                    "{}",
                    format!(
                        "Successfully transferred donation {} to {}!",
                        amount, donation.recipient_id
                    )
                );

                // transfer protocol fee
                if donation.protocol_fee.0 > 0 {
                    log!(
                        "{}",
                        format!(
                            "Transferring protocol fee {:?} to {}",
                            donation.protocol_fee, self.protocol_fee_recipient_account
                        )
                    );
                    self.handle_transfer(
                        donation.clone(),
                        FundsReceiver::Protocol,
                        donation.protocol_fee,
                        self.protocol_fee_recipient_account.clone(),
                    );
                }

                // transfer referrer fee
                if let (Some(referrer_fee), Some(referrer_id)) =
                    (donation.referrer_fee.clone(), donation.referrer_id.clone())
                {
                    if referrer_fee.0 > 0 {
                        log!(
                            "{}",
                            format!(
                                "Transferring referrer fee {:?} to {}",
                                referrer_fee, referrer_id
                            )
                        );
                        self.handle_transfer(
                            donation.clone(),
                            FundsReceiver::Referrer,
                            referrer_fee,
                            referrer_id,
                        );
                    }
                }

                // transfer creator fee
                if donation.creator_fee.0 > 0 {
                    let campaign = Campaign::from(
                        self.campaigns_by_id
                            .get(&donation.campaign_id)
                            .expect("Campaign not found")
                            .clone(),
                    );
                    let recipient_id = campaign.owner;
                    log!(
                        "{}",
                        format!(
                            "Transferring creator fee {:?} to {}",
                            donation.creator_fee, recipient_id
                        )
                    );
                    self.handle_transfer(
                        donation.clone(),
                        FundsReceiver::Creator,
                        donation.creator_fee,
                        recipient_id,
                    );
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
}
