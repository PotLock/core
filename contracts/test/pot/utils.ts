import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";
import { parseNearAmount } from "near-api-js/lib/utils/format";

const READ_METHODS = {
  IS_ROUND_ACTIVE: "is_round_active",
  GET_APPLICATIONS: "get_applications",
  GET_APPLICATION_BY_PROJECT_ID: "get_application_by_project_id",
  GET_POT_CONFIG: "get_pot_config",
  GET_DONATIONS_BALANCE: "get_donations_balance",
  GET_MATCHING_POOL_BALANCE: "get_matching_pool_balance",
  GET_PATRON_DONATIONS: "get_patron_donations",
  GET_DONATIONS: "get_donations",
};

const WRITE_METHODS = {
  NEW: "new",
  APPLY: "apply",
  UNAPPLY: "unapply",
  ADMIN_SET_APPLICATION_START_MS: "admin_set_application_start_ms",
  ADMIN_SET_APPLICATION_END_MS: "admin_set_application_end_ms",
  ADMIN_SET_CHEF: "admin_set_chef",
  ADMIN_SET_CHEF_FEE_BASIS_POINTS: "admin_set_chef_fee_basis_points",
  CHEF_SET_APPLICATION_STATUS: "chef_set_application_status",
  CHEF_SET_DONATION_REQUIREMENT: "chef_set_donation_requirement",
  PATRON_DONATE_TO_MATCHING_POOL: "patron_donate_to_matching_pool",
  DONATE: "donate",
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

export const getApplicationByProjectId = async (
  projectId: ProjectId
): Promise<Application> => {
  return contractView({
    methodName: READ_METHODS.GET_APPLICATION_BY_PROJECT_ID,
    args: { project_id: projectId },
  });
};

// CHEF

export const chefSetApplicationStatus = async (
  chefAccount: Account,
  projectId: ProjectId,
  applicationStatus: string, // ApplicationStatus
  reviewNotes: string
) => {
  return contractCall({
    callerAccount: chefAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.CHEF_SET_APPLICATION_STATUS,
    args: {
      project_id: projectId,
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

export const adminSetChef = async (
  adminAccount: Account,
  chefId: AccountId
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_CHEF,
    args: { chef_id: chefId },
  });
};

export const adminSetChefFeeBasisPoints = async (
  adminAccount: Account,
  chefFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: adminAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.ADMIN_SET_CHEF_FEE_BASIS_POINTS,
    args: { chef_fee_basis_points: chefFeeBasisPoints },
  });
};

// PATRON / MATCHING POOL

export const patronDonateToMatchingPool = async ({
  patronAccount,
  donationAmount,
  message,
  referrerId,
}: {
  patronAccount: Account;
  donationAmount: string;
  message?: string;
  referrerId?: AccountId;
}) => {
  return contractCall({
    callerAccount: patronAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.PATRON_DONATE_TO_MATCHING_POOL,
    args: {
      message: message || null,
      referrer_id: referrerId || null,
    },
    attachedDeposit: donationAmount,
  });
};

export const getPatronDonations = async (): Promise<PatronDonation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_PATRON_DONATIONS,
  });
};

export const getMatchingPoolBalance = async (): Promise<string> => {
  return contractView({
    methodName: READ_METHODS.GET_MATCHING_POOL_BALANCE,
  });
};

// DONATIONS

export const donate = async ({
  donorAccount,
  projectId,
  donationAmount,
  message,
}: {
  donorAccount: Account;
  projectId: number | null; // If null, donation will be split among all approved applications
  donationAmount: string;
  message?: string;
}) => {
  return contractCall({
    callerAccount: donorAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.DONATE,
    args: {
      project_id: projectId,
      message: message || null,
    },
    attachedDeposit: donationAmount,
  });
};

export const getDonationsBalance = async (): Promise<string> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS_BALANCE,
  });
};

export const getDonations = async (): Promise<Donation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS,
  });
};

export const chefSetDonationRequirement = async (
  chefAccount: Account,
  donationRequirement: SBTRequirement | null
) => {
  return contractCall({
    callerAccount: chefAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.CHEF_SET_DONATION_REQUIREMENT,
    args: { donation_requirement: donationRequirement },
  });
};

// ROUND

export const isRoundActive = async (): Promise<boolean> => {
  return contractView({
    methodName: READ_METHODS.IS_ROUND_ACTIVE,
  });
};
