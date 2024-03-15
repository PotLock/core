use crate::*;

pub const TGAS: u64 = 1_000_000_000_000; // 1 TGAS
pub const XCC_GAS_DEFAULT: u64 = TGAS * 10; // 10 TGAS
pub const NO_DEPOSIT: Balance = 0;

pub const PROVIDER_DEFAULT_WEIGHT: u32 = 100;
pub const MAX_PROVIDER_NAME_LENGTH: usize = 64;
pub const MAX_PROVIDER_DESCRIPTION_LENGTH: usize = 256;
pub const MAX_PROVIDER_EXTERNAL_URL_LENGTH: usize = 256;
pub const MAX_PROVIDER_ICON_URL_LENGTH: usize = 256;
pub const MAX_TAGS_PER_PROVIDER: usize = 10;
pub const MAX_TAG_LENGTH: usize = 32;
pub const MAX_GAS: u64 = 100_000_000_000_000;
