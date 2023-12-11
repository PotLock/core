import { utils } from "near-api-js";

export const DEFAULT_GAS = "300000000000000";
export const DEFAULT_DEPOSIT = utils.format.parseNearAmount("0.1") as string;
export const NO_DEPOSIT = "0";

// export const functionCallBase = {
//   gas: DEFAULT_GAS,
//   attachedDeposit: utils.format.parseNearAmount("0.1"),
// };

export const NO_CONTRACT_HASH = "1".repeat(32);

// POT CONFIG
export const DEFAULT_MAX_ROUND_TIME = 1000 * 60 * 60 * 24 * 7 * 8; // 8 weeks
export const DEFAULT_ROUND_LENGTH = 1000 * 60 * 60 * 24 * 7 * 7; // 7 weeks
export const DEFAULT_MAX_APPLICATION_TIME = 1000 * 60 * 60 * 24 * 7 * 2; // 2 weeks
export const DEFAULT_APPLICATION_LENGTH = 1000 * 60 * 60 * 24 * 7 * 1; // 1 week
export const DEFAULT_PROTOCOL_FEE_BASIS_POINTS = 700; // 7%
export const DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS = 1000; // 10%
export const DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS = 100; // 1%
export const DEFAULT_MAX_CHEF_FEE_BASIS_POINTS = 500; // 5%
export const DEFAULT_REFERRAL_FEE_BASIS_POINTS = 100; // 1% // TODO: clean this up
export const DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS = 100; // 1%
export const DEFAULT_chef_fee_basis_points = 100; // 1%
export const DEFAULT_MAX_PROJECTS = 10;
export const DEFAULT_BASE_CURRENCY = "near";
export const DEFAULT_REGISTRY_ID = "registry-unstable.i-am-human.testnet";
export const DEFAULT_ISSUER_ID = "i-am-human-staging.testnet";
export const DEFAULT_CLASS_ID = 1;

export const DEFAULT_PARENT_ACCOUNT_ID = "test-contracts.potlock.testnet"; // accounts created during testing, with the exception of near dev-deploy, will be subaccounts of this account.
export const DEFAULT_NEW_ACCOUNT_AMOUNT = utils.format.parseNearAmount("10"); // 10 NEAR
