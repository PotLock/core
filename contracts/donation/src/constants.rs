use crate::*;

pub const MAX_PROTOCOL_FEE_BASIS_POINTS: u32 = 1000;
pub const MAX_REFERRAL_FEE_BASIS_POINTS: u32 = 200;

pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";

pub const TGAS: u64 = 1_000_000_000_000; // 1 TGAS
pub const XCC_GAS_DEFAULT: u64 = TGAS * 10; // 10 TGAS
pub const NO_DEPOSIT: Balance = 0;
