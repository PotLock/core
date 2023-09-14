import assert from "assert";
import fs from "fs";
import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { contractId } from "./config";
import { functionCallBase } from "../utils/near";

// The constants below will vary based on your setup
const NETWORK_URL = "http://localhost:3030";
const MASTER_ACCOUNT_SEED = "test.near";
const CONTRACT_ACCOUNT_ID = "test_contract.test.near"; // Replace with your contract account ID

describe("Contract Tests", () => {
  let near: Near;
  let contractAccount: Account;
  let networkId = "testnet";
  let nodeUrl = "https://rpc.testnet.near.org";

  let credentials;
  console.log("contractId: ", contractId);
  try {
    credentials = JSON.parse(
      fs.readFileSync(
        `${process.env.HOME}/.near-credentials/${networkId}/${contractId}.json`,
        "utf-8"
      )
    );
  } catch (e) {
    console.warn("credentials not found, looking in /neardev");
    credentials = JSON.parse(
      fs.readFileSync(
        `./registry/neardev/${networkId}/${contractId}.json`,
        "utf-8"
      )
    );
  }

  const keyStore = new keyStores.InMemoryKeyStore();
  keyStore.setKey(
    networkId,
    contractId,
    KeyPair.fromString(credentials.private_key)
  );

  before(async () => {
    near = new Near({
      networkId,
      deps: { keyStore },
      nodeUrl,
    });

    contractAccount = new Account(near.connection, contractId);
  });

  it("Owner can add & remove admins", async () => {
    // Add admin
    try {
      const admin = "admin1.testnet";
      // TODO: PICK UP HERE, change to ownerAccount.functionCall()...
      await contractAccount
        .functionCall({
          contractId,
          methodName: "contract_set_metadata",
          args: {
            metadata: newMetadata,
          },
          ...functionCallBase,
        })
        .owner_add_admins([admin]);

      // Check if admin was added
      const admins = await contract.get_admins(); // Replace with the actual method name
      console.log("admins line 60: ", admins);

      assert(admins.includes(admin));

      // Remove admin
      await contract.owner_remove_admins([admin]);

      // Check if admin was removed
      const updatedAdmins = await contract.get_admins(); // Replace with the actual method name
      console.log("updated admins: ", updatedAdmins);
      assert(!updatedAdmins.includes(admin));
    } catch (e) {
      console.log("admins error: ", e);
      assert(false);
    }
  });

  // Add tests for other scenarios similarly...

  // Note: The test structure provided here is a basic guideline. You might need to adjust based on the real contract functions and the actual setup.
});
