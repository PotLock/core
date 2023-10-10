# PotLock Registry Contract

## Purpose

Projects that wish to apply for a Pot (funding round) must first be registered on the PotLock Registry (a singleton). Each Pot contract will verify the project against the Registry when a project applies for the Pot.

## Contract Structure

### General Types

```rs
type ProjectId = AccountId;
type TimestampMs = u64;
```

### Contract

```rs
pub struct Contract {
    owner: AccountId,
    admins: UnorderedSet<AccountId>,
    project_ids: UnorderedSet<ProjectId>,
    projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    project_team_members_by_project_id: LookupMap<ProjectId, UnorderedSet<AccountId>>,
}
```

### Projects

_NB: Projects are automatically approved by default._

```rs
pub enum ProjectStatus {
    Submitted,
    InReview,
    Approved,
    Rejected,
}

// ProjectInternal is the data structure that is stored within the contract
pub struct ProjectInternal {
    pub id: ProjectId,
    pub name: String,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

// Ephemeral data structure used for view methods, not stored within contract
pub struct ProjectExternal {
    pub id: ProjectId,
    pub name: String,
    pub team_members: Vec<AccountId>,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}
```

## Methods

### Write Methods

```rs
// PROJECTS

#[payable]
pub fn register(
    &mut self,
    name: String,
    team_members: Vec<AccountId>,
    _project_id: Option<AccountId>, // NB: _project_id can only be specified by admin; otherwise, it is the caller
) -> ProjectExternal 

#[payable]
pub fn admin_set_project_status(
    &mut self,
    project_id: ProjectId,
    status: ProjectStatus,
    review_notes: Option<String>,
) -> ()

// ADMINS

#[payable]
pub fn owner_add_admins(&mut self, admins: Vec<AccountId>)

#[payable]
pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>)
```

### Read Methods

```rs
// PROJECTS

pub fn get_projects(&self) -> Vec<ProjectExternal>

pub fn get_project_by_id(&self, project_id: ProjectId) -> ProjectExternal

// ADMINS

pub fn get_admins(&self) -> Vec<AccountId>
```