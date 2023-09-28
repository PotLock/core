type TimestampMs = number;
type AccountId = string;
type ProjectId = AccountId;
type ApplicationId = number; // increments from 1

enum ProjectStatus {
  Submitted = "Submitted",
  InReview = "InReview",
  Approved = "Approved",
  Rejected = "Rejected",
}

enum ApplicationStatus {
  Pending = "Pending",
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
  protocol_fee_recipient_account: AccountId;
}

interface PotConfig extends PotArgs {
  created_by: AccountId;
  matching_pool_balance: string;
  donations_balance: string;
  cooldown_end_ms: TimestampMs | null;
  paid_out: boolean;
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

interface Application {
  id: ApplicationId;
  project_id: ProjectId;
  status: ApplicationStatus;
  submitted_at: TimestampMs;
  updated_at: TimestampMs | null;
  review_notes: string | null;
}

/// Patron donation; no application specified
interface PatronDonation {
  id: number;
  donor_id: AccountId;
  total_amount: string;
  message: string | null;
  donated_at: TimestampMs;
  referrer_id: AccountId | null;
  referrer_fee: string | null;
  protocol_fee: string;
  amount_after_fees: string;
}

/// End-user donation; must specify application
interface Donation {
  application_id: ApplicationId;
  id: number;
  donor_id: AccountId;
  total_amount: string;
  message: string | null;
  donated_at: TimestampMs;
  protocol_fee: string;
  amount_after_fees: string;
}
