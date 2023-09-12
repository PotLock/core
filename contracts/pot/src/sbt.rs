use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SBTRequirement {
    pub registry_id: AccountId,
    pub issuer_id: AccountId,
    pub class_id: u64,
}

#[near_bindgen]
impl Contract {
    // TODO: possibly remove this; currently unused
    pub(crate) fn query_sbts_for_owner(
        &self,
        registry_id: AccountId,
        issuer: Option<AccountId>,
        from_class: Option<u64>,
    ) -> Promise {
        let promise = sbt_registry::ext(registry_id)
            .with_static_gas(Gas(5 * TGAS))
            .sbt_tokens_by_owner(env::predecessor_account_id(), issuer, from_class);

        return promise.then(
            // Create a promise to callback query_greeting_callback
            Self::ext(env::current_account_id())
                .with_static_gas(Gas(5 * TGAS))
                .query_sbt_callback(),
        );
    }

    #[private] // Public - but only callable by env::current_account_id()
    pub fn query_sbt_callback(
        &self,
        #[callback_result] call_result: Result<SbtTokensByOwnerResult, PromiseError>,
    ) -> SbtTokensByOwnerResult {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
            log!("There was an error querying SBTs");
            return vec![];
        }

        // Return the tokens
        let tokens: Vec<(AccountId, Vec<OwnedToken>)> = call_result.unwrap();
        tokens
    }
}
