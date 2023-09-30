// taken from https://github.com/gitcoinco/quadratic-funding/blob/master/quadratic-funding/clr.py
// import BN from "bn.js";
// import bnsqrt from "bn-sqrt";
const BN = require("bn.js");
const bnsqrt = require("bn-sqrt");
// console.log("sqrt: ", bnsqrt.sqrt(new BN(2).mul(new BN(8))).toString());

type YoctoBN = typeof BN;
type UserId = AccountId;

// type GrantContribution = {
//   projectId: ProjectId;
//   contributions: Array<{ [key: AccountId]: YoctoBN }>;
// };
type GrantContribution = [ProjectId, UserId, YoctoBN];

let CLR_PERCENTAGE_DISTRIBUTED = 0;
const GRANT_CONTRIBUTIONS_EXAMPLE: GrantContribution[] = [
  ["4", "1", new BN("100000000000")],
  ["4", "2", new BN("50000000000")],
  ["4", "2", new BN("100000000000")],
  ["4", "3", new BN("70000000000")],
  ["4", "5", new BN("50000000000")],
  ["4", "4", new BN("100000000000")],
  ["4", "5", new BN("50000000000")],
  ["4", "5", new BN("50000000000")],
  ["5", "1", new BN("100000000000")],
  ["5", "1", new BN("50000000000")],
  ["5", "2", new BN("200000000000")],
  ["5", "3", new BN("30000000000")],
  ["5", "8", new BN("20000000000")],
  ["5", "9", new BN("100000000000")],
  ["5", "7", new BN("70000000000")],
  ["5", "2", new BN("50000000000")],
];

// // This function takes the grant data as input and produces a list of
// // contributions with the format [projectId, userId, contributionAmount].
// // Essentially, it's flattening the data structure to make it more manageable.
// function translateData(
//   grantsData: GrantContribution[]
// ): [ProjectId, UserId, BN][] {
//   let grantsList: [ProjectId, UserId, YoctoBN][] = [];
//   for (const g of grantsData) {
//     const grantId = g.projectId;
//     for (const c of g.contributions) {
//       const val: [ProjectId, UserId, YoctoBN] = [
//         grantId,
//         Object.keys(c)[0],
//         Object.values(c)[0],
//       ];
//       grantsList.push(val);
//     }
//   }
//   return grantsList;
// }

// This function takes the flattened list of contributions and aggregates
// the amounts contributed by each user to each project.
// It returns a dictionary where each key is a projectId and its value
// is another dictionary of userIds and their aggregated contribution amounts.
type ContribDict = { [key: ProjectId]: { [key: UserId]: YoctoBN } };
function aggregateContributions(
  grantContributions: GrantContribution[]
): ContribDict {
  const contribDict: { [key: ProjectId]: { [key: UserId]: YoctoBN } } = {};
  for (const [proj, user, amount] of grantContributions) {
    if (!contribDict[proj]) {
      contribDict[proj] = {};
    }
    contribDict[proj][user] = new BN(contribDict[proj][user] || 0).add(amount);
  }
  return contribDict;
}

// This function calculates the total overlapping contribution amounts between pairs of users for each project.
// It returns a nested dictionary where the outer keys are userIds and the inner keys are also userIds,
// and the inner values are the total overlap between these two users' contributions.
type PairTotals = { [key: UserId]: { [key: UserId]: YoctoBN } };
function getTotalsByPair(contribDict: ContribDict): PairTotals {
  console.log("contribDict: ", contribDict);
  const totOverlap: { [key: UserId]: { [key: UserId]: YoctoBN } } = {};
  for (const contribz of Object.values(contribDict)) {
    for (const [k1, v1] of Object.entries(contribz)) {
      if (!totOverlap[k1]) {
        totOverlap[k1] = {};
      }
      for (const [k2, v2] of Object.entries(contribz)) {
        if (!totOverlap[k1][k2]) {
          totOverlap[k1][k2] = new BN(0);
        }
        totOverlap[k1][k2] = totOverlap[k1][k2].add(bnsqrt.sqrt(v1.mul(v2)));
      }
    }
  }
  return totOverlap;
}

