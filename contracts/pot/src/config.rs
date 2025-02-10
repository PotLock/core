use crate::*;

/// Used ephemerally in view methods
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PotConfig {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub pot_name: String,
    pub pot_description: String,
    pub tags: Vec<String>,
    pub pot_operation_rules: String,
    pub token_address: Option<AccountId>,
    pub applications_open: bool,
    pub reup_policy: ReupPolicy,
    pub pot_status: PotStatus,
    pub governance_type: GovernanceType,
    pub max_projects: u32,
    pub base_currency: AccountId,
    pub deployed_by: AccountId,
    pub registry_provider: Option<ProviderId>,
    pub min_treasury_donation_amount: U128,
    pub sybil_wrapper_provider: Option<ProviderId>,
    pub custom_sybil_checks: Option<HashMap<ProviderId, SybilProviderWeight>>,
    pub custom_min_threshold_score: Option<u32>,
    pub referral_fee_treasury_pool_basis_points: u32,
    pub referral_fee_public_round_basis_points: u32,
    pub total_treasury_pool_donations: U128,
    pub total_public_donations: U128,
    pub public_donations_count: u32,
    pub payouts: Vec<PayoutExternal>,
    pub cooldown_period_ms: u64,
    pub cooldown_end_ms: Option<TimestampMs>,
    pub compliance_period_ms: Option<u64>,
    pub compliance_end_ms: Option<TimestampMs>,
    pub all_paid_out: bool,
    pub protocol_config_provider: Option<ProviderId>,
}

#[near_bindgen]
impl Contract {
    pub fn get_config(&self) -> PotConfig {
        PotConfig {
            owner: self.owner.clone(),
            admins: self.admins.to_vec(),
            pot_name: self.pot_name.clone(),
            pot_description: self.pot_description.clone(),
            pot_operation_rules: self.pot_operation_rules.clone(),
            applications_open: self.applications_open,
            reup_policy: self.reup_policy.clone(),
            pot_status: self.pot_status.clone(),
            governance_type: self.governance_type.clone(),
            token_address: self.token_address.clone(),
            min_treasury_donation_amount: self.min_treasury_donation_amount.into(),
            referral_fee_treasury_pool_basis_points: self.referral_fee_treasury_pool_basis_points,
            tags: self.tags.clone(),
            max_projects: self.max_projects.unwrap_or(0),
            base_currency: self.base_currency.clone(),
            deployed_by: self.deployed_by.clone(),
            registry_provider: self.registry_provider.get(),
            sybil_wrapper_provider: self.sybil_wrapper_provider.get(),
            custom_sybil_checks: self.custom_sybil_checks.get(),
            custom_min_threshold_score: self.custom_min_threshold_score.get(),
            referral_fee_public_round_basis_points: self.referral_fee_spending_pool_basis_points,
            total_treasury_pool_donations: self.total_treasury_pool_donations.into(),
            total_public_donations: self.total_spending_pool_donations.into(),
            public_donations_count: self.spending_pool_donation_ids.len() as u32,
            payouts: self.get_payouts(None, None),
            cooldown_period_ms: self.cooldown_period_ms,
            cooldown_end_ms: self.cooldown_end_ms.get(),
            compliance_period_ms: self.compliance_period_ms.get(),
            compliance_end_ms: self.compliance_end_ms.get(),
            all_paid_out: self.all_paid_out,
            protocol_config_provider: self.protocol_config_provider.get(),
        }
    }
}
