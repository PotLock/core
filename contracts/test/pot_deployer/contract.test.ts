import assert from "assert";
import { Account, utils } from "near-api-js";
import {
  ASSERT_ADMIN_ERROR_STR,
  ASSERT_ADMIN_OR_WHITELISTED_DEPLOYER_ERROR_STR,
  DEFAULT_WHITELISTED_DEPLOYER_ID,
  POT_DEPLOYER_ALWAYS_ADMIN_ID,
  contractId,
} from "./config";
import {
  DEFAULT_APPLICATION_LENGTH,
  DEFAULT_BASE_CURRENCY,
  DEFAULT_CLASS_ID,
  DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS,
  DEFAULT_ISSUER_ID,
  DEFAULT_MAX_APPLICATION_TIME,
  DEFAULT_MAX_CHEF_FEE_BASIS_POINTS,
  DEFAULT_MAX_PROJECTS,
  DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_MAX_ROUND_TIME,
  DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS,
  DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_REGISTRY_ID,
  DEFAULT_ROUND_LENGTH,
  DEFAULT_chef_fee_basis_points,
} from "../utils/constants";
import { near } from "./setup";
import {
  adminAddWhitelistedDeployers,
  adminRemoveWhitelistedDeployers,
  adminUpdateDefaultChefFeeBasisPoints,
  adminUpdateMaxApplicationTime,
  adminUpdateMaxChefFeeBasisPoints,
  adminUpdateMaxProtocolFeeBasisPoints,
  adminUpdateMaxRoundTime,
  adminUpdateProtocolFeeBasisPoints,
  deployPot,
  getConfig,
  getPots,
  initializeContract,
} from "./utils";

/*
TEST CASES (taken from ../README.md):
- Only admin or whitelisted_deployer can deploy a new Pot
  - TODO: Specified chef must have "chef" role in ReFi DAO
- Admin (DAO) can:
  - Update protocol fee basis points (must be <= max_protocol_fee_basis_points)
  - Update default chef fee basis points (must be <= default_chef_fee_basis_points)
  - Update max protocol fee basis points
  - Update max chef fee basis points
  - Update max round time
  - Update max application time
  - Add whitelisted deployers
  - Remove whitelisted deployers
*/

// TODO: CREATE MORE ACCOUNTS ON SETUP

