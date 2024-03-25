use std::hash;

use near_sdk::PromiseOrValue;

use crate::*;

#[near_bindgen]
impl Contract {
    pub fn withdraw_refund(&mut self) -> PromiseOrValue<u128> {
        let refund_amount = self
            .refund_claims_by_registrant_id
            .get(&env::predecessor_account_id())
            .unwrap_or(0);
        if refund_amount > 0 {
            PromiseOrValue::Promise(
                Promise::new(env::predecessor_account_id())
                    .transfer(refund_amount)
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(XCC_GAS)
                            .refund_callback(env::predecessor_account_id(), refund_amount),
                    ),
            )
        } else {
            PromiseOrValue::Value(refund_amount)
        }
    }

    #[private]
    pub fn refund_callback(
        &mut self,
        recipient: AccountId,
        refund_amount: u128,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> PromiseOrValue<u128> {
        if call_result.is_err() {
            log!(format!(
                "Error paying out refund {:#?} to {}",
                refund_amount, recipient
            ));
            PromiseOrValue::Value(0)
        } else {
            log!(format!(
                "Successfully paid out refund {:#?} to {}",
                refund_amount, recipient
            ));
            self.refund_claims_by_registrant_id.insert(&recipient, &0);
            PromiseOrValue::Value(refund_amount)
        }
    }

    pub fn get_available_refund(&self, account_id: Option<AccountId>) -> Balance {
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        self.refund_claims_by_registrant_id
            .get(&account_id)
            .unwrap_or(0)
    }

    pub fn get_refunds(&self) -> HashMap<AccountId, Balance> {
        let mut refunds = HashMap::new();
        for (account_id, refund) in self.refund_claims_by_registrant_id.iter() {
            refunds.insert(account_id, refund);
        }
        refunds
    }
}
