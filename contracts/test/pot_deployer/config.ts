import { utils } from "near-api-js";

export const contractId = "1695219221.test-contracts.potlock.testnet";
export const networkId = "testnet";
export const nodeUrl = `https://rpc.${networkId}.near.org`;

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
