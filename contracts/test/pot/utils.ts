import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";

const READ_METHODS = {
  IS_ROUND_ACTIVE: "is_round_active",
  GET_APPLICATIONS: "get_applications",
  GET_APPLICATION_BY_ID: "get_application_by_id",
  GET_POT_CONFIG: "get_pot_config",
};

const WRITE_METHODS = {
  NEW: "new",
  APPLY: "apply",
  UNAPPLY: "unapply",
  ADMIN_SET_APPLICATION_START_MS: "admin_set_application_start_ms",
  ADMIN_SET_APPLICATION_END_MS: "admin_set_application_end_ms",
  CHEF_SET_APPLICATION_STATUS: "chef_set_application_status",
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

// CONFIG

export const getPotConfig = async (): Promise<PotConfig> => {
  return contractView({
    methodName: READ_METHODS.GET_POT_CONFIG,
  });
};

// APPLICATIONS

export const apply = async (callerAccount: Account) => {
  return contractCall({
    callerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.APPLY,
  });
};

export const unapply = async (callerAccount: Account) => {
  return contractCall({
    callerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.UNAPPLY,
  });
};

export const getApplications = async (): Promise<Application[]> => {
  return contractView({
    methodName: READ_METHODS.GET_APPLICATIONS,
  });
};

export const getApplicationById = async (
  applicationId: ApplicationId
): Promise<Application> => {
  return contractView({
    methodName: READ_METHODS.GET_APPLICATION_BY_ID,
    args: { application_id: applicationId },
  });
};

// CHEF

export const chefSetApplicationStatus = async (
  chefAccount: Account,
  applicationId: number,
  applicationStatus: string, // ApplicationStatus
  reviewNotes: string
) => {
  return contractCall({
    callerAccount: chefAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.CHEF_SET_APPLICATION_STATUS,
    args: {
      application_id: applicationId,
      status: applicationStatus,
      notes: reviewNotes,
    },
  });
};

// ADMIN

export const adminSetApplicationStartMs = async (
  adminAccount: Account,
  applicationStartMs: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_APPLICATION_START_MS,
    args: { application_start_ms: applicationStartMs },
  });
};

export const adminSetApplicationEndMs = async (
  adminAccount: Account,
  applicationEndMs: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_APPLICATION_END_MS,
    args: { application_end_ms: applicationEndMs },
  });
};

// ROUND

export const isRoundActive = async (): Promise<boolean> => {
  return contractView({
    methodName: READ_METHODS.IS_ROUND_ACTIVE,
  });
};
