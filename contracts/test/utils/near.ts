import { utils } from "near-api-js";

export const DEFAULT_GAS = "300000000000000";

export const functionCallBase = {
  gas: DEFAULT_GAS,
  attachedDeposit: utils.format.parseNearAmount("0.1"),
};

export const NO_CONTRACT_HASH = "1".repeat(32);
