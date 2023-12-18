use crate::*;

pub const EXTRA_BYTES: usize = 10_000;
pub const TGAS: u64 = 1_000_000_000_000; // 1 TGAS
pub const XCC_GAS: Gas = Gas(TGAS * 50); // 50 TGAS
pub const NO_DEPOSIT: u128 = 0;
pub const XCC_SUCCESS: u64 = 1;
pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";
