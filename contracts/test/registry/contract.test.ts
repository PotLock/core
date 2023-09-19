import assert from "assert";
import fs from "fs";
import { Account } from "near-api-js";
import { contractId } from "./config";
import { near } from "./setup";
import {
  adminSetProjectStatus,
  getAdmins,
  getProjectById,
  getProjects,
  initializeContract,
  ownerAddAdmins,
  ownerRemoveAdmins,
  registerProject,
} from "./utils";

describe("Contract Tests", () => {
  let ownerAccount: Account;
  let ownerId: AccountId = contractId;
  let adminId: AccountId = contractId;
  let adminAccount: Account;
  let projectAccount: Account;
  let projectId: AccountId = contractId;

  before(async () => {
    ownerAccount = new Account(near.connection, ownerId);
    adminAccount = new Account(near.connection, adminId);
    projectAccount = new Account(near.connection, projectId);

    // initialize contract if it's not already
    // attempt to initialize; if it fails, it's already initialized
    try {
      await initializeContract({
        owner: ownerId,
        admins: [ownerId],
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
  });

  it("Owner can add & remove admins", async () => {
    try {
      // Add admin
      const admin = "admin1.testnet";
      await ownerAddAdmins(ownerAccount, [admin]);
      // Verify admin was added
      const admins = await getAdmins();
      assert(admins.includes(admin));
      // Remove admin
      await ownerRemoveAdmins(ownerAccount, [admin]);
      // Verify admin was removed
      const updatedAdmins = await getAdmins();
      assert(!updatedAdmins.includes(admin));
    } catch (e) {
      console.log("Error adding or removing admins:", e);
      assert(false);
    }
  });

  it("End user or Admins can register a Project", async () => {
    // Project ID should not already be registered
    // Project should be approved by default❓
    try {
      // End user registers a project
      const projectName = `${projectId}#${Date.now()}`;
      const teamMemberA: AccountId = "a.testnet";
      const teamMemberB: AccountId = "b.testnet";
      const teamMembers = [teamMemberA, teamMemberB];
      await registerProject(projectAccount, projectName, teamMembers);

      // Verify project was registered by end user & approved by default
      let projects = await getProjects();
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
        await registerProject(projectAccount, projectName, teamMembers);
        assert(false);
      } catch (e) {
        assert(JSON.stringify(e).includes("Project already exists"));
      }

      // Admin registers a project, specifying _project_id
      const projectId2: AccountId = "project2.testnet";
      await registerProject(adminAccount, projectName, teamMembers, projectId2);

      // Verify project was registered by admin & approved by default
      projects = await getProjects();
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
      let projects = await getProjects();

      if (projects.length === 0) {
        // If no projects, create new project and refetch
        const projectName = `${projectId}#${Date.now()}`;
        const teamMemberA: AccountId = "a.testnet";
        const teamMemberB: AccountId = "b.testnet";
        const teamMembers = [teamMemberA, teamMemberB];
        await registerProject(projectAccount, projectName, teamMembers);
        projects = await getProjects();
      }
      // Update status of first project
      const project = projects[0];
      const newStatus = "Rejected";
      const reviewNotes =
        "This project is rejected because it gives Few and Far vibes";
      await adminSetProjectStatus(
        projectAccount,
        project.id,
        newStatus,
        reviewNotes
      );
      let updatedProject = await getProjectById(project.id);
      assert(
        updatedProject.status === newStatus &&
          updatedProject.review_notes == reviewNotes
      );
    } catch (e) {
      console.log("Error setting project status:", e);
      assert(false);
    }
  });
});