// This function computes the CLR (Contribution Matching) amount for each project.
// It takes the aggregated contributions, the total overlaps between user pairs,
// a threshold value, and the total pot available for matching.
// It then calculates the matching amount for each project using the quadratic formula
// and returns a list of objects containing the projectId, the number of contributions,
// the total contributed amount, and the matching amount.
type ClrTotal = {
  id: ProjectId;
  number_contributions: number;
  contribution_amount: YoctoBN;
  matching_amount: YoctoBN;
};
function calculateClr(
  aggregatedContributions: ContribDict,
  pairTotals: PairTotals,
  threshold: typeof BN,
  totalPot: YoctoBN
): ClrTotal[] {
  console.log("aggregated contributions: ", aggregatedContributions);
  console.log("pair totals: ", pairTotals);
  let bigtot = new BN(0);
  const totals: {
    id: string;
    number_contributions: number;
    contribution_amount: YoctoBN;
    matching_amount: YoctoBN;
  }[] = [];

  for (const [proj, contribz] of Object.entries(aggregatedContributions)) {
    let tot = new BN(0);
    let _num = 0;
    let _sum = new BN(0);

    for (const [k1, v1] of Object.entries(contribz)) {
      _num += 1;
      _sum = _sum.add(v1);
      for (const [k2, v2] of Object.entries(contribz)) {
        if (k2 > k1) {
          console.log("v1: ", v1.toString());
          console.log("v2: ", v2.toString());
          console.log("k2: ", k2, "k1: ", k1);
          console.log("pairTotals[k1][k2]: ", pairTotals[k1][k2].toString());
          console.log(
            "threshold.add(new BN(1)): ",
            threshold.add(new BN(1)).toString()
          );
          //   console.log(
          //     "bnsqrt.sqrt(v1.mul(v2)): ",
          //     bnsqrt.sqrt(v1.mul(v2)).toString()
          //   );
          const sqrt = bnsqrt.sqrt(v1.mul(v2));
          console.log("sqrt: ", sqrt.toString());
          const prod1 = sqrt.div(pairTotals[k1][k2]);
          console.log("prod1: ", prod1.toString());
          const prod2 = prod1.div(threshold.add(new BN(1)));
          console.log("prod2: ", prod2.toString());
          const addtl = bnsqrt
            .sqrt(v1.mul(v2))
            .div(pairTotals[k1][k2])
            .div(threshold.add(new BN(1)));
          console.log("addtl: ", addtl.toString());
          tot = tot.add(
            bnsqrt
              .sqrt(v1.mul(v2))
              .div(pairTotals[k1][k2])
              .div(threshold.add(new BN(1)))
          );
          console.log("tot: ", tot.toString());
          // TODO: DEBUG WHY TOT IS ALWAYS ZERO
          //   tot += Math.sqrt(v1 * v2) / (pairTotals[k1][k2] / (threshold + 1));
        }
      }
    }

    bigtot = bigtot.add(tot);
    totals.push({
      id: proj,
      number_contributions: _num,
      contribution_amount: _sum,
      matching_amount: tot,
    });
  }

  if (bigtot >= totalPot) {
    // Assuming CLR_PERCENTAGE_DISTRIBUTED is a mutable global variable
    CLR_PERCENTAGE_DISTRIBUTED = 100;
    for (const t of totals) {
      t.matching_amount = t.matching_amount.div(bigtot).mul(totalPot);
    }
  }

  return totals;
}

// This is the main function that ties everything together. It translates the data, aggregates contributions,
// calculates pairwise overlaps, and then calculates the CLR matching amounts.
// It returns the final list of matching amounts for each project.
function runClrCalcs(
  grantContribsCurr: GrantContribution[],
  threshold: typeof BN,
  totalPot: YoctoBN
): ClrTotal[] {
  //   const contribData = translateData(grantContribsCurr);
  const contributions = aggregateContributions(grantContribsCurr);
  const pairTotals = getTotalsByPair(contributions);
  const totals = calculateClr(contributions, pairTotals, threshold, totalPot);
  return totals;
}

// Sample call
const res = runClrCalcs(
  GRANT_CONTRIBUTIONS_EXAMPLE,
  new BN(25.0),
  new BN(5000)
);
console.log("res: ", res);
for (const obj of res) {
  console.log("matching amount:");
}
