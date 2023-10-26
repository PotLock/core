import { _contractCall, _contractView } from "../utils/helpers";
import { contractId as _contractId } from "./config";
import { Account } from "near-api-js";
import { contractAccount } from "./setup";
import { NO_DEPOSIT } from "../utils/constants";
import { parseNearAmount } from "near-api-js/lib/utils/format";

const READ_METHODS = {
  GET_DONATIONS: "get_donations",
  GET_DONATION_BY_ID: "get_donation_by_id",
  GET_DONATIONS_FOR_RECIPIENT: "get_donations_for_recipient",
  GET_DONATIONS_FOR_DONOR: "get_donations_for_donor",
  GET_DONATIONS_FOR_FT: "get_donations_for_ft",
};

const WRITE_METHODS = {
  NEW: "new",
  DONATE: "donate",
  OWNER_CHANGE_OWNER: "owner_change_owner",
  OWNER_SET_PROTOCOL_FEE_BASIS_POINTS: "owner_set_protocol_fee_basis_points",
  OWNER_SET_REFERRAL_FEE_BASIS_POINTS: "owner_set_referral_fee_basis_points",
  OWNER_SET_PROTOCOL_FEE_RECIPIENT_ACCOUNT:
    "owner_set_protocol_fee_recipient_account",
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

export const getDonations = async (): Promise<Donation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS,
  });
};

export const getDonationById = async (
  donationId: DonationId
): Promise<Donation> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATION_BY_ID,
    args: { donation_id: donationId },
  });
};

export const getDonationsForRecipient = async (
  recipientId: AccountId
): Promise<Donation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS_FOR_RECIPIENT,
    args: { recipient_id: recipientId },
  });
};

export const getDonationsForDonor = async (
  donorId: AccountId
): Promise<Donation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS_FOR_DONOR,
    args: { donor_id: donorId },
  });
};

export const getDonationsForFt = async (
  ftId: AccountId
): Promise<Donation[]> => {
  return contractView({
    methodName: READ_METHODS.GET_DONATIONS_FOR_FT,
    args: { ft_id: ftId },
  });
};

export const donate = async ({
  donorAccount,
  recipientId,
  donationAmount,
  message,
  referrerId,
}: {
  donorAccount: Account;
  recipientId: AccountId;
  donationAmount: string;
  message?: string;
  referrerId?: AccountId;
}) => {
  return contractCall({
    callerAccount: donorAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.DONATE,
    args: {
      recipient_id: recipientId,
      message: message || null,
      referrer_id: referrerId || null,
    },
    attachedDeposit: donationAmount,
  });
};

export const ownerChangeOwner = async (
  ownerAccount: Account,
  newOwner: AccountId
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_CHANGE_OWNER,
    args: { owner: newOwner },
  });
};

export const ownerSetProtocolFeeBasisPoints = async (
  ownerAccount: Account,
  protocolFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_SET_PROTOCOL_FEE_BASIS_POINTS,
    args: { protocol_fee_basis_points: protocolFeeBasisPoints },
  });
};

export const ownerSetReferralFeeBasisPoints = async (
  ownerAccount: Account,
  referralFeeBasisPoints: number
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_SET_REFERRAL_FEE_BASIS_POINTS,
    args: { referral_fee_basis_points: referralFeeBasisPoints },
  });
};

export const ownerSetProtocolFeeRecipientAccount = async (
  ownerAccount: Account,
  protocolFeeRecipientAccount: AccountId
) => {
  return contractCall({
    callerAccount: ownerAccount,
    contractId: _contractId,
    methodName: WRITE_METHODS.OWNER_SET_PROTOCOL_FEE_RECIPIENT_ACCOUNT,
    args: { protocol_fee_recipient_account: protocolFeeRecipientAccount },
  });
};
