import assert from "assert";
import BN from "bn.js";
import { Account } from "near-api-js";
import { DEFAULT_PROJECT_ID, contractId } from "./config";
import { contractId as registryContractId } from "../registry/config";
import { contractId as potDeployerContractId } from "../pot_deployer/config";
import { contractAccount, near } from "./setup";
import {
  adminSetApplicationEndMs,
  adminSetApplicationStartMs,
  apply,
  chefSetApplicationStatus,
  donate,
  getApplicationById,
  getApplications,
  getDonations,
  getDonationsBalance,
  getMatchingPoolBalance,
  getPatronDonations,
  getPotConfig,
  initializeContract,
  patronDonateToMatchingPool,
  unapply,
} from "./utils";
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
} from "../utils/constants";
import { registerProject } from "../registry/utils";
import { POT_DEPLOYER_ALWAYS_ADMIN_ID } from "../pot_deployer/config";
import { parseNearAmount } from "near-api-js/lib/utils/format";

/*
TEST CASES (taken from ../README.md):
- Enforces round_start_ms & round_end_ms
- ✅ Enforces application_start_ms & application_end_ms
- Enforces max_projects
- Enforces supported base_currency
- Enforces donation_requirement (SBT)
- ✅ Project can apply for round
  - ✅ Enforces haven't already applied
  - ⏰ Enforces max_projects not met
  - ✅ Enforces application period open
  - ✅ Enforces registered project
  - Emits event
- ✅ Project can unapply
  - ✅ Enforces application is in Pending status
  - Emits event
- ✅ Chef can update application status
  - ✅ Enforces only chef
  - ✅ Must provide notes (reason)
- ✅ Patron can donate to matching pool
  - ✅ Protocol & chef fees paid out
  - ✅ Referrer paid out
  - ✅ Enforces round not closed
  - Emits event
- End user can donate to specific project
  - Enforces round open
  - Emits event
- End user can donate to all projects
  - Enforces round open
  - Emits events
- PotDeployer Admin (DAO) can change chef & chef fee
- Chef can set (update) the application requirement
- Chef can set (update) the donation requirement
- Chef can update the patron referral fee❓
- Chef can set payouts (CLR / quadratic calculations)
- PotDeployer Admin (DAO) can process payouts
  - Can cooldown period be overridden?
*/

