import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { contractId, networkId, nodeUrl } from "./config";
import { loadCredentials } from "../utils/helpers";

// set up contract account credentials
const contractCredentials = loadCredentials(networkId, contractId);
const contractKeyStore = new keyStores.InMemoryKeyStore();
contractKeyStore.setKey(
  networkId,
  contractId,
  KeyPair.fromString(contractCredentials.private_key)
);

// set up near connection
export const near = new Near({
  networkId,
  deps: { keyStore: contractKeyStore },
  nodeUrl,
});

export const contractAccount = new Account(near.connection, contractId);
