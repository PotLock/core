import assert from "assert";
import BN from "bn.js";
import { Account } from "near-api-js";
import { DEFAULT_PROJECT_ID, contractId } from "./config";
import { contractId as registryContractId } from "../registry/config";
import { contractId as potDeployerContractId } from "../pot_factory/config";
import {
  contractAccount,
  getChefAccount,
  getPatronAccount,
  getProjectAccounts,
  near,
} from "./setup";
import {
  adminCloseRound,
  adminSetApplicationEndMs,
  adminSetApplicationStartMs,
  adminSetChef,
  adminSetChefFeeBasisPoints,
  adminSetRoundOpen,
  apply,
  adminProcessPayouts,
  chefSetApplicationStatus,
  chefSetDonationRequirement,
  chefSetPayouts,
  donate,
  getApplicationByProjectId,
  getApplications,
  getDonations,
  getDonationsBalance,
  getMatchingPoolBalance,
  getPatronDonations,
  getPayouts,
  getPotConfig,
  initializeContract,
  patronDonateToMatchingPool,
  unapply,
  adminSetCooldownPeriodComplete,
} from "./utils";
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
import { registerProject } from "../registry/utils";
import { POT_FACTORY_ALWAYS_ADMIN_ID } from "../pot_factory/config";
import { parseNearAmount } from "near-api-js/lib/utils/format";
import {
  convertDonationsToProjectContributions,
  calculateQuadraticPayouts,
} from "../utils/quadratics";

/*
TEST CASES (taken from ../README.md):
- Enforces public_round_start_ms & public_round_end_ms
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
- ✅ End user can donate to specific project
  - ✅ Enforces round open
  - Emits event
- ✅ End user can donate to all projects
  - ✅ Enforces round open
  - Emits events
- ✅ PotDeployer Admin (DAO) can change chef & chef fee basis points
- ✅ Chef can set (update) the donation requirement
- Chef can update the patron referral fee❓
- ✅ Chef can set payouts (CLR / quadratic calculations)
- ✅ PotDeployer Admin (DAO) can process payouts
  - ✅ Enforces cooldown period ended
  - ✅ Admin can end cooldown period manually
*/

