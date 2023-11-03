use crate::*;

#[near_bindgen]
impl Contract {
    // OWNER
    #[payable]
    pub fn owner_change_owner(&mut self, owner: AccountId) {
        // TODO: consider renaming to owner_set_owner, but currently deployed Registry uses owner_change_owner.
        self.assert_owner();
        self.owner = owner;
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    // FEES CONFIG
    #[payable]
    pub fn owner_set_protocol_fee_basis_points(&mut self, protocol_fee_basis_points: u32) {
        self.assert_owner();
        self.protocol_fee_basis_points = protocol_fee_basis_points;
    }

    // referral_fee_basis_points
    #[payable]
    pub fn owner_set_referral_fee_basis_points(&mut self, referral_fee_basis_points: u32) {
        self.assert_owner();
        self.referral_fee_basis_points = referral_fee_basis_points;
    }

    // protocol_fee_recipient_account
    #[payable]
    pub fn owner_set_protocol_fee_recipient_account(
        &mut self,
        protocol_fee_recipient_account: AccountId,
    ) {
        self.assert_owner();
        self.protocol_fee_recipient_account = protocol_fee_recipient_account;
    }
}
