import assert from "assert";
import BN from "bn.js";
import { Account } from "near-api-js";
import { contractId } from "./config";
import { contractId as registryContractId } from "../registry/config";
import { contractId as potDeployerContractId } from "../pot_factory/config";
import {
  contractAccount,
  // getChefAccount,
  // getPatronAccount,
  // getProjectAccounts,
  near,
} from "./setup";
import { donate, getDonations, initializeContract } from "./utils";
import {
  DEFAULT_PARENT_ACCOUNT_ID,
  DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_REFERRAL_FEE_BASIS_POINTS,
} from "../utils/constants";
import { registerProject } from "../registry/utils";
import { POT_FACTORY_ALWAYS_ADMIN_ID } from "../pot_factory/config";
import { parseNearAmount } from "near-api-js/lib/utils/format";
import {
  convertDonationsToProjectContributions,
  calculateQuadraticPayouts,
} from "../utils/quadratics";

/*
TEST CASES (taken from ../README.md):

Donation
- ✅ User can donate
- Owner can change the owner
- Owner can set protocol_fee_basis_points
- Owner can set referral_fee_basis_points
- Owner can set protocol_fee_recipient_account
*/

describe("Donation Contract Tests", async () => {
  // other accounts
  let donorAccountId = contractId;
  let donorAccount: Account;
  // let projectAccounts: Account[];
  // // let chefId: AccountId; // TODO:
  // let chefAccount: Account;
  // let potDeployerAdminId: AccountId = POT_DEPLOYER_ALWAYS_ADMIN_ID;
  // let potDeployerAdminAccount: Account;
  // let patronAccount: Account;

  before(async () => {
    // projectAccount = new Account(near.connection, projectId);
    donorAccount = new Account(near.connection, donorAccountId);

    // attempt to initialize contract; if it fails, it's already initialized
    const now = Date.now();
    const defaultDonationInitArgs = {
      owner: contractId,
      protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
      referral_fee_basis_points: DEFAULT_REFERRAL_FEE_BASIS_POINTS,
      protocol_fee_recipient_account: DEFAULT_PARENT_ACCOUNT_ID,
    };
    try {
      // initialize contract unless already initialized
      await initializeContract(defaultDonationInitArgs);
      console.log(`✅ Initialized Donation contract ${contractId}`);
    } catch (e) {
      if (
        JSON.stringify(e).includes("The contract has already been initialized")
      ) {
        console.log(`Donation contract ${contractId} is already initialized`);
      } else {
        console.log("🚨 Donation initialize error: ", e);
        assert(false);
      }
    }
  });

  it("User can donate", async () => {
    try {
      const message = "Go go go!";
      const referrerId = contractId;
      const donationAmount = parseNearAmount("0.1") as string; // 0.1 NEAR in YoctoNEAR
      await donate({
        donorAccount,
        recipientId: DEFAULT_PARENT_ACCOUNT_ID,
        donationAmount,
        message,
        referrerId,
      });
      // get donations
      const donations = await getDonations();
      console.log("donations: ", donations);
      // assert that donation record was created
      const exists = donations.some(
        (d) => d.message === message && d.donor_id === donorAccountId
      );
      assert(exists);
    } catch (e) {
      console.log("🚨 Error donating: ", e);
      assert(false);
    }
  });
});