describe("Pot Contract Tests", async () => {
  // other accounts
  // let projectId = DEFAULT_PROJECT_ID;
  // let projectAccount: Account;
  let projectAccounts: Account[];
  // let chefId: AccountId; // TODO:
  let chefAccount: Account;
  let potDeployerAdminId: AccountId = POT_FACTORY_ALWAYS_ADMIN_ID;
  let potDeployerAdminAccount: Account;
  let patronAccount: Account;

  // before(async () => {
  //   // projectAccount = new Account(near.connection, projectId);
  //   chefAccount = await getChefAccount();
  //   potDeployerAdminAccount = new Account(near.connection, potDeployerAdminId);
  //   patronAccount = await getPatronAccount();
  //   projectAccounts = await getProjectAccounts();

  //   // attempt to initialize contract; if it fails, it's already initialized
  //   const now = Date.now();
  //   const defaultPotArgs = {
  //     // deployed_by: chefAccount.accountId,
  //     chef: chefAccount.accountId,
  //     pot_name: "test round",
  //     pot_description: "test round description",
  //     max_projects: DEFAULT_MAX_PROJECTS,
  //     application_start_ms: now,
  //     application_end_ms: now + DEFAULT_APPLICATION_LENGTH, // 1 week
  //     public_round_start_ms: now,
  //     public_round_end_ms: now + DEFAULT_ROUND_LENGTH,

  //     referral_fee_basis_points: DEFAULT_REFERRAL_FEE_BASIS_POINTS,
  //     chef_fee_basis_points: DEFAULT_CHEF_FEE_BASIS_POINTS,
  //     protocol_fee_basis_points: DEFAULT_PROTOCOL_FEE_BASIS_POINTS,
  //     protocol_fee_recipient_account: potDeployerAdminId,
  //     registry_contract_id: registryContractId,
  //   };
  //   try {
  //     // initialize contract unless already initialized
  //     await initializeContract(defaultPotArgs);
  //     console.log(`✅ Initialized Pot contract ${contractId}`);
  //   } catch (e) {
  //     if (
  //       JSON.stringify(e).includes("The contract has already been initialized")
  //     ) {
  //       console.log(`Pot contract ${contractId} is already initialized`);
  //     } else {
  //       console.log("🚨 Pot initialize error: ", e);
  //       assert(false);
  //     }
  //   } finally {
  //     // join potlock registry unless already joined
  //     for (const account of projectAccounts) {
  //       try {
  //         console.log("📄 Registering project " + account.accountId);
  //         await registerProject(account, "New Project");
  //       } catch (e) {
  //         if (JSON.stringify(e).includes("Project already exists")) {
  //           console.log(
  //             `➡️ Project ${account.accountId} already registered... skipping registration`
  //           );
  //           continue;
  //         } else {
  //           console.log("🚨 Error registering project in before hook: ", e);
  //           assert(false);
  //         }
  //       }
  //     }
  //   }
  // });

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

  // it("Pot Deployer Admin can update chef", async () => {
  //   try {
  //     const newChef = "new-chef.testnet";
  //     await adminSetChef(potDeployerAdminAccount, newChef);
  //     let config = await getPotConfig();
  //     assert(config.chef === newChef);
  //     // change back to original chef
  //     await adminSetChef(potDeployerAdminAccount, chefAccount.accountId);
  //     // non-admin CANNOT set chef
  //     try {
  //       await adminSetChef(projectAccounts[0], newChef);
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes("Caller is not admin"));
  //     }
  //   } catch (e) {
  //     console.log("🚨 Error updating chef: ", e);
  //     assert(false);
  //   }
  // });

  // it("Pot Deployer Admin can update chef fee basis points", async () => {
  //   try {
  //     let config = await getPotConfig();
  //     let originalBasisPoints = config.chef_fee_basis_points;
  //     let updatedBasisPoints = config.chef_fee_basis_points + 1;
  //     await adminSetChefFeeBasisPoints(
  //       potDeployerAdminAccount,
  //       updatedBasisPoints
  //     );
  //     config = await getPotConfig();
  //     assert(config.chef_fee_basis_points === updatedBasisPoints);
  //     // change back to original
  //     await adminSetChefFeeBasisPoints(
  //       potDeployerAdminAccount,
  //       originalBasisPoints
  //     );
  //     // non-admin CANNOT set chef fee basis points
  //     try {
  //       await adminSetChefFeeBasisPoints(
  //         projectAccounts[0],
  //         updatedBasisPoints
  //       );
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes("Caller is not admin"));
  //     }
  //   } catch (e) {
  //     console.log("🚨 Error updating chef fee basis points: ", e);
  //     assert(false);
  //   }
  // });

  // it("Chef can update donation requirement", async () => {
  //   try {
  //     const originalConfig = await getPotConfig();
  //     const newDonationRequirement: SBTRequirement = {
  //       registry_id: "new-registry-id.testnet",
  //       issuer_id: "new-issuer-id.testnet",
  //       class_id: 1,
  //     };
  //     await chefSetDonationRequirement(chefAccount, newDonationRequirement);
  //     let config = await getPotConfig();
  //     assert.deepEqual(config.donation_requirement, newDonationRequirement);
  //     // change back to original donationRequirement
  //     await chefSetDonationRequirement(
  //       chefAccount,
  //       originalConfig.donation_requirement
  //     );
  //     config = await getPotConfig();
  //     assert.deepEqual(
  //       config.donation_requirement,
  //       originalConfig.donation_requirement
  //     );
  //     // non-chef CANNOT update donation requirement
  //     try {
  //       await chefSetDonationRequirement(
  //         projectAccounts[0],
  //         newDonationRequirement
  //       );
  //       assert(false);
  //     } catch (e) {
  //       assert(
  //         JSON.stringify(e).includes("Only the chef can call this method")
  //       );
  //     }
  //   } catch (e) {
  //     console.log("🚨 Error updating donation requirement: ", e);
  //     assert(false);
  //   }
  // });

  // it("Project can apply if they are on registry & application period is open", async () => {
  //   try {
  //     // NB: project account has been added to registry in "before" hook
  //     // first, try to apply before application period starts
  //     const start = Date.now() + DEFAULT_APPLICATION_LENGTH;
  //     await adminSetApplicationStartMs(potDeployerAdminAccount, start);
  //     const projectAccount = projectAccounts[0];
  //     try {
  //       await apply(projectAccount);
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes("Application period is not open"));
  //       // update application period to start from now
  //       await adminSetApplicationStartMs(potDeployerAdminAccount, Date.now());
  //       // try applying; this time it should succeed
  //       // apply for all accounts
  //       for (const account of projectAccounts) {
  //         console.log("applying for project " + account.accountId);
  //         await apply(account);
  //       }
  //       const applications = await getApplications();
  //       console.log("applications line 222: ", applications);
  //       console.log(
  //         "projectAccount.accountId line 223: ",
  //         projectAccount.accountId
  //       );
  //       console.log("chefAccount.accountId line 224: ", chefAccount.accountId);
  //       const exists = applications.some(
  //         (a) => a.project_id === projectAccount.accountId
  //       );
  //       assert(exists);
  //       // unapply
  //       await unapply(projectAccount);
  //       assert(true);
  //       // reapply
  //       await apply(projectAccount);
  //       assert(true);
  //       // cannot reapply once already applied
  //       try {
  //         await apply(projectAccount);
  //         assert(false);
  //       } catch (e) {
  //         assert(JSON.stringify(e).includes("Application already exists"));
  //         // non-chef CANNOT set/update application status
  //         const applications = await getApplications();
  //         const newStatus = "Approved";
  //         const notes = "LGTM";
  //         try {
  //           await chefSetApplicationStatus(
  //             projectAccount,
  //             applications[0].project_id,
  //             newStatus,
  //             notes
  //           );
  //           assert(false);
  //         } catch (e) {
  //           assert(
  //             JSON.stringify(e).includes("Only the chef can call this method") // TODO: make this error message a constant
  //           );
  //           // chef can set/update application status
  //           for (const application of applications) {
  //             // approve application
  //             await chefSetApplicationStatus(
  //               chefAccount,
  //               application.project_id,
  //               newStatus,
  //               notes
  //             );
  //             const app = await getApplicationByProjectId(
  //               application.project_id
  //             );
  //             assert(app.status === newStatus);
  //             assert(app.review_notes === notes);
  //           }
  //         }
  //       }
  //     }
  //   } catch (e) {
  //     console.log("🚨 Error applying for pot:", e);
  //     assert(false);
  //   }
  // });

  // it("Project CANNOT apply if they are NOT on registry", async () => {
  //   try {
  //     // contract account has not been added to registry
  //     await apply(contractAccount);
  //     assert(false);
  //   } catch (e) {
  //     assert(JSON.stringify(e).includes("Project is not registered"));
  //   }

  //   // it("Enforces public_round_start_ms & public_round_end_ms", async () => {
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
  //     const referrerId = chefAccount.accountId;
  //     const donationAmount = parseNearAmount("1") as string; // 1 NEAR in YoctoNEAR
  //     const potConfig = await getPotConfig();
  //     const amountPerBasisPoint = new BN(donationAmount).div(new BN(10_000));
  //     // calculate referrer fee
  //     let referrerFee = amountPerBasisPoint.mul(
  //       new BN(potConfig.referral_fee_basis_points)
  //     );
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
  //       (d) => d.message === message && d.donor_id === patronAccount.accountId
  //     );
  //     assert(exists);
  //     // TODO: verify referrer & protocol fee paid out
  //   } catch (e) {
  //     console.log("error making patron donation: ", e);
  //     assert(false);
  //   }
  // });

  // it("End user can donate to specific project", async () => {
  //   try {
  //     const applications = await getApplications();
  //     const application = applications.find((a) => a.status === "Approved");
  //     if (!application) {
  //       console.log("No approved application to donate to");
  //       assert(false);
  //     }
  //     const message = "Go go go!";
  //     const donationAmount = parseNearAmount("1") as string; // 1 NEAR in YoctoNEAR
  //     const donorAccount = chefAccount;
  //     const potConfig = await getPotConfig();
  //     const amountPerBasisPoint = new BN(donationAmount).div(new BN(10_000));
  //     // calculate protocol fee
  //     let protocolFee = amountPerBasisPoint.mul(
  //       new BN(potConfig.protocol_fee_basis_points)
  //     );
  //     const donationsBalanceBefore = new BN(await getDonationsBalance()); // YoctoNEAR
  //     await donate({
  //       donorAccount,
  //       projectId: application.project_id,
  //       donationAmount,
  //       message,
  //     });
  //     const donationsBalanceAfter = new BN(await getDonationsBalance()); // YoctoNEAR
  //     // assert that fees were correctly taken out
  //     assert(
  //       donationsBalanceAfter.sub(donationsBalanceBefore).toString() ===
  //         new BN(donationAmount).sub(protocolFee).toString()
  //     );
  //     // assert that the donation record was created
  //     const donations = await getDonations();
  //     let exists = donations.some(
  //       (d) => d.message === message && d.donor_id === donorAccount.accountId
  //     );
  //     assert(exists);
  //   } catch (e) {
  //     console.log("error making patron donation: ", e);
  //     assert(false);
  //   }
  // });

  // it("End user can donate to all projects", async () => {
  //   try {
  //     // // make sure round is open
  //     // await adminSetRoundOpen(
  //     //   potDeployerAdminAccount,
  //     //   Date.now() + DEFAULT_ROUND_LENGTH
  //     // );
  //     const applications = await getApplications();
  //     const message = "Go go go!";
  //     const donationAmount = parseNearAmount("1") as string; // 1 NEAR in YoctoNEAR
  //     const donorAccount = potDeployerAdminAccount;
  //     const potConfig = await getPotConfig();
  //     const amountPerBasisPoint = new BN(donationAmount).div(new BN(10_000));
  //     // calculate protocol fee
  //     let protocolFee = amountPerBasisPoint.mul(
  //       new BN(potConfig.protocol_fee_basis_points)
  //     );
  //     const donationsBalanceBefore = new BN(await getDonationsBalance()); // YoctoNEAR
  //     await donate({
  //       donorAccount,
  //       projectId: null,
  //       donationAmount,
  //       message,
  //     });
  //     const donationsBalanceAfter = new BN(await getDonationsBalance()); // YoctoNEAR
  //     // assert that fees were correctly taken out
  //     assert(
  //       donationsBalanceAfter
  //         .sub(donationsBalanceBefore)
  //         .toString()
  //         .substring(0, 10) ===
  //         new BN(donationAmount).sub(protocolFee).toString().substring(0, 10)
  //     );
  //     // assert that the donation record was created
  //     // TODO: assert that multiple donation records were created
  //     const donations = await getDonations();
  //     let exists = donations.some(
  //       (d) => d.message === message && d.donor_id === donorAccount.accountId
  //     );
  //     assert(exists);
  //   } catch (e) {
  //     console.log("error making patron donation: ", e);
  //     assert(false);
  //   }
  // });

  // it("Chef can set payouts", async () => {
  //   try {
  //     // first, close the round
  //     await adminCloseRound(potDeployerAdminAccount);
  //     // get matching pot balance
  //     const matchingPoolBalance = await getMatchingPoolBalance();
  //     // get all donations
  //     let next = true;
  //     let fromIndex = 0;
  //     let limit = 100;
  //     const allDonations = [];
  //     while (next) {
  //       console.log(
  //         "Getting donations from " +
  //           fromIndex +
  //           " to " +
  //           (fromIndex + limit - 1) +
  //           "... "
  //       );
  //       const donationRecords = await getDonations(fromIndex, limit);
  //       allDonations.push(...donationRecords);
  //       if (donationRecords.length < limit) {
  //         next = false;
  //       } else {
  //         // get more
  //         fromIndex += limit;
  //       }
  //     }
  //     // format donations as [ProjectId, UserId, YoctoBN]
  //     const formattedDonations =
  //       convertDonationsToProjectContributions(allDonations);
  //     // run quadratic calculations to get payouts
  //     const payoutsInputs = calculateQuadraticPayouts(
  //       formattedDonations,
  //       new BN("25000000000000000000000000"), // TODO: figure out threshold
  //       new BN(matchingPoolBalance)
  //     );
  //     // set payouts
  //     await chefSetPayouts(chefAccount, payoutsInputs);
  //     // verify payouts
  //     const payouts = await getPayouts();
  //     // verify cooldown period started
  //     const config = await getPotConfig();
  //     assert(config.cooldown_end_ms);
  //   } catch (e) {
  //     console.log("error in chef set payouts test: ", e);
  //   }
  // });

  // it("Admin (DAO) can process payouts", async () => {
  //   try {
  //     // make sure there are payouts
  //     let payouts = await getPayouts();
  //     if (payouts.length === 0) {
  //       console.log("No payouts to process");
  //       assert(false);
  //     }
  //     // process payouts
  //     // should fail because cooldown period isn't over
  //     try {
  //       await adminProcessPayouts(potDeployerAdminAccount);
  //       assert(false);
  //     } catch (e) {
  //       assert(JSON.stringify(e).includes("Cooldown period is not over"));
  //       // end cooldown period manually
  //       await adminSetCooldownPeriodComplete(potDeployerAdminAccount);
  //       // try processing payouts again (should succeed)
  //       await adminProcessPayouts(potDeployerAdminAccount);
  //       // get payouts and verify paid_at
  //       payouts = await getPayouts();
  //       for (const payout of payouts) {
  //         assert(payout.paid_at);
  //       }
  //       // verify round config
  //       const config = await getPotConfig();
  //       assert(config.all_paid_out);
  //     }
  //   } catch (e) {
  //     console.log("error processing payouts: ", e);
  //   }
  // });
});
