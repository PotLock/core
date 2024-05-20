use crate::*;

pub(crate) fn assert_valid_protocol_fee_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_PROTOCOL_FEE_BASIS_POINTS,
        "Protocol fee matching pool basis points cannot exceed {}",
        MAX_PROTOCOL_FEE_BASIS_POINTS
    );
}

pub(crate) fn assert_valid_referral_fee_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_REFERRAL_FEE_BASIS_POINTS,
        "Referral fee public round basis points cannot exceed {}",
        MAX_REFERRAL_FEE_BASIS_POINTS
    );
}

pub(crate) fn assert_valid_creator_fee_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_CREATOR_FEE_BASIS_POINTS,
        "Creator fee public round basis points cannot exceed {}",
        MAX_CREATOR_FEE_BASIS_POINTS
    );
}