describe("Pot Contract Tests", () => {
  // other accounts
  let projectId = DEFAULT_PROJECT_ID;
  let projectAccount: Account;
  let chefId: AccountId = contractId;
  let chefAccount: Account;
  let potDeployerAdminId: AccountId = POT_DEPLOYER_ALWAYS_ADMIN_ID;
  let potDeployerAdminAccount: Account;
  let patronId: AccountId = contractId; // TODO: change this to use dedicated patron subaccount of contract (will need to move from near dev-deploy in order to do this)
  let patronAccount: Account;

  before(async () => {
    projectAccount = new Account(near.connection, projectId);
    chefAccount = new Account(near.connection, chefId);
    potDeployerAdminAccount = new Account(near.connection, potDeployerAdminId);
    patronAccount = new Account(near.connection, patronId);

    // attempt to initialize contract; if it fails, it's already initialized
    const now = Date.now();
    const defaultPotArgs = {
      created_by: chefId,
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
        // TODO: removing this for now for ease of testing
        registry_id: DEFAULT_REGISTRY_ID,
        issuer_id: DEFAULT_ISSUER_ID,
        class_id: DEFAULT_CLASS_ID,
      },
      patron_referral_fee_basis_points:
        DEFAULT_PATRON_REFERRAL_FEE_BASIS_POINTS,
      max_patron_referral_fee: DEFAULT_MAX_PATRON_REFERRAL_FEE,
      round_manager_fee_basis_points: DEFAULT_ROUND_MANAGER_FEE_BASIS_POINTS,
      protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
      protocol_fee_recipient_account: potDeployerAdminId,
      registry_contract_id: registryContractId,
      pot_deployer_contract_id: potDeployerContractId,
    };
    try {
      // initialize contract unless already initialized
      await initializeContract(defaultPotArgs);
      console.log(`✅ Initialized Pot contract ${contractId}`);
    } catch (e) {
      if (
        JSON.stringify(e).includes("The contract has already been initialized")
      ) {
        console.log(`Pot contract ${contractId} is already initialized`);
      } else {
        console.log("🚨 Pot initialize error: ", e);
        assert(false);
      }
    } finally {
      // join potlock registry unless already joined
      try {
        console.log("📄 Registering project...");
        await registerProject(projectAccount, "New Project", []);
      } catch (e) {
        if (JSON.stringify(e).includes("Project already exists")) {
          console.log("➡️ Project already registered... skipping registration");
        } else {
          console.log("🚨 Error registering project in before hook: ", e);
        }
      }
    }
  });

  // it("Pot Deployer Admin can update application start & end times", async () => {
  //   try {
  //     const now = Date.now();
  //     const start = now;
  //     const end = now + DEFAULT_APPLICATION_LENGTH;
  //     await adminSetApplicationStartMs(potDeployerAdminAccount, start);
  //     let config = await getPotConfig();
  //     assert(config.application_start_ms === start);
  //     await adminSetApplicationEndMs(potDeployerAdminAccount, end);
  //     config = await getPotConfig();
  //     assert(config.application_end_ms === end);
  //   } catch (e) {
  //     console.log("🚨 Error updating application start or end times: ", e);
  //     assert(false);
  //   }
  // });

  // it("Non-Admin CANNOT update application start & end times", async () => {
  //   const now = Date.now();
  //   const start = now;
  //   const end = now + DEFAULT_APPLICATION_LENGTH;
  //   try {
  //     // chef is not an admin; we'll use their account
  //     await adminSetApplicationStartMs(chefAccount, start);
  //     assert(false);
  //   } catch (e) {
  //     assert(JSON.stringify(e).includes("Caller is not admin"));
  //     try {
  //       await adminSetApplicationEndMs(chefAccount, end);
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes("Caller is not admin"));
  //     }
  //   }
  // });

  it("Project can apply if they are on registry & application period is open", async () => {
    try {
      // NB: project account has been added to registry in "before" hook
      // first, try to apply before application period starts
      const start = Date.now() + DEFAULT_APPLICATION_LENGTH;
      await adminSetApplicationStartMs(potDeployerAdminAccount, start);
      try {
        await apply(projectAccount);
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes("Application period is not open"));
        // update application period to start from now
        await adminSetApplicationStartMs(potDeployerAdminAccount, Date.now());
        // try applying; this time it should succeed
        await apply(projectAccount);
        const applications = await getApplications();
        const exists = applications.some((a) => a.project_id === projectId);
        assert(exists);
        // unapply
        await unapply(projectAccount);
        assert(true);
        // reapply
        await apply(projectAccount);
        assert(true);
        // cannot reapply once already applied
        try {
          await apply(projectAccount);
          assert(false);
        } catch (e) {
          assert(JSON.stringify(e).includes("Application already exists"));
          // chef can set/update application status
          const applications = await getApplications();
          let application = applications.find(
            (a) => a.project_id === projectId
          );
          if (!application || application.status !== "Pending") {
            console.log(
              "Error setting application status: No pending application found"
            );
            assert(false);
          }
          const newStatus = "Approved";
          const notes = "LGTM";
          await chefSetApplicationStatus(
            chefAccount,
            application.id,
            newStatus,
            notes
          );
          application = await getApplicationById(application.id);
          assert(application.status === newStatus);
          assert(application.review_notes === notes);
          // non-chef (e.g. project) CANNOT update application status
          try {
            await chefSetApplicationStatus(
              projectAccount,
              application.id,
              newStatus,
              notes
            );
            assert(false);
          } catch (e) {
            assert(
              JSON.stringify(e).includes("Only the chef can call this method")
            );
          }
        }
      }
    } catch (e) {
      console.log("🚨 Error applying for pot:", e);
      assert(false);
    }
  });

  // it("Project CANNOT apply if they are NOT on registry", async () => {
  //   try {
  //     // contract account has not been added to registry
  //     await apply(contractAccount);
  //     assert(false);
  //   } catch (e) {
  //     assert(JSON.stringify(e).includes("Project is not registered"));
  //   }

  //   // it("Enforces round_start_ms & round_end_ms", async () => {
  //   //   // what cannot occur outside of round start/end?
  //   //   // patron CAN donate before round start
  //   //   // patron CANNOT donate after round end
  //   //   // end user CANNOT donate before round start or after round end
  //   //   // chef CANNOT set payouts before round end
  //   //   // base currency CANNOT be changed after round start
  //   //   // ❓ Can Projects be added after round start? (assuming max projects not met)
  //   //   assert(true);
  //   // });
  // });

  // it("Patron can donate to matching pool", async () => {
  //   try {
  //     const message = "Go go go!";
  //     const referrerId = chefId;
  //     const donationAmount = parseNearAmount("1") as string; // 1 NEAR in YoctoNEAR
  //     const potConfig = await getPotConfig();
  //     // const referrerFee = new BN(potConfig.max_patron_referral_fee);
  //     const amountPerBasisPoint = new BN(donationAmount).div(new BN(10_000));
  //     // calculate referrer fee
  //     let referrerFee = amountPerBasisPoint.mul(
  //       new BN(potConfig.patron_referral_fee_basis_points)
  //     );
  //     if (referrerFee > new BN(potConfig.max_patron_referral_fee)) {
  //       referrerFee = new BN(potConfig.max_patron_referral_fee);
  //     }
  //     // calculate protocol fee
  //     let protocolFee = amountPerBasisPoint.mul(
  //       new BN(potConfig.protocol_fee_basis_points)
  //     );
  //     const matchingPoolBalanceBefore = new BN(await getMatchingPoolBalance()); // YoctoNEAR
  //     await patronDonateToMatchingPool({
  //       patronAccount,
  //       donationAmount,
  //       message,
  //       referrerId,
  //     });
  //     const matchingPoolBalanceAfter = new BN(await getMatchingPoolBalance()); // YoctoNEAR
  //     // assert that fees were correctly taken out
  //     assert(
  //       matchingPoolBalanceAfter.sub(matchingPoolBalanceBefore).toString() ===
  //         new BN(donationAmount).sub(referrerFee).sub(protocolFee).toString()
  //     );
  //     // assert that the patron donation record was created
  //     const patronDonations = await getPatronDonations();
  //     let exists = patronDonations.some(
  //       (d) => d.message === message && d.donor_id === patronId
  //     );
  //     assert(exists);
  //     // TODO: verify referrer & protocol fee paid out
  //   } catch (e) {
  //     console.log("error making patron donation: ", e);
  //     assert(false);
  //   }
  // });

  it("End user can donate to specific project", async () => {
    try {
      const applications = await getApplications();
      const application = applications.find((a) => a.status === "Approved");
      if (!application) {
        console.log("No approved application to donate to");
        assert(false);
      }
      const message = "Go go go!";
      const donationAmount = parseNearAmount("1") as string; // 1 NEAR in YoctoNEAR
      const donorAccount = chefAccount;
      const potConfig = await getPotConfig();
      // const referrerFee = new BN(potConfig.max_patron_referral_fee);
      const amountPerBasisPoint = new BN(donationAmount).div(new BN(10_000));
      // calculate protocol fee
      let protocolFee = amountPerBasisPoint.mul(
        new BN(potConfig.protocol_fee_basis_points)
      );
      const donationsBalanceBefore = new BN(await getDonationsBalance()); // YoctoNEAR
      await donate({
        donorAccount,
        applicationId: application.id,
        donationAmount,
        message,
      });
      const donationsBalanceAfter = new BN(await getDonationsBalance()); // YoctoNEAR
      // assert that fees were correctly taken out
      assert(
        donationsBalanceAfter.sub(donationsBalanceBefore).toString() ===
          new BN(donationAmount).sub(protocolFee).toString()
      );
      // assert that the donation record was created
      const donations = await getDonations();
      let exists = donations.some(
        (d) => d.message === message && d.donor_id === donorAccount.accountId
      );
      assert(exists);
    } catch (e) {
      console.log("error making patron donation: ", e);
      assert(false);
    }
  });
});
