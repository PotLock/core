type TimestampMs = number;
type AccountId = string;
type ProjectId = AccountId;
type ApplicationId = number; // increments from 1
type PayoutId = number; // increments from 1
type DonationId = number; // increments from 1

enum ProjectStatus {
  Submitted = "Submitted",
  InReview = "InReview",
  Approved = "Approved",
  Rejected = "Rejected",
  Graylisted = "Graylisted",
  Blacklisted = "Blacklisted",
}

enum ApplicationStatus {
  Pending = "Pending",
  InReview = "InReview",
  Approved = "Approved",
  Rejected = "Rejected",
}

interface Project {
  id: AccountId;
  status: ProjectStatus;
  submitted_ms: TimestampMs;
  updated_ms: TimestampMs;
  review_notes: string | null;
}

interface PotArgs {
  chef: AccountId;
  pot_name: String;
  pot_description: String;
  public_round_start_ms: TimestampMs;
  public_round_end_ms: TimestampMs;
  application_start_ms: TimestampMs;
  application_end_ms: TimestampMs;
  max_projects: number;
  base_currency: AccountId;
  donation_requirement: SBTRequirement | null;
  referral_fee_basis_points: number;
  chef_fee_basis_points: number;
  protocol_fee_basis_points: number;
  protocol_fee_recipient_account: AccountId;
}

interface PotConfig extends PotArgs {
  deployed_by: AccountId;
  matching_pool_balance: string;
  donations_balance: string;
  cooldown_end_ms: TimestampMs | null;
  all_paid_out: boolean;
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
  id: number;
  donor_id: AccountId;
  total_amount: string;
  message: string | null;
  donated_at: TimestampMs;
  project_id: ProjectId;
  protocol_fee: string;
  amount_after_fees: string;
}

/// Project payout
interface Payout {
  id: PayoutId;
  project_id: ProjectId;
  matching_pool_amount: string;
  donations_amount: string;
  amount_total: string;
  paid_at: TimestampMs | null;
}

interface PayoutInput {
  project_id: ProjectId;
  matching_pool_amount: string;
  donations_amount: string;
}
