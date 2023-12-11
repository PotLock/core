use crate::*;

pub const ONE_DAY_MS: u64 = 86_400_000;
pub const ONE_WEEK_MS: u64 = ONE_DAY_MS * 7;
pub const TGAS: u64 = 1_000_000_000_000;
pub const XCC_GAS: Gas = Gas(TGAS * 5);
