import fs from "fs";
import BN from "bn.js";
import assert from "assert";

import { Account, utils } from "near-api-js";
import { DEFAULT_DEPOSIT, DEFAULT_GAS } from "./constants";

export const loadCredentials = (networkId: string, contractId: string) => {
  let path = `${process.env.HOME}/.near-credentials/${networkId}/${contractId}.json`;
  if (!fs.existsSync(path)) {
    path = `./registry/neardev/${networkId}/${contractId}.json`;
    if (!fs.existsSync(path)) {
      console.warn("Credentials not found");
      return null;
    }
  }
  return JSON.parse(fs.readFileSync(path, "utf-8"));
};

export const _contractCall = async ({
  contractId,
  callerAccount,
  methodName,
  args,
  gas,
  attachedDeposit,
}: {
  contractId: AccountId;
  callerAccount: Account;
  methodName: string;
  args?: Record<string, any>;
  gas?: string;
  attachedDeposit?: string;
}) => {
  return await callerAccount.functionCall({
    contractId,
    methodName,
    args,
    gas: new BN(gas || DEFAULT_GAS),
    attachedDeposit:
      !attachedDeposit || attachedDeposit === "0"
        ? undefined
        : new BN(attachedDeposit),
  });
};

export const _contractView = async ({
  contractId,
  callerAccount,
  methodName,
  args,
  gas,
  attachedDeposit,
}: {
  contractId: AccountId;
  callerAccount: Account;
  methodName: string;
  args?: Record<string, any>;
  gas?: string;
  attachedDeposit?: string;
}) => {
  return await callerAccount.viewFunction({
    contractId,
    methodName,
    args,
    gas: new BN(gas || DEFAULT_GAS),
    attachedDeposit:
      !attachedDeposit || attachedDeposit === "0"
        ? undefined
        : new BN(attachedDeposit),
  });
};
