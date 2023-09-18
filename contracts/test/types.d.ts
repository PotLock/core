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
