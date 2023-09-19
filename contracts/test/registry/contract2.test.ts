import assert from "assert";
import fs from "fs";
import { KeyPair, Near, Account, keyStores } from "near-api-js";
import { contractId } from "./config";
import { DEFAULT_GAS, functionCallBase } from "../utils/constants";

const NETWORK_URL = "http://localhost:3030";
const MASTER_ACCOUNT_SEED = "test.near";
const CONTRACT_ACCOUNT_ID = "test_contract.test.near";
const networkId = "testnet";
const nodeUrl = "https://rpc.testnet.near.org";

function loadCredentials() {
  let path = `${process.env.HOME}/.near-credentials/${networkId}/${contractId}.json`;
  if (!fs.existsSync(path)) {
    path = `./registry/neardev/${networkId}/${contractId}.json`;
    if (!fs.existsSync(path)) {
      console.warn("Credentials not found");
      return null;
    }
  }
  return JSON.parse(fs.readFileSync(path, "utf-8"));
}

function handleTestError(e: Error) {
  console.error(e);
  assert(false);
}

async function registerProject(
  account: Account,
  name: string,
  teamMembers: string[],
  projectId?: string
) {
  return account.functionCall({
    contractId,
    methodName: "register",
    args: {
      name,
      team_members: teamMembers,
      ...(projectId ? { _project_id: projectId } : {}),
    },
    ...functionCallBase,
  });
}

describe("Contract Tests", () => {
  let near;
  let contractAccount;
  let ownerAccount;
  let adminAccount;
  let projectAccount;

  const credentials = loadCredentials();
  if (!credentials) {
    throw new Error("No credentials found");
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
    ownerAccount = new Account(near.connection, contractId);
    adminAccount = new Account(near.connection, contractId);
    projectAccount = new Account(near.connection, contractId);

    try {
      await contractAccount.functionCall({
        contractId,
        methodName: "new",
        args: { owner: contractId, admins: [contractId] },
        gas: DEFAULT_GAS,
      });
      console.log(`✅ Initialized Registry contract ${contractId}`);
    } catch (e) {
      if (e.message.includes("The contract has already been initialized")) {
        console.log(`Registry contract ${contractId} is already initialized`);
      } else {
        console.error("Registry initialize error: ", e);
        assert(false);
      }
    }
  });

  it("End user or Admins can register a Project", async () => {
    try {
      const projectName = `${contractId}#${Date.now()}`;
      const teamMembers = ["a.testnet", "b.testnet"];

      await registerProject(projectAccount, projectName, teamMembers);

      let projects = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });
      const project = projects.find(
        (p) => p.name === projectName && p.status === "Approved"
      );

      assert(!!project);
      assert.deepEqual(project.team_members, teamMembers);

      try {
        await registerProject(projectAccount, projectName, teamMembers);
        assert(false, "Should not be able to re-register");
      } catch (e) {
        assert(e.message.includes("Project already exists"));
      }

      const projectId2 = "project2.testnet";
      await registerProject(adminAccount, projectName, teamMembers, projectId2);

      projects = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });
      const adminProject = projects.find(
        (p) => p.id === projectId2 && p.status === "Approved"
      );

      assert(!!adminProject);
      assert.deepEqual(adminProject.team_members, teamMembers);
    } catch (e) {
      handleTestError(e);
    }
  });

  it("Admins can change status of Project", async () => {
    try {
      let projects = await contractAccount.viewFunction({
        contractId,
        methodName: "get_projects",
      });

      if (projects.length === 0) {
        const projectName = `${contractId}#${Date.now()}`;
        const teamMembers = ["a.testnet", "b.testnet"];
        await registerProject(projectAccount, projectName, teamMembers);
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

      const updatedProject = await contractAccount.viewFunction({
        contractId,
        methodName: "get_project_by_id",
        args: { project_id: project.id },
      });

      assert(
        updatedProject.status === newStatus &&
          updatedProject.review_notes == reviewNotes
      );
    } catch (e) {
      handleTestError(e);
    }
  });
});
