import { utils } from "near-api-js";

export const contractId = "1695237705.test-contracts.potlock.testnet";
export const parentAccountId = contractId.split(".").slice(1).join(".");
export const networkId = "testnet";
export const nodeUrl = `https://rpc.${networkId}.near.org`;

export const POT_DEPLOYER_ALWAYS_ADMIN_ID = contractId;
export const DEFAULT_WHITELISTED_DEPLOYER_ID = "test-contracts.potlock.testnet"; // easy to set it to this value as we have the key for it

export const ASSERT_ADMIN_ERROR_STR = "Only admin can call this method";
export const ASSERT_ADMIN_OR_WHITELISTED_DEPLOYER_ERROR_STR =
  "Only admin or whitelisted deployers can call this method";
