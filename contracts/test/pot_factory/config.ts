import { utils } from "near-api-js";
import { DEFAULT_PARENT_ACCOUNT_ID } from "../utils/constants";

export const contractId = "1702409170.test-contracts.potlock.testnet";
export const parentAccountId = DEFAULT_PARENT_ACCOUNT_ID;
export const networkId = "testnet";
export const nodeUrl = `https://rpc.${networkId}.near.org`;

export const POT_FACTORY_ALWAYS_ADMIN_ID = contractId;
export const DEFAULT_WHITELISTED_DEPLOYER_ID = DEFAULT_PARENT_ACCOUNT_ID; // easy to set it to this value as we have the key for it // TODO: consider changing this for clarity

export const ASSERT_ADMIN_ERROR_STR = "Only admin can call this method";
export const ASSERT_ADMIN_OR_WHITELISTED_DEPLOYER_ERROR_STR =
  "Only admin or whitelisted deployers can call this method";
