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
    /// Contract superuser
    owner: AccountId,
    /// Contract admins (can be added/removed by owner)
    admins: UnorderedSet<AccountId>,
    /// Old set (deprecated but empty set must be retained in state or serialization will break)
    _deprecated_project_ids: UnorderedSet<ProjectId>,
    /// Old map (deprecated but empty map must be retained in state or serialization will break)
    _deprecated_projects_by_id: LookupMap<ProjectId, VersionedProjectInternal>,
    /// Records of all Projects deployed by this Registry, indexed at their account ID, versioned for easy upgradeability
    projects_by_id: UnorderedMap<ProjectId, VersionedProjectInternal>,
    /// Projects pending approval
    pending_project_ids: UnorderedSet<ProjectId>,
    /// Projects approved
    approved_project_ids: UnorderedSet<ProjectId>,
    /// Projects rejected
    rejected_project_ids: UnorderedSet<ProjectId>,
    /// Contract "source" metadata, as specified in NEP 0330 (https://github.com/near/NEPs/blob/master/neps/nep-0330.md), with addition of `commit_hash`
    contract_source_metadata: LazyOption<VersionedContractSourceMetadata>,
    /// Default status when project registers
    default_project_status: ProjectStatus,
}

pub struct ContractConfig {
    pub owner: AccountId,
    pub admins: Vec<AccountId>,
    pub default_project_status: ProjectStatus,
    pub pending_project_count: u64,
    pub approved_project_count: u64,
    pub rejected_project_count: u64,
}
```

### Projects

_NB: Projects are automatically approved by default._

```rs
pub enum ProjectStatus {
    Pending,
    Approved,
    Rejected,
}

// ProjectInternal is the data structure that is stored within the contract
pub struct ProjectInternal {
    pub id: ProjectId,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}

// Ephemeral data structure used for view methods, not stored within contract
pub struct ProjectExternal {
    pub id: ProjectId,
    pub status: ProjectStatus,
    pub submitted_ms: TimestampMs,
    pub updated_ms: TimestampMs,
    pub review_notes: Option<String>,
}
```

## Methods

### Write Methods

**NB: ALL privileged write methods (those beginning with `admin_*` or `owner_*`) require an attached deposit of at least one yoctoNEAR, for security purposes.**

```rs
// INIT

pub fn new(
    owner: AccountId,
    admins: Vec<AccountId>,
    source_metadata: ContractSourceMetadata,
) -> Self


// OWNER

#[payable]
pub fn owner_change_owner(&mut self, owner: AccountId)


// ADMINS

#[payable]
pub fn owner_add_admins(&mut self, admins: Vec<AccountId>)

#[payable]
pub fn owner_remove_admins(&mut self, admins: Vec<AccountId>)

#[payable]
pub fn admin_set_default_project_status(&mut self, status: ProjectStatus)

#[payable]
pub fn admin_set_project_status(
    &mut self,
    project_id: ProjectId,
    status: ProjectStatus,
    review_notes: Option<String>,
) -> ()


// PROJECTS

#[payable]
pub fn register(
    &mut self,
    _project_id: Option<AccountId>, // NB: _project_id can only be specified by admin; otherwise, it is the caller
) -> ProjectExternal 


// SOURCE METADATA

pub fn self_set_source_metadata(&mut self, source_metadata: ContractSourceMetadata) // only callable by the contract account (reasoning is that this should be able to be updated by the same account that can deploy code to the account)
```

### Read Methods

```rs
// CONTRACT

pub fn get_config(&self) -> ContractConfig


// PROJECTS

pub fn is_registered(&self, account_id: ProjectId) -> bool

pub fn get_projects(
    &self,
    status: Option<ProjectStatus>,
    from_index: Option<u64>,
    limit: Option<u64>,
) -> Vec<ProjectExternal>

pub fn get_project_by_id(&self, project_id: ProjectId) -> ProjectExternal


// SOURCE METADATA

pub fn get_contract_source_metadata(&self) -> Option<ContractSourceMetadata>
```