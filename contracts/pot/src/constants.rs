use crate::*;

pub const ONE_DAY_MS: u64 = 86_400_000;
pub const ONE_WEEK_MS: u64 = ONE_DAY_MS * 7;
pub const TGAS: u64 = 1_000_000_000_000;
pub const XCC_GAS: Gas = Gas(TGAS * 5);
pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";

// Pot args constraints
pub const MAX_POT_NAME_LENGTH: usize = 64;
pub const MAX_POT_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_MAX_PROJECTS: u32 = 100; // TODO: figure out actual limit based on gas
pub const MAX_REFERRAL_FEE_MATCHING_POOL_BASIS_POINTS: u32 = 1000; // 10%
pub const MAX_REFERRAL_FEE_PUBLIC_ROUND_BASIS_POINTS: u32 = 1000; // 10%
pub const MAX_CHEF_FEE_BASIS_POINTS: u32 = 1000; // 10%
pub const MAX_PROTOCOL_FEE_BASIS_POINTS: u32 = 1000; // 10%
