use crate::*;

pub const TGAS: u64 = 1_000_000_000_000;
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;

// TokenId and ClassId must be positive (0 is not a valid ID)
pub type TokenId = u64;
pub type ClassId = u64;

/// TokenMetadata defines attributes for each SBT token.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub class: ClassId,            // token class. Required. Must be non zero.
    pub issued_at: Option<u64>,    // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>,   // When token expires, Unix epoch in milliseconds
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OwnedToken {
    pub token: TokenId,
    pub metadata: TokenMetadata,
}

pub type SbtTokensByOwnerResult = Vec<(AccountId, Vec<OwnedToken>)>;

#[ext_contract(sbt_registry)]
trait SbtRegistry {
    fn sbt_tokens_by_owner(
        &self,
        account: AccountId,
        issuer: Option<AccountId>,
        from_class: Option<u64>,
    ) -> SbtTokensByOwnerResult;
}
