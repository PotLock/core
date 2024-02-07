use crate::*;

pub(crate) fn assert_valid_pot_name(name: &str) {
    assert!(
        name.len() <= MAX_POT_NAME_LENGTH,
        "Provider name is too long"
    );
}

pub(crate) fn assert_valid_pot_description(description: &str) {
    assert!(
        description.len() <= MAX_POT_DESCRIPTION_LENGTH,
        "Provider description is too long"
    );
}

pub(crate) fn assert_valid_max_projects(max_projects: u32) {
    assert!(
        max_projects <= MAX_MAX_PROJECTS,
        "Max projects cannot exceed {}",
        MAX_MAX_PROJECTS
    );
}

pub(crate) fn assert_valid_referral_fee_matching_pool_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_REFERRAL_FEE_MATCHING_POOL_BASIS_POINTS,
        "Referral fee matching pool basis points cannot exceed {}",
        MAX_REFERRAL_FEE_MATCHING_POOL_BASIS_POINTS
    );
}

pub(crate) fn assert_valid_referral_fee_public_round_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_REFERRAL_FEE_PUBLIC_ROUND_BASIS_POINTS,
        "Referral fee public round basis points cannot exceed {}",
        MAX_REFERRAL_FEE_PUBLIC_ROUND_BASIS_POINTS
    );
}

pub(crate) fn assert_valid_chef_fee_basis_points(basis_points: u32) {
    assert!(
        basis_points <= MAX_CHEF_FEE_BASIS_POINTS,
        "Chef fee basis points cannot exceed {}",
        MAX_CHEF_FEE_BASIS_POINTS
    );
}

pub(crate) fn assert_valid_pot_timestamps(args: &PotArgs) {
    assert!(
        args.application_start_ms < args.application_end_ms,
        "Application start must be before application end"
    );
    assert!(
        args.application_end_ms < args.public_round_start_ms,
        "Application end must be before public round start"
    );
    assert!(
        args.public_round_start_ms < args.public_round_end_ms,
        "Public round start must be before public round end"
    );
}

pub(crate) fn assert_valid_provider_id(provider_id: &ProviderId) {
    provider_id.validate();
}

pub(crate) fn assert_valid_cooldown_period_ms(cooldown_period_ms: u64) {
    assert!(
        cooldown_period_ms >= MIN_COOLDOWN_PERIOD_MS,
        "Cooldown period must be at least {} ms",
        MIN_COOLDOWN_PERIOD_MS
    );
}

pub(crate) fn assert_valid_pot_args(args: &PotArgs) {
    assert_valid_pot_name(&args.pot_name);
    assert_valid_pot_description(&args.pot_description);
    assert_valid_max_projects(args.max_projects);
    assert_valid_referral_fee_matching_pool_basis_points(
        args.referral_fee_matching_pool_basis_points,
    );
    assert_valid_referral_fee_public_round_basis_points(
        args.referral_fee_public_round_basis_points,
    );
    assert_valid_chef_fee_basis_points(args.chef_fee_basis_points);
    assert_valid_pot_timestamps(args);
    if let Some(cooldown_period_ms) = args.cooldown_period_ms {
        assert_valid_cooldown_period_ms(cooldown_period_ms);
    }
    if let Some(provider_id) = &args.registry_provider {
        assert_valid_provider_id(provider_id);
    }
    if let Some(provider_id) = &args.sybil_wrapper_provider {
        assert_valid_provider_id(provider_id);
    }
    if let Some(custom_sybil_checks) = &args.custom_sybil_checks {
        for check in custom_sybil_checks {
            assert_valid_provider_id(&ProviderId::new(
                check.contract_id.clone().to_string(),
                check.method_name.clone(),
            ));
        }
    }
    if let Some(protocol_config_provider) = &args.protocol_config_provider {
        assert_valid_provider_id(protocol_config_provider);
    }
}
