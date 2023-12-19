const CONTRACT_ID = "1702409170.test-contracts.potlock.testnet";
const L_NFT_TEST = "lachlan-nft-test.testnet";
const L_NFT_TEST_2 = "lachlan-nft-test-2.testnet";
const L_NFT_TEST_8 = "lachlan-nft-test-8.testnet";

// PotDeployer init args

// console.log(
//   JSON.stringify({
//     owner: CONTRACT_ID,
//     admins: [L_NFT_TEST],
//     protocol_fee_basis_points: 200,
//     protocol_fee_recipient_account: L_NFT_TEST_2,
//     default_chef_fee_basis_points: 500,
//     whitelisted_deployers: [L_NFT_TEST_8],
//     require_whitelist: true,
//   })
// );

// PotArgs

// pub struct PotArgs {
//     owner: Option<AccountId>,
//     admins: Option<Vec<AccountId>>,
//     chef: Option<AccountId>,
//     pot_name: String,
//     pot_description: String,
//     max_projects: u32,
//     application_start_ms: TimestampMs,
//     application_end_ms: TimestampMs,
//     public_round_start_ms: TimestampMs,
//     public_round_end_ms: TimestampMs,
//     registry_provider: Option<ProviderId>,
//     sybil_wrapper_provider: Option<ProviderId>,
//     custom_sybil_checks: Option<Vec<CustomSybilCheck>>,
//     custom_min_threshold_score: Option<u32>,
//     referral_fee_matching_pool_basis_points: u32,
//     referral_fee_public_round_basis_points: u32,
//     chef_fee_basis_points: u32,
// }

console.log(
  JSON.stringify({
    pot_name: "test",
    pot_description: "test",
    max_projects: 3,
    application_start_ms: Date.now(),
    application_end_ms: Date.now() + 1000 * 60 * 60 * 24 * 7,
    public_round_start_ms: Date.now() + 1000 * 60 * 60 * 24 * 7,
    public_round_end_ms: Date.now() + 1000 * 60 * 60 * 24 * 7 * 2,
    referral_fee_matching_pool_basis_points: 500,
    referral_fee_public_round_basis_points: 200,
    chef_fee_basis_points: 500,
  })
);
