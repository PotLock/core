import { utils } from "near-api-js";

export const contractId = "1695237705.test-contracts.potlock.testnet";
export const parentAccountId = contractId.split(".").slice(1).join(".");
export const networkId = "testnet";
export const nodeUrl = `https://rpc.${networkId}.near.org`;

export const DEFAULT_WHITELISTED_DEPLOYER_ID = "test-contracts.potlock.testnet"; // easy to set it to this value as we have the key for it

export const DEFAULT_MAX_ROUND_TIME = 1000 * 60 * 60 * 24 * 7 * 8; // 8 weeks
export const DEFAULT_ROUND_LENGTH = 1000 * 60 * 60 * 24 * 7 * 7; // 7 weeks
export const DEFAULT_MAX_APPLICATION_TIME = 1000 * 60 * 60 * 24 * 7 * 2; // 2 weeks
export const DEFAULT_APPLICATION_LENGTH = 1000 * 60 * 60 * 24 * 7 * 1; // 1 week
export const DEFAULT_PROTOCOL_FEE_BASIS_POINTS = 700; // 7%
export const DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS = 1000; // 10%
export const DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS = 100; // 1%
export const DEFAULT_MAX_CHEF_FEE_BASIS_POINTS = 500; // 5%
export const DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS = 100; // 1%
export const DEFAULT_MAX_PATRON_REFERRAL_FEE = utils.format.parseNearAmount(
  "1000"
) as string;
export const DEFAULT_ROUND_MANAGER_FEE_BASIS_POINTS = 100; // 1%
export const DEFAULT_MAX_PROJECTS = 10;
export const DEFAULT_BASE_CURRENCY = "near";
export const DEFAULT_REGISTRY_ID = "registry-unstable.i-am-human.testnet";
export const DEFAULT_ISSUER_ID = "i-am-human-staging.testnet";
export const DEFAULT_CLASS_ID = 1;

export const ASSERT_ADMIN_ERROR_STR = "Only admin can call this method";
export const ASSERT_ADMIN_OR_WHITELISTED_DEPLOYER_ERROR_STR =
  "Only admin or whitelisted deployers can call this method";
