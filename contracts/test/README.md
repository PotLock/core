TEST CASES

Donation
- User can donate
- Owner can change the owner
- Owner can set protocol_fee_basis_points
- Owner can set referral_fee_basis_points
- Owner can set protocol_fee_recipient_account

Registry
- Can be deployed and initialized
- Owner can add & remove admins
  - admins must be DAO❓
- End user or Admins can register a Project
  - Project ID should not already be registered
  - Project should be approved by default❓
- Admins can change status of a Project

PotDeployer
- Can be deployed and initialized
- Admin (DAO) or whitelisted_deployer can deploy a new Pot
  - Specified chef must have "chef" role in ReFi DAO
- Admin (DAO) can:
  - Update protocol fee basis points (must be <= max_protocol_fee_basis_points)
  - Update default chef fee basis points (must be <= default_chef_fee_basis_points)
  - Update max protocol fee basis points
  - Update max chef fee basis points
  - Update max round time
  - Update max application time
  - Update max milestones
  - Add whitelisted deployers

Pot
- Can be deployed and initialized
- Enforces round_start_ms & round_end_ms
- Enforces application_start_ms & application_end_ms
- Enforces max_projects
- Enforces supported base_currency
- Enforces application_requirement (SBT)
- Enforces donation_requirement (SBT)
- Project can apply for round
  - Enforces haven't already applied
  - Enforces max_projects not met
  - Enforces application period open
  - Enforces registered project
  - Enforces caller is project admin
  - Enforces caller (or project ID❓) meets application requirement (SBT)
  - Emits event
- Project can unapply
  - Enforces caller is project admin
  - Enforces application is in Pending status
  - Emits event
- Chef can update application status
  - Enforces only chef
  - Must provide notes (reason)
- Patron can donate to matching pool
  - Protocol & chef fees paid out
  - Referrer paid out
  - Enforces round open
  - Emits event
- End user can donate to specific project
  - Enforces round open
  - Emits event
- End user can donate to all projects
  - Enforces round open
  - Emits events
- PotDeployer Admin (DAO) can change chef & chef fee
- Chef can set (update) the application requirement
- Chef can set (update) the donation requirement
- Chef can update the patron referral fee❓
- Chef can set payouts (CLR / quadratic calculations)
- PotDeployer Admin (DAO) can process payouts
  - Can cooldown period be overridden?



