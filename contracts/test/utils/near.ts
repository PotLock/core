import { utils } from "near-api-js";

export const functionCallBase = {
  gas: "300000000000000",
  attachedDeposit: utils.format.parseNearAmount("0.1"),
};
