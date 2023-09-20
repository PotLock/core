type TimestampMs = number;
type AccountId = string;

enum ProjectStatus {
  Submitted = "Submitted",
  InReview = "InReview",
  Approved = "Approved",
  Rejected = "Rejected",
}

interface Project {
  id: AccountId;
  name: string;
  team_members: AccountId[];
  status: ProjectStatus;
  submitted_ms: TimestampMs;
  updated_ms: TimestampMs;
  review_notes: string | null;
}

interface PotArgs {
  chef_id: AccountId;
  round_name: String;
  round_description: String;
  round_start_ms: TimestampMs;
  round_end_ms: TimestampMs;
  application_start_ms: TimestampMs;
  application_end_ms: TimestampMs;
  max_projects: number;
  base_currency: AccountId;
  donation_requirement: SBTRequirement | null;
  patron_referral_fee_basis_points: number;
  max_patron_referral_fee: string;
  round_manager_fee_basis_points: number;
  protocol_fee_basis_points: number;
}

interface Pot {
  on_chain_name: string;
  deployed_by: AccountId;
}

interface SBTRequirement {
  registry_id: AccountId;
  issuer_id: AccountId;
  class_id: number;
}

interface PotDeployerConfig {
  protocol_fee_basis_points: number;
  max_protocol_fee_basis_points: number;
  default_chef_fee_basis_points: number;
  max_chef_fee_basis_points: number;
  max_round_time: number;
  max_application_time: number;
}
