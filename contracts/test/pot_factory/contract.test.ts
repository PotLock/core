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
  DEFAULT_REFERRAL_FEE_BASIS_POINTS,
  DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
  DEFAULT_REGISTRY_ID,
  DEFAULT_ROUND_LENGTH,
  DEFAULT_CHEF_FEE_BASIS_POINTS,
} from "../utils/constants";
import { near } from "./setup";
import {
  adminAddWhitelistedDeployers,
  adminRemoveWhitelistedDeployers,
  adminSetDefaultChefFeeBasisPoints,
  adminSetProtocolFeeBasisPoints,
  deployPot,
  getConfig,
  getPots,
  initializeContract,
  slugify,
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
        owner: alwaysAdminId,
        admins: [],
        protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
        protocol_fee_recipient_account: alwaysAdminId,
        default_chef_fee_basis_points: DEFAULT_DEFAULT_CHEF_FEE_BASIS_POINTS,
        whitelisted_deployers: [whitelistedDeployerId],
        require_whitelist: true,
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
    const defaultPotArgs: PotArgs = {
      chef: chefId,
      pot_name: potOnChainName,
      pot_description: "test round description",
      public_round_start_ms: now,
      public_round_end_ms: now + DEFAULT_ROUND_LENGTH,
      application_start_ms: now,
      application_end_ms: now + DEFAULT_APPLICATION_LENGTH, // 1 week
      max_projects: DEFAULT_MAX_PROJECTS,
      sybil_wrapper_provider: "dev-1701729483653-58884486628411:is_human",
      referral_fee_matching_pool_basis_points:
        DEFAULT_REFERRAL_FEE_BASIS_POINTS,
      public_round_referral_fee_basis_points: DEFAULT_REFERRAL_FEE_BASIS_POINTS,
      chef_fee_basis_points: DEFAULT_CHEF_FEE_BASIS_POINTS,
    };
    try {
      // admin can deploy a new pot
      await deployPot(alwaysAdminAccount, defaultPotArgs);

      let pots = await getPots();
      console.log("pots: ", pots);
      const slugifiedName = slugify(potOnChainName);

      let exists = pots.some((p) => {
        console.log("slugified: ", slugifiedName);
        return (
          p.pot_id.startsWith(slugifiedName) &&
          p.deployed_by == alwaysAdminAccount.accountId
        );
      });
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
      exists = pots.some((p) => {
        console.log("slugified: ", slugifiedName);
        return (
          p.pot_id.startsWith(slugifiedName) &&
          p.deployed_by == alwaysAdminAccount.accountId
        );
      });
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
      await adminSetProtocolFeeBasisPoints(
        alwaysAdminAccount,
        newProtocolFeeBasisPoints
      );
      config = await getConfig();
      assert(config.protocol_fee_basis_points == newProtocolFeeBasisPoints);
      // non-admin cannot
      // TODO: wrap/componentize this to remove duplication across tests
      try {
        await adminSetProtocolFeeBasisPoints(
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
      await adminSetDefaultChefFeeBasisPoints(
        alwaysAdminAccount,
        newDefaultChefFeeBasisPoints
      );
      config = await getConfig();
      assert(
        config.default_chef_fee_basis_points == newDefaultChefFeeBasisPoints
      );
      // non-admin cannot
      try {
        await adminSetDefaultChefFeeBasisPoints(
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

  // it("Admin can update max protocol fee basis points (and non-admin cannot)", async () => {
  //   let config = await getConfig();
  //   const newMaxProtocolFeeBasisPoints =
  //     config.max_protocol_fee_basis_points + 1;
  //   try {
  //     await adminUpdateMaxProtocolFeeBasisPoints(
  //       alwaysAdminAccount,
  //       newMaxProtocolFeeBasisPoints
  //     );
  //     config = await getConfig();
  //     assert(
  //       config.max_protocol_fee_basis_points == newMaxProtocolFeeBasisPoints
  //     );
  //     // non-admin cannot
  //     try {
  //       await adminUpdateMaxProtocolFeeBasisPoints(
  //         alwaysNOTAdminAccount,
  //         newMaxProtocolFeeBasisPoints
  //       );
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
  //     }
  //   } catch (e) {
  //     console.log("Error updating max protocol fee basis points:", e);
  //     assert(false);
  //   }
  // });

  // it("Admin can update max chef fee basis points (and non-admin cannot)", async () => {
  //   let config = await getConfig();
  //   const newMaxChefFeeBasisPoints = config.max_chef_fee_basis_points + 1;
  //   try {
  //     await adminUpdateMaxChefFeeBasisPoints(
  //       alwaysAdminAccount,
  //       newMaxChefFeeBasisPoints
  //     );
  //     config = await getConfig();
  //     assert(config.max_chef_fee_basis_points == newMaxChefFeeBasisPoints);
  //     // non-admin cannot
  //     try {
  //       await adminUpdateMaxChefFeeBasisPoints(
  //         alwaysNOTAdminAccount,
  //         newMaxChefFeeBasisPoints
  //       );
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
  //     }
  //   } catch (e) {
  //     console.log("Error updating max chef fee basis points:", e);
  //     assert(false);
  //   }
  // });

  // it("Admin can update max round time (and non-admin cannot)", async () => {
  //   let config = await getConfig();
  //   const newMaxRoundTime = config.max_round_time + 1;
  //   try {
  //     await adminUpdateMaxRoundTime(alwaysAdminAccount, newMaxRoundTime);
  //     config = await getConfig();
  //     assert(config.max_round_time == newMaxRoundTime);
  //     // non-admin cannot
  //     try {
  //       await adminUpdateMaxRoundTime(alwaysNOTAdminAccount, newMaxRoundTime);
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
  //     }
  //   } catch (e) {
  //     console.log("Error updating max round time:", e);
  //     assert(false);
  //   }
  // });

  // it("Admin can update max application time (and non-admin cannot)", async () => {
  //   let config = await getConfig();
  //   const newMaxApplicationTime = config.max_application_time + 1;
  //   try {
  //     await adminUpdateMaxApplicationTime(
  //       alwaysAdminAccount,
  //       newMaxApplicationTime
  //     );
  //     config = await getConfig();
  //     assert(config.max_application_time == newMaxApplicationTime);
  //     // non-admin cannot
  //     try {
  //       await adminUpdateMaxApplicationTime(
  //         alwaysNOTAdminAccount,
  //         newMaxApplicationTime
  //       );
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes(ASSERT_ADMIN_ERROR_STR));
  //     }
  //   } catch (e) {
  //     console.log("Error updating max application time:", e);
  //     assert(false);
  //   }
  // });
});
