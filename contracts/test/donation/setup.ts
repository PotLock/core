import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { contractId, networkId, nodeUrl } from "./config";
import { loadCredentials } from "../utils/helpers";
import { POT_DEPLOYER_ALWAYS_ADMIN_ID } from "../pot_deployer/config";
import {
  DEFAULT_NEW_ACCOUNT_AMOUNT,
  DEFAULT_PARENT_ACCOUNT_ID,
} from "../utils/constants";
import { BN } from "bn.js";
import { parseNearAmount } from "near-api-js/lib/utils/format";

// set up contract account credentials
const contractCredentials = loadCredentials(networkId, contractId);
const keyStore = new keyStores.InMemoryKeyStore();
keyStore.setKey(
  networkId,
  contractId,
  KeyPair.fromString(contractCredentials.private_key)
);

// // TODO: revise this
// // set project account key
// const projectCredentials = loadCredentials(networkId, DEFAULT_PROJECT_ID);
// keyStore.setKey(
//   networkId,
//   DEFAULT_PROJECT_ID,
//   KeyPair.fromString(projectCredentials.private_key)
// );
// // set pot deployer admin account key
// const potDeployerAdminCredentials = loadCredentials(
//   networkId,
//   POT_DEPLOYER_ALWAYS_ADMIN_ID
// );
// keyStore.setKey(
//   networkId,
//   POT_DEPLOYER_ALWAYS_ADMIN_ID,
//   KeyPair.fromString(potDeployerAdminCredentials.private_key)
// );

// set up near connection
export const near = new Near({
  networkId,
  deps: { keyStore },
  nodeUrl,
});

export const contractAccount = new Account(near.connection, contractId);

// // set parent account key
// const parentAccountCredentials = loadCredentials(
//   networkId,
//   DEFAULT_PARENT_ACCOUNT_ID
// );
// keyStore.setKey(
//   networkId,
//   DEFAULT_PARENT_ACCOUNT_ID,
//   KeyPair.fromString(parentAccountCredentials.private_key)
// );
// const parentAccount = new Account(near.connection, DEFAULT_PARENT_ACCOUNT_ID);

// export const getPatronAccount = async () => {
//   const now = Date.now();
//   const patronAccountPrefix = `patron-${now}.`;
//   // create account which shares the credentials of the parent account
//   const patronAccountId = patronAccountPrefix + DEFAULT_PARENT_ACCOUNT_ID;
//   console.log("🧙‍♀️ Creating patron account", patronAccountId);
//   try {
//     await parentAccount.createAccount(
//       patronAccountId,
//       parentAccountCredentials.public_key,
//       new BN(DEFAULT_NEW_ACCOUNT_AMOUNT as string)
//     );
//     console.log("✅ Created patron account", patronAccountId);
//     keyStore.setKey(
//       networkId,
//       patronAccountId,
//       KeyPair.fromString(parentAccountCredentials.private_key)
//     );
//     const patronAccount = new Account(near.connection, patronAccountId);
//     return patronAccount;
//   } catch (e) {
//     // console.log("error creating project account", patronAccountId, e);
//     throw e;
//   }
// };

// export const getChefAccount = async () => {
//   // TODO: change this so that it creates a single subaccount "chef" under contract account. this way we can determine if they have already been created, and skip if so.
//   const now = Date.now();
//   const chefAccountPrefix = `chef-${now}.`;
//   // create account which shares the credentials of the parent account
//   const chefAccountId = chefAccountPrefix + DEFAULT_PARENT_ACCOUNT_ID;
//   console.log("👨‍🍳 Creating chef account", chefAccountId);
//   try {
//     await parentAccount.createAccount(
//       chefAccountId,
//       parentAccountCredentials.public_key,
//       new BN(DEFAULT_NEW_ACCOUNT_AMOUNT as string)
//     );
//     console.log("✅ Created chef account", chefAccountId);
//     keyStore.setKey(
//       networkId,
//       chefAccountId,
//       KeyPair.fromString(parentAccountCredentials.private_key)
//     );
//     const chefAccount = new Account(near.connection, chefAccountId);
//     return chefAccount;
//   } catch (e) {
//     throw e;
//   }
// };

// export const getProjectAccounts = async () => {
//   // TODO: change this so that it creates three "project" subaccounts under contract account. this way we can determine if they have already been created, and skip if so.
//   // create 3 x project accounts
//   const now = Date.now();
//   const projectAccountPrefixes = [
//     `project-${now}-1.`,
//     `project-${now}-2.`,
//     `project-${now}-3.`,
//   ];
//   const projectAccounts = [];
//   const parentAccount = new Account(near.connection, DEFAULT_PARENT_ACCOUNT_ID);
//   for (const projectAccountPrefix of projectAccountPrefixes) {
//     // create account which shares the credentials of the parent account
//     const projectAccountId = projectAccountPrefix + DEFAULT_PARENT_ACCOUNT_ID;
//     console.log("🧙‍♀️ Creating project account", projectAccountId);
//     try {
//       await parentAccount.createAccount(
//         projectAccountId,
//         parentAccountCredentials.public_key,
//         new BN(DEFAULT_NEW_ACCOUNT_AMOUNT as string)
//       );
//       console.log("✅ Created project account", projectAccountId);
//       keyStore.setKey(
//         networkId,
//         projectAccountId,
//         KeyPair.fromString(parentAccountCredentials.private_key)
//       );
//       const projectAccount = new Account(near.connection, projectAccountId);
//       projectAccounts.push(projectAccount);
//     } catch (e) {
//       throw e;
//     }
//   }
//   return projectAccounts;
// };
