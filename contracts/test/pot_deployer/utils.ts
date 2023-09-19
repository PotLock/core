import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account, utils } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";

const READ_METHODS = {
  GET_POTS: "get_pots",
};

const WRITE_METHODS = {
  NEW: "new",
  DEPLOY_POT: "deploy_pot",
};

// Wrapper around contractView that defaults to the contract account
const contractView = async ({
  contractId,
  methodName,
  args,
  gas,
  attachedDeposit,
}: {
  contractId?: string;
  methodName: string;
  args?: Record<string, any>;
  gas?: string;
  attachedDeposit?: string;
}) => {
  return _contractView({
    contractId: contractId || _contractId,
    callerAccount: contractAccount,
    methodName,
    args,
    gas,
    attachedDeposit,
  });
};

// Wrapper around contractCall that defaults to the contract account
const contractCall = async ({
  callerAccount,
  contractId = _contractId,
  methodName,
  args,
  gas,
  attachedDeposit,
}: {
  callerAccount: Account;
  contractId: string;
  methodName: string;
  args?: Record<string, any>;
  gas?: string;
  attachedDeposit?: string;
}) => {
  return _contractCall({
    contractId,
    callerAccount,
    methodName,
    args,
    gas,
    attachedDeposit,
  });
};

// Helper function for the common case of contract calling itself
export const callSelf = async ({
  methodName,
  args,
  gas,
  attachedDeposit,
}: {
  methodName: string;
  args?: Record<string, any>;
  gas?: string;
  attachedDeposit?: string;
}) => {
  return contractCall({
    callerAccount: contractAccount,
    contractId: _contractId,
    methodName,
    args,
    gas,
    attachedDeposit,
  });
};

export const initializeContract = async (
  initializeArgs?: Record<string, any>
) => {
  return callSelf({
    methodName: WRITE_METHODS.NEW,
    args: initializeArgs,
    attachedDeposit: NO_DEPOSIT,
  });
};

// DEPLOYING POTS

export const deployPot = async (
  callerAccount: Account,
  potOnChainName: string,
  potArgs: PotArgs
) => {
  return contractCall({
    callerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.DEPLOY_POT,
    args: {
      pot_on_chain_name: potOnChainName,
      pot_args: potArgs,
    },
    attachedDeposit: utils.format.parseNearAmount("1") as string,
  });
};

export const getPots = async (): Promise<Pot[]> => {
  return contractView({
    methodName: READ_METHODS.GET_POTS,
  });
};
