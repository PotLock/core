use crate::*;

pub const ONE_DAY_MS: u64 = 86_400_000;
pub const ONE_WEEK_MS: u64 = ONE_DAY_MS * 7;
pub const EXTRA_BYTES: usize = 10_000;
pub const TGAS: u64 = 1_000_000_000_000; // 1 TGAS
pub const XCC_GAS: Gas = Gas(TGAS * 50); // 50 TGAS
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;
pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";

// Pot args constraints
pub const MAX_POT_NAME_LENGTH: usize = 64;
pub const MAX_POT_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_MAX_PROJECTS: u32 = 100; // TODO: figure out actual limit based on gas
pub const MAX_REFERRAL_FEE_MATCHING_POOL_BASIS_POINTS: u32 = 1000; // 10%
pub const MAX_REFERRAL_FEE_PUBLIC_ROUND_BASIS_POINTS: u32 = 1000; // 10%
pub const MAX_CHEF_FEE_BASIS_POINTS: u32 = 1000; // 10%
pub const MIN_COOLDOWN_PERIOD_MS: u64 = ONE_WEEK_MS;
pub const DEFAULT_COOLDOWN_PERIOD_MS: u64 = ONE_WEEK_MS;
