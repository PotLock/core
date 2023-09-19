import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";

const READ_METHODS = {
  GET_ADMINS: "get_admins",
  GET_PROJECTS: "get_projects",
  GET_PROJECT_BY_ID: "get_project_by_id",
};

const WRITE_METHODS = {
  NEW: "new",
  OWNER_ADD_ADMINS: "owner_add_admins",
  OWNER_REMOVE_ADMINS: "owner_remove_admins",
  REGISTER: "register",
  ADMIN_SET_PROJECT_STATUS: "admin_set_project_status",
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

// ADMINS

export const ownerAddAdmins = async (
  ownerAccount: Account,
  admins: AccountId[]
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_ADD_ADMINS,
    args: {
      admins,
    },
  });
};

export const ownerRemoveAdmins = async (
  ownerAccount: Account,
  admins: AccountId[]
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_REMOVE_ADMINS,
    args: {
      admins,
    },
  });
};

export const getAdmins = async (): Promise<AccountId[]> => {
  return contractView({
    methodName: READ_METHODS.GET_ADMINS,
  });
};

// PROJECTS

export const registerProject = async (
  callerAccount: Account,
  name: string,
  teamMembers: AccountId[],
  projectId?: string
) => {
  return contractCall({
    callerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.REGISTER,
    args: {
      name,
      team_members: teamMembers,
      ...(projectId ? { _project_id: projectId } : {}),
    },
  });
};

export const getProjects = async (): Promise<Project[]> => {
  return contractView({
    methodName: READ_METHODS.GET_PROJECTS,
  });
};

export const getProjectById = async (
  projectId: AccountId
): Promise<Project> => {
  return contractView({
    methodName: READ_METHODS.GET_PROJECT_BY_ID,
    args: {
      project_id: projectId,
    },
  });
};

export const adminSetProjectStatus = async (
  adminAccount: Account,
  projectId: AccountId,
  status: string,
  reviewNotes: string
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_PROJECT_STATUS,
    args: {
      project_id: projectId,
      status,
      review_notes: reviewNotes,
    },
  });
};
