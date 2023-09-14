use crate::*;

/// Used ephemerally in view methods
pub struct PotConfig {
    /// Address (ID) of round manager ("chef"), essentially the contract owner
    pub chef_id: AccountId,
    /// Friendly & descriptive round name
    pub round_name: String,
    /// Friendly & descriptive round description
    pub round_description: String,
    /// MS Timestamp when the round starts
    pub round_start_ms: TimestampMs,
    /// MS Timestamp when the round ends
    pub round_end_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted from
    pub application_start_ms: TimestampMs,
    /// MS Timestamp when applications can be submitted until
    pub application_end_ms: TimestampMs,
    /// Maximum number of projects that can be approved for the round
    pub max_projects: u32,
    /// Base currency for the round
    pub base_currency: AccountId,
    /// Account ID that deployed this Pot contract
    pub created_by: AccountId,
    /// If project raises less than this amount in donations, milestone submissions aren't required
    pub milestone_threshold: U64, // TODO: is this practical to implement?
    pub basis_points_paid_upfront: u32, // TODO: what does this mean? how will it be paid upfront if there are no donations yet?
    /// SBTs required to submit an application
    pub application_requirement: Option<SBTRequirement>,
    /// SBTs required to donate to a project
    pub donation_requirement: Option<SBTRequirement>,
    // payment_per_milestone: u32,
    pub patron_referral_fee_basis_points: u32,
    /// Max amount that can be paid to an account that referred a Patron
    pub max_patron_referral_fee: U128, // TODO: consider whether this is necessary
    /// Chef's fee for managing the round
    pub round_manager_fee_basis_points: u32, // TODO: should this be basis points or a fixed amount?
    /// Protocol fee
    pub protocol_fee_basis_points: u32, // e.g. 700 (7%)
    /// Amount of matching funds available
    pub matching_pool_balance: U128,
    /// Amount of donated funds available
    pub donations_balance: U128,
    /// Cooldown period starts when Chef sets payouts
    pub cooldown_end_ms: Option<TimestampMs>,
    /// Have all projects been paid out?
    pub paid_out: bool,
}

impl Contract {
    pub fn get_pot_config(&self) -> PotConfig {
        PotConfig {
            chef_id: self.chef_id.clone(),
            round_name: self.round_name.clone(),
            round_description: self.round_description.clone(),
            round_start_ms: self.round_start_ms,
            round_end_ms: self.round_end_ms,
            application_start_ms: self.application_start_ms,
            application_end_ms: self.application_end_ms,
            max_projects: self.max_projects,
            base_currency: self.base_currency.clone(),
            created_by: self.created_by.clone(),
            milestone_threshold: self.milestone_threshold,
            basis_points_paid_upfront: self.basis_points_paid_upfront,
            application_requirement: self.application_requirement.clone(),
            donation_requirement: self.donation_requirement.clone(),
            patron_referral_fee_basis_points: self.patron_referral_fee_basis_points,
            max_patron_referral_fee: self.max_patron_referral_fee,
            round_manager_fee_basis_points: self.round_manager_fee_basis_points,
            protocol_fee_basis_points: self.protocol_fee_basis_points,
            matching_pool_balance: self.matching_pool_balance,
            donations_balance: self.donations_balance,
            cooldown_end_ms: self.cooldown_end_ms,
            paid_out: self.paid_out,
        }
    }
}
