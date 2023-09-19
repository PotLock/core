import assert from "assert";
import { Account, utils } from "near-api-js";
import {
  DEFAULT_APPLICATION_LENGTH,
  DEFAULT_BASE_CURRENCY,
  DEFAULT_CLASS_ID,
  DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS,
  DEFAULT_ISSUER_ID,
  DEFAULT_MAX_APPLICATION_TIME,
  DEFAULT_MAX_CHEF_FEE_BASIS_POINTS,
  DEFAULT_MAX_PATRON_REFERRAL_FEE,
  DEFAULT_MAX_PROJECTS,
  DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_MAX_ROUND_TIME,
  DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS,
  DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_REGISTRY_ID,
  DEFAULT_ROUND_LENGTH,
  DEFAULT_ROUND_MANAGER_FEE_BASIS_POINTS,
  contractId,
} from "./config";
import { near } from "./setup";
import { deployPot, getPots, initializeContract } from "./utils";

/*
TEST CASES (taken from ../README.md):
- Only admin or whitelisted_deployer can deploy a new Pot
  - Specified chef must have "chef" role in ReFi DAO
- Admin (DAO) can:
  - Update protocol fee basis points (must be <= max_protocol_fee_basis_points)
  - Update default chef fee basis points (must be <= default_chef_fee_basis_points)
  - Update max protocol fee basis points
  - Update max chef fee basis points
  - Update max round time
  - Update max application time
  - Update max milestones
  - Add whitelisted deployers
*/

describe("PotDelpoyer Contract Tests", () => {
  let adminId: AccountId = contractId;
  let adminAccount: Account;
  let chefId: AccountId = contractId;
  let chefAccount: Account;

  before(async () => {
    adminAccount = new Account(near.connection, adminId);
    chefAccount = new Account(near.connection, chefId);

    // attempt to initialize contract; if it fails, it's already initialized
    try {
      await initializeContract({
        max_round_time: DEFAULT_MAX_ROUND_TIME,
        max_application_time: DEFAULT_MAX_APPLICATION_TIME,
        protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
        max_protocol_fee_basis_points: DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS,
        default_chef_fee_basis_points: DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS,
        max_chef_fee_basis_points: DEFAULT_MAX_CHEF_FEE_BASIS_POINTS,
        admin: adminId,
      });
      console.log(`✅ Initialized PotDeployer contract ${contractId}`);
    } catch (e) {
      if (
        JSON.stringify(e).includes("The contract has already been initialized")
      ) {
        console.log(
          `PotDeployer contract ${contractId} is already initialized`
        );
      } else {
        console.log("PotDeployer initialize error: ", e);
        assert(false);
      }
    }
  });

  it("Only Admin or whitelisted_deployer can deploy a new Pot", async () => {
    const potOnChainName = "test pot";
    const now = Date.now();
    try {
      await deployPot(adminAccount, potOnChainName, {
        chef_id: chefId,
        round_name: "test round",
        round_description: "test round description",
        round_start_ms: now,
        round_end_ms: now + DEFAULT_ROUND_LENGTH,
        application_start_ms: now,
        application_end_ms: now + DEFAULT_APPLICATION_LENGTH, // 1 week
        max_projects: DEFAULT_MAX_PROJECTS,
        base_currency: DEFAULT_BASE_CURRENCY,
        donation_requirement: {
          registry_id: DEFAULT_REGISTRY_ID,
          issuer_id: DEFAULT_ISSUER_ID,
          class_id: DEFAULT_CLASS_ID,
        },
        patron_referral_fee_basis_points:
          DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS,
        max_patron_referral_fee: DEFAULT_MAX_PATRON_REFERRAL_FEE,
        round_manager_fee_basis_points: DEFAULT_ROUND_MANAGER_FEE_BASIS_POINTS,
        protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
      });

      const pots = await getPots();
      console.log("pots line 103: ", pots);
      assert(true);
    } catch (e) {
      console.log("Error deploying pot:", e);
      assert(false);
    }
  });
});
