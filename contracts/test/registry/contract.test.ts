import assert from "assert";
import fs from "fs";
import { KeyPair, Near, Account, Contract, keyStores } from "near-api-js";
import { contractId } from "./config";
import { DEFAULT_GAS, NO_CONTRACT_HASH, functionCallBase } from "../utils/near";

// The constants below will vary based on your setup
const NETWORK_URL = "http://localhost:3030";
const MASTER_ACCOUNT_SEED = "test.near";
const CONTRACT_ACCOUNT_ID = "test_contract.test.near"; // Replace with your contract account ID

describe("Contract Tests", () => {
  let near: Near;
  let contractAccount: Account;
  let ownerAccount: Account;
  let ownerId: AccountId = contractId;
  let adminId: AccountId = contractId;
  let adminAccount: Account;
  let projectAccount: Account;
  let projectId: AccountId = contractId;
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
    ownerAccount = new Account(near.connection, ownerId);
    adminAccount = new Account(near.connection, adminId);
    projectAccount = new Account(near.connection, projectId);

    // initialize contract if it's not already
    // attempt to initialize; if it failes, it's already initialized
    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "new",
        args: {
          owner: ownerId,
          admins: [ownerId],
        },
        gas: DEFAULT_GAS,
      });
      console.log(`✅ Initialized Registry contract ${contractId}`);
    } catch (e) {
      if (
        JSON.stringify(e).includes("The contract has already been initialized")
      ) {
        console.log(`Registry contract ${contractId} is already initialized`);
      } else {
        console.log("Registry initialize error: ", e);
        assert(false);
      }
    }
    // make contract address the contract owner
  });

  // it("Owner can add & remove admins", async () => {
  //   // Add admin
  //   try {
  //     const admin = "admin1.testnet";
  //     await ownerAccount.functionCall({
  //       contractId,
  //       methodName: "owner_add_admins",
  //       args: {
  //         admins: [admin],
  //       },
  //       ...functionCallBase,
  //     });

  //     // Verify admin was added
  //     const admins = await contractAccount.viewFunction({
  //       contractId,
  //       methodName: "get_admins",
  //     });
  //     assert(admins.includes(admin));

  //     // Remove admin
  //     await contractAccount.functionCall({
  //       contractId,
  //       methodName: "owner_remove_admins",
  //       args: {
  //         admins: [admin],
  //       },
  //       ...functionCallBase,
  //     });

  //     // Verify admin was removed
  //     const updatedAdmins = await contractAccount.viewFunction({
  //       contractId,
  //       methodName: "get_admins",
  //     });
  //     assert(!updatedAdmins.includes(admin));
  //   } catch (e) {
  //     console.log("Error adding or removing admins:", e);
  //     assert(false);
  //   }
  // });

  it("End user or Admins can register a Project", async () => {
    // Project ID should not already be registered
    // Project should be approved by default❓
    try {
      // End user registers a project
      const projectName = `${projectId}#${Date.now()}`;
      const teamMemberA: AccountId = "a.testnet";
      const teamMemberB: AccountId = "b.testnet";
      const teamMembers = [teamMemberA, teamMemberB];
      await projectAccount.functionCall({
        contractId,
        methodName: "register",
        args: {
          name: projectName,
          team_members: teamMembers,
        },
        ...functionCallBase,
      });

      // Verify project was registered by end user & approved by default
      let projects: Project[] = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });
      let existing = projects.find(
        (p) =>
          p.name === projectName &&
          p.id === projectId &&
          p.status === "Approved"
      );
      assert(!!existing);
      assert.deepEqual(existing.team_members, teamMembers);

      // Cannot reregister
      try {
        await projectAccount.functionCall({
          contractId,
          methodName: "register",
          args: {
            name: projectName,
            team_members: teamMembers,
          },
          ...functionCallBase,
        });
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes("Project already exists"));
      }

      // Admin registers a project, specifying _project_id
      const projectId2: AccountId = "project2.testnet";
      await adminAccount.functionCall({
        contractId,
        methodName: "register",
        args: {
          name: projectName,
          team_members: teamMembers,
          _project_id: projectId2,
        },
        ...functionCallBase,
      });

      // Verify project was registered by admin & approved by default
      projects = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });
      existing = projects.find(
        (p) =>
          p.name === projectName &&
          p.id === projectId2 &&
          p.status === "Approved"
      );
      assert(!!existing);
      assert.deepEqual(existing.team_members, teamMembers);
    } catch (e) {
      console.log("Error registering project:", e);
      assert(false);
    }
  });

  it("Admins can change status of Project", async () => {
    // Project ID should not already be registered
    // Project should be approved by default❓
    try {
      // Get projects
      let projects: Project[] = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });

      if (projects.length === 0) {
        // If no projects, create new project and refetch
        const projectName = `${projectId}#${Date.now()}`;
        const teamMemberA: AccountId = "a.testnet";
        const teamMemberB: AccountId = "b.testnet";
        const teamMembers = [teamMemberA, teamMemberB];
        await projectAccount.functionCall({
          contractId,
          methodName: "register",
          args: {
            name: projectName,
            team_members: teamMembers,
          },
          ...functionCallBase,
        });
        projects = await contractAccount.viewFunction({
          contractId,
          methodName: "get_projects",
        });
      }

      const project = projects[0];
      const newStatus = "Rejected";
      const reviewNotes =
        "This project is rejected because it gives Few and Far vibes";

      await projectAccount.functionCall({
        contractId,
        methodName: "admin_set_project_status",
        args: {
          project_id: project.id,
          status: newStatus,
          review_notes: reviewNotes,
        },
        ...functionCallBase,
      });

      let updatedProject: Project = await contractAccount.viewFunction({
        contractId,
        methodName: "get_project_by_id",
        args: {
          project_id: project.id,
        },
      });

      assert(
        updatedProject.status === newStatus &&
          updatedProject.review_notes == reviewNotes
      );
    } catch (e) {
      console.log("Error setting project status:", e);
      assert(false);
    }
  });

  // Add tests for other scenarios similarly...

  // Note: The test structure provided here is a basic guideline. You might need to adjust based on the real contract functions and the actual setup.
});
