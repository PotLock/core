import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account, utils } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";

const READ_METHODS = {
  GET_POTS: "get_pots",
  GET_WHITELISTED_DEPLOYERS: "get_whitelisted_deployers",
  GET_CONFIG: "get_config",
  GET_ADMIN: "get_admin",
};

const WRITE_METHODS = {
  NEW: "new",
  DEPLOY_POT: "deploy_pot",
  ADMIN_ADD_WHITELISTED_DEPLOYERS: "admin_add_whitelisted_deployers",
  ADMIN_REMOVE_WHITELISTED_DEPLOYERS: "admin_remove_whitelisted_deployers",
  ADMIN_UPDATE_PROTOCOL_FEE_BASIS_POINTS:
    "admin_update_protocol_fee_basis_points",
  ADMIN_SET_DEFAULT_CHEF_FEE_BASIS_POINTS:
    "admin_set_default_chef_fee_basis_points",
  ADMIN_UPDATE_MAX_PROTOCOL_FEE_BASIS_POINTS:
    "admin_update_max_protocol_fee_basis_points",
  ADMIN_UPDATE_MAX_CHEF_FEE_BASIS_POINTS:
    "admin_update_max_chef_fee_basis_points",
  ADMIN_UPDATE_MAX_ROUND_TIME: "admin_update_max_round_time",
  ADMIN_UPDATE_MAX_APPLICATION_TIME: "admin_update_max_application_time",
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

// WHITELISTED DEPLOYERS

export const adminAddWhitelistedDeployers = async (
  adminAccount: Account,
  whitelistedDeployers: AccountId[]
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_ADD_WHITELISTED_DEPLOYERS,
    args: {
      account_ids: whitelistedDeployers,
    },
  });
};

export const adminRemoveWhitelistedDeployers = async (
  adminAccount: Account,
  whitelistedDeployers: AccountId[]
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_REMOVE_WHITELISTED_DEPLOYERS,
    args: {
      account_ids: whitelistedDeployers,
    },
  });
};

export const getWhitelistedDeployers = async (): Promise<AccountId[]> => {
  return contractView({
    methodName: READ_METHODS.GET_WHITELISTED_DEPLOYERS,
  });
};

// CONFIG

export const getConfig = async (): Promise<PotDeployerConfig> => {
  return contractView({
    methodName: READ_METHODS.GET_CONFIG,
  });
};

export const getAdmin = async (): Promise<AccountId> => {
  return contractView({
    methodName: READ_METHODS.GET_ADMIN,
  });
};

export const adminUpdateProtocolFeeBasisPoints = async (
  adminAccount: Account,
  protocolFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_UPDATE_PROTOCOL_FEE_BASIS_POINTS,
    args: {
      protocol_fee_basis_points: protocolFeeBasisPoints,
    },
  });
};

export const adminUpdateDefaultChefFeeBasisPoints = async (
  adminAccount: Account,
  defaultChefFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_DEFAULT_CHEF_FEE_BASIS_POINTS,
    args: {
      default_chef_fee_basis_points: defaultChefFeeBasisPoints,
    },
  });
};

export const adminUpdateMaxProtocolFeeBasisPoints = async (
  adminAccount: Account,
  maxProtocolFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_UPDATE_MAX_PROTOCOL_FEE_BASIS_POINTS,
    args: {
      max_protocol_fee_basis_points: maxProtocolFeeBasisPoints,
    },
  });
};

export const adminUpdateMaxChefFeeBasisPoints = async (
  adminAccount: Account,
  maxChefFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_UPDATE_MAX_CHEF_FEE_BASIS_POINTS,
    args: {
      max_chef_fee_basis_points: maxChefFeeBasisPoints,
    },
  });
};

export const adminUpdateMaxRoundTime = async (
  adminAccount: Account,
  maxRoundTime: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_UPDATE_MAX_ROUND_TIME,
    args: {
      max_round_time: maxRoundTime,
    },
  });
};

export const adminUpdateMaxApplicationTime = async (
  adminAccount: Account,
  maxApplicationTime: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_UPDATE_MAX_APPLICATION_TIME,
    args: {
      max_application_time: maxApplicationTime,
    },
  });
};
