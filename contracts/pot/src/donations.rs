use crate::*;

#[near_bindgen]
impl Contract {
    /// Assert that at least 1 yoctoNEAR was attached.
    // TODO: change to pub(crate) and call internally
    pub fn assert_can_donate(&self) -> Promise {
        if let Some(donation_requirement) = &self.donation_requirement {
            // let tokens = self.query_sbts_for_owner(
            //     donation_requirement.registry_id,
            //     Some(donation_requirement.issuer_id),
            //     Some(donation_requirement.class_id),
            // );

            // require!(
            //     env::attached_deposit() >= 1,
            //     "Requires attached deposit of at least 1 yoctoNEAR"
            // )
            let promise = sbt_registry::ext(donation_requirement.registry_id.clone())
                .with_static_gas(Gas(5 * TGAS))
                .sbt_tokens_by_owner(
                    env::predecessor_account_id(),
                    Some(donation_requirement.issuer_id.clone()),
                    Some(donation_requirement.class_id),
                );

            promise.then(
                // Create a promise to callback query_greeting_callback
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .assert_can_donate_callback(),
            )
        } else {
            // no donation requirement. always allow
            Promise::new(env::current_account_id()).then(
                Self::ext(env::current_account_id())
                    .with_static_gas(Gas(5 * TGAS))
                    .always_allow_callback(),
            )
        }
    }

    #[private]
    pub fn always_allow_callback(&self) -> bool {
        true
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn assert_can_donate_callback(
        &self,
        #[callback_result] call_result: Result<SbtTokensByOwnerResult, PromiseError>,
    ) -> bool {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
            log!("There was an error querying SBTs");
            return false;
        }

        // Return the tokens
        let tokens: Vec<(AccountId, Vec<OwnedToken>)> = call_result.unwrap();
        if tokens.len() > 0 {
            true
        } else {
            false
        }
    }
}
