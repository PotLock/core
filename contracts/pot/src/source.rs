use crate::*;

/// CONTRACT SOURCE METADATA - as per NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractSourceMetadata {
    /// Version of source code, e.g. "v1.0.0", could correspond to Git tag
    pub version: String,
    /// Git commit hash of currently deployed contract code
    pub commit_hash: String,
    /// GitHub repo url for currently deployed contract code
    pub link: String,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) {
        // only contract account (aka the account that can deploy new code to this contract) can call this method
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "Only contract account can call this method"
        );
        self.contract_source_metadata.set(&source_metadata);
        // emit event
        log_set_source_metadata_event(&source_metadata);
    }

    pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata> {
        let source_metadata = self.contract_source_metadata.get();
        if source_metadata.is_some() {
            Some(ContractSourceMetadata::from(source_metadata.unwrap()))
        } else {
            None
        }
    }
}
