import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { DEFAULT_PROJECT_ID, contractId, networkId, nodeUrl } from "./config";
import { loadCredentials } from "../utils/helpers";
import { POT_DEPLOYER_ALWAYS_ADMIN_ID } from "../pot_deployer/config";

// set up contract account credentials
const contractCredentials = loadCredentials(networkId, contractId);
const keyStore = new keyStores.InMemoryKeyStore();
keyStore.setKey(
  networkId,
  contractId,
  KeyPair.fromString(contractCredentials.private_key)
);
// set project account key
const projectCredentials = loadCredentials(networkId, DEFAULT_PROJECT_ID);
keyStore.setKey(
  networkId,
  DEFAULT_PROJECT_ID,
  KeyPair.fromString(projectCredentials.private_key)
);
// set pot deployer admin account key
const potDeployerAdminCredentials = loadCredentials(
  networkId,
  POT_DEPLOYER_ALWAYS_ADMIN_ID
);
keyStore.setKey(
  networkId,
  POT_DEPLOYER_ALWAYS_ADMIN_ID,
  KeyPair.fromString(potDeployerAdminCredentials.private_key)
);

// set up near connection
export const near = new Near({
  networkId,
  deps: { keyStore },
  nodeUrl,
});

export const contractAccount = new Account(near.connection, contractId);
