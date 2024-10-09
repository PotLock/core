use crate::*;

#[near]
impl Contract {
    // FEES CONFIG
    #[payable]
    pub fn admin_set_protocol_fee_basis_points(&mut self, protocol_fee_basis_points: u32) {
        self.assert_contract_admin_or_greater();
        assert_valid_protocol_fee_basis_points(protocol_fee_basis_points);
        let initial_storage_usage = env::storage_usage();
        self.protocol_fee_basis_points = protocol_fee_basis_points;
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    // referral_fee_basis_points
    #[payable]
    pub fn admin_set_referral_fee_basis_points(&mut self, referral_fee_basis_points: u32) {
        self.assert_contract_admin_or_greater();
        assert_valid_referral_fee_basis_points(referral_fee_basis_points);
        let initial_storage_usage = env::storage_usage();
        self.default_referral_fee_basis_points = referral_fee_basis_points;
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    // creator_fee_basis_points
    #[payable]
    pub fn admin_set_creator_fee_basis_points(&mut self, creator_fee_basis_points: u32) {
        self.assert_contract_admin_or_greater();
        assert_valid_creator_fee_basis_points(creator_fee_basis_points);
        let initial_storage_usage = env::storage_usage();
        self.default_creator_fee_basis_points = creator_fee_basis_points;
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }

    // protocol_fee_recipient_account
    #[payable]
    pub fn admin_set_protocol_fee_recipient_account(
        &mut self,
        protocol_fee_recipient_account: AccountId,
    ) {
        self.assert_contract_admin_or_greater();
        let initial_storage_usage = env::storage_usage();
        self.protocol_fee_recipient_account = protocol_fee_recipient_account;
        refund_deposit(initial_storage_usage);
        log_config_update_event(&self.get_config());
    }
}
