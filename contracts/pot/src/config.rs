use crate::*;

/// Used ephemerally in view methods
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotConfig {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub chef: Option<AccountId>,
    pub pot_name: String,
    pub pot_description: String,
    pub max_projects: u32,
    pub base_currency: AccountId,
    pub application_start_ms: TimestampMs,
    pub application_end_ms: TimestampMs,
    pub public_round_start_ms: TimestampMs,
    pub public_round_end_ms: TimestampMs,
    pub deployed_by: AccountId,
    pub registry_provider: Option<ProviderId>,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<HashMap<ProviderId, SybilProviderWeight>>,
    pub custom_min_threshold_score: Option<u32>,
    pub patron_referral_fee_basis_points: u32,
    pub public_round_referral_fee_basis_points: u32,
    pub chef_fee_basis_points: u32, // TODO: should this be basis points or a fixed amount?
    pub matching_pool_balance: U128,
    pub total_donations: U128,
    pub cooldown_end_ms: Option<TimestampMs>,
    pub all_paid_out: bool,
}

#[near_bindgen]
impl Contract {
    pub fn get_pot_config(&self) -> PotConfig {
        PotConfig {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            chef: self.chef.get(),
            pot_name: self.pot_name.clone(),
            pot_description: self.pot_description.clone(),
            max_projects: self.max_projects,
            base_currency: self.base_currency.clone(),
            application_start_ms: self.application_start_ms,
            application_end_ms: self.application_end_ms,
            public_round_start_ms: self.public_round_start_ms,
            public_round_end_ms: self.public_round_end_ms,
            deployed_by: self.deployed_by.clone(),
            registry_provider: self.registry_provider.get(),
            sybil_wrapper_provider: self.sybil_wrapper_provider.get(),
            custom_sybil_checks: self.custom_sybil_checks.get(),
            custom_min_threshold_score: self.custom_min_threshold_score.get(),
            patron_referral_fee_basis_points: self.patron_referral_fee_basis_points,
            public_round_referral_fee_basis_points: self.public_round_referral_fee_basis_points,
            chef_fee_basis_points: self.chef_fee_basis_points,
            matching_pool_balance: self.matching_pool_balance,
            total_donations: self.total_donations,
            cooldown_end_ms: self.cooldown_end_ms.get(),
            all_paid_out: self.all_paid_out,
        }
    }
}
