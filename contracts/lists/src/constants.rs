use crate::*;

pub const EVENT_JSON_PREFIX: &str = "EVENT_JSON:";
pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_PER_TRANSFER: Gas = Gas(TGAS / 2);
pub const XCC_GAS: Gas = Gas(TGAS * 5);
