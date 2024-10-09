use crate::*;

pub fn calculate_required_storage_deposit(initial_storage_usage: u64) -> Balance {
    let storage_used = env::storage_usage() - initial_storage_usage;
    log!("Storage used: {} bytes", storage_used);
    let required_cost = env::storage_byte_cost().as_yoctonear() * Balance::from(storage_used);
    required_cost
}

pub fn refund_deposit(initial_storage_usage: u64) {
    let attached_deposit = env::attached_deposit().as_yoctonear();
    let mut refund = attached_deposit;
    if env::storage_usage() > initial_storage_usage {
        // caller should pay for the extra storage they used and be refunded for the rest
        // let storage_used = env::storage_usage() - initial_storage_usage;
        let required_deposit = calculate_required_storage_deposit(initial_storage_usage);
        // env::storage_byte_cost() * Balance::from(storage_used);
        require!(
            required_deposit <= attached_deposit,
            format!(
                "Must attach {} yoctoNEAR to cover storage",
                required_deposit
            )
        );
        refund -= required_deposit;
    } else {
        // storage was freed up; caller should be refunded for what they freed up, in addition to the deposit they sent
        let storage_freed = initial_storage_usage - env::storage_usage();
        let cost_freed = env::storage_byte_cost().as_yoctonear() * Balance::from(storage_freed);
        refund += cost_freed;
    }
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund));
    }
}

pub(crate) fn calculate_fee(amount: u128, basis_points: u32) -> u128 {
    let total_basis_points = 10_000u128;
    let fee_amount = (basis_points as u128).saturating_mul(amount);
    // Round down
    fee_amount / total_basis_points
}

#[near]
impl Contract {
    pub(crate) fn calculate_protocol_fee(&self, amount: u128) -> u128 {
        calculate_fee(amount, self.protocol_fee_basis_points)
    }

    pub(crate) fn calculate_fees_and_remainder(
        &self,
        amount: u128,
        campaign: &Campaign,
        referrer_id: Option<AccountId>,
        bypass_protocol_fee: Option<bool>,
        bypass_creator_fee: Option<bool>,
    ) -> (u128, Option<u128>, u128, u128) {
        // calculate protocol fee
        let mut remainder = amount;
        let user_can_avoid_fees = campaign.allow_fee_avoidance;
        let protocol_fee = if bypass_protocol_fee.unwrap_or(false) && user_can_avoid_fees {
            0
        } else {
            self.calculate_protocol_fee(amount)
        };
        remainder -= protocol_fee;

        // calculate referrer fee, if applicable
        let mut referrer_fee = None;
        if let Some(_referrer_id) = referrer_id.clone() {
            let referrer_amount = calculate_fee(amount, campaign.referral_fee_basis_points);
            remainder -= referrer_amount;
            referrer_fee = Some(referrer_amount);
        }

        // calculate creator fee, if applicable
        let creator_fee: u128 = if bypass_creator_fee.unwrap_or(false) {
            0
        } else {
            calculate_fee(amount, campaign.creator_fee_basis_points)
        };
        remainder -= creator_fee;

        (protocol_fee, referrer_fee, creator_fee, remainder)
    }
}