describe("PotDelpoyer Contract Tests", () => {
  // account that will always be admin for the duration of these tests
  const alwaysAdminId: AccountId = POT_DEPLOYER_ALWAYS_ADMIN_ID;
  let alwaysAdminAccount: Account;

  // account that will always NOT be admin for the duration of these tests
  // we can use the whitelisted deployer for this purpose
  const alwaysNOTAdminAccountId: AccountId = DEFAULT_WHITELISTED_DEPLOYER_ID;
  let alwaysNOTAdminAccount: Account;

  // other accounts
  let chefId: AccountId = contractId;
  let chefAccount: Account;
  let whitelistedDeployerId: AccountId = DEFAULT_WHITELISTED_DEPLOYER_ID;
  let whitelistedDeployerAccount: Account;

  before(async () => {
    alwaysAdminAccount = new Account(near.connection, alwaysAdminId);
    alwaysNOTAdminAccount = new Account(
      near.connection,
      alwaysNOTAdminAccountId
    );
    chefAccount = new Account(near.connection, chefId);
    whitelistedDeployerAccount = new Account(
      near.connection,
      whitelistedDeployerId
    );

    // attempt to initialize contract; if it fails, it's already initialized
    try {
      await initializeContract({
        max_round_time: DEFAULT_MAX_ROUND_TIME,
        max_application_time: DEFAULT_MAX_APPLICATION_TIME,
        protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
        max_protocol_fee_basis_points: DEFAULT_MAX_PROTOCOL_FEE_BASIS_POINTS,
        default_chef_fee_basis_points: DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS,
        max_chef_fee_basis_points: DEFAULT_MAX_CHEF_FEE_BASIS_POINTS,
        admin: alwaysAdminId,
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
    let potOnChainName = "test pot";
    const now = Date.now();
    const defaultPotArgs = {
      chef: chefId,
      pot_name: "test round",
      pot_description: "test round description",
      public_round_start_ms: now,
      public_round_end_ms: now + DEFAULT_ROUND_LENGTH,
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
      chef_fee_basis_points: DEFAULT_chef_fee_basis_points,
      protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
      protocol_fee_recipient_account: alwaysAdminId,
    };
    try {
      // admin can deploy a new pot
      await deployPot(alwaysAdminAccount, potOnChainName, defaultPotArgs);

      let pots = await getPots();
      let exists = pots.some(
        (p) =>
          p.on_chain_name === potOnChainName &&
          p.deployed_by == alwaysAdminAccount.accountId
      );
      assert(exists);

      // whitelisted deployer can deploy a new pot
      // admin can add whitelisted deployers
      await adminAddWhitelistedDeployers(alwaysAdminAccount, [
        whitelistedDeployerId,
      ]);
      potOnChainName += Date.now();
      await deployPot(
        whitelistedDeployerAccount,
        potOnChainName,
        defaultPotArgs
      );
      pots = await getPots();
      exists = pots.some(
        (p) =>
          p.on_chain_name === potOnChainName &&
          p.deployed_by == whitelistedDeployerAccount.accountId
      );
      assert(exists);

      // non-whitelisted deployer cannot deploy a new pot
      // admin can remove whitelisted deployers
      await adminRemoveWhitelistedDeployers(alwaysAdminAccount, [
        whitelistedDeployerId,
      ]);
      try {
        await deployPot(
          whitelistedDeployerAccount,
          potOnChainName,
          defaultPotArgs
        );
        assert(false);
      } catch (e) {
        assert(
          JSON.stringify(e).includes(
            ASSERT_ADMIN_OR_WHITELISTED_DEPLOYER_ERROR_STR
          )
        );
      }
    } catch (e) {
      console.log("Error deploying pot:", e);
      assert(false);
    }
  });

  it("Admin can update protocol fee basis points (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newProtocolFeeBasisPoints = config.protocol_fee_basis_points + 1;
    try {
      await adminUpdateProtocolFeeBasisPoints(
        alwaysAdminAccount,
        newProtocolFeeBasisPoints
      );
      config = await getConfig();
      assert(config.protocol_fee_basis_points == newProtocolFeeBasisPoints);
      // non-admin cannot
      // TODO: wrap/componentize this to remove duplication across tests
      try {
        await adminUpdateProtocolFeeBasisPoints(
          alwaysNOTAdminAccount,
          newProtocolFeeBasisPoints
        );
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating protocol fee basis points:", e);
      assert(false);
    }
  });

  it("Admin can update default chef fee basis points (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newDefaultChefFeeBasisPoints =
      config.default_chef_fee_basis_points + 1;
    try {
      await adminUpdateDefaultChefFeeBasisPoints(
        alwaysAdminAccount,
        newDefaultChefFeeBasisPoints
      );
      config = await getConfig();
      assert(
        config.default_chef_fee_basis_points == newDefaultChefFeeBasisPoints
      );
      // non-admin cannot
      try {
        await adminUpdateDefaultChefFeeBasisPoints(
          alwaysNOTAdminAccount,
          newDefaultChefFeeBasisPoints
        );
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating default chef fee basis points:", e);
      assert(false);
    }
  });

  it("Admin can update max protocol fee basis points (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newMaxProtocolFeeBasisPoints =
      config.max_protocol_fee_basis_points + 1;
    try {
      await adminUpdateMaxProtocolFeeBasisPoints(
        alwaysAdminAccount,
        newMaxProtocolFeeBasisPoints
      );
      config = await getConfig();
      assert(
        config.max_protocol_fee_basis_points == newMaxProtocolFeeBasisPoints
      );
      // non-admin cannot
      try {
        await adminUpdateMaxProtocolFeeBasisPoints(
          alwaysNOTAdminAccount,
          newMaxProtocolFeeBasisPoints
        );
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating max protocol fee basis points:", e);
      assert(false);
    }
  });

  it("Admin can update max chef fee basis points (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newMaxChefFeeBasisPoints = config.max_chef_fee_basis_points + 1;
    try {
      await adminUpdateMaxChefFeeBasisPoints(
        alwaysAdminAccount,
        newMaxChefFeeBasisPoints
      );
      config = await getConfig();
      assert(config.max_chef_fee_basis_points == newMaxChefFeeBasisPoints);
      // non-admin cannot
      try {
        await adminUpdateMaxChefFeeBasisPoints(
          alwaysNOTAdminAccount,
          newMaxChefFeeBasisPoints
        );
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating max chef fee basis points:", e);
      assert(false);
    }
  });

  it("Admin can update max round time (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newMaxRoundTime = config.max_round_time + 1;
    try {
      await adminUpdateMaxRoundTime(alwaysAdminAccount, newMaxRoundTime);
      config = await getConfig();
      assert(config.max_round_time == newMaxRoundTime);
      // non-admin cannot
      try {
        await adminUpdateMaxRoundTime(alwaysNOTAdminAccount, newMaxRoundTime);
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating max round time:", e);
      assert(false);
    }
  });

  it("Admin can update max application time (and non-admin cannot)", async () => {
    let config = await getConfig();
    const newMaxApplicationTime = config.max_application_time + 1;
    try {
      await adminUpdateMaxApplicationTime(
        alwaysAdminAccount,
        newMaxApplicationTime
      );
      config = await getConfig();
      assert(config.max_application_time == newMaxApplicationTime);
      // non-admin cannot
      try {
        await adminUpdateMaxApplicationTime(
          alwaysNOTAdminAccount,
          newMaxApplicationTime
        );
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
      }
    } catch (e) {
      console.log("Error updating max application time:", e);
      assert(false);
    }
  });
});
