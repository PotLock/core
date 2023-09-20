import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { contractId, parentAccountId, networkId, nodeUrl } from "./config";
import { loadCredentials } from "../utils/helpers";

// set up contract account credentials
const contractCredentials = loadCredentials(networkId, contractId);
const keyStore = new keyStores.InMemoryKeyStore();
keyStore.setKey(
  networkId,
  contractId,
  KeyPair.fromString(contractCredentials.private_key)
);
// set parent account key (same as contract account key)
keyStore.setKey(
  networkId,
  parentAccountId,
  KeyPair.fromString(contractCredentials.private_key)
);

// set up near connection
export const near = new Near({
  networkId,
  deps: { keyStore },
  nodeUrl,
});

export const contractAccount = new Account(near.connection, contractId);
