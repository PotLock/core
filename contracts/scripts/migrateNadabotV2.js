import { parseNearAmount } from "near-api-js/lib/utils/format.js";
import { attachedDeposit, gas, getAccount } from "./near-utils.js";
import fs from "fs";
import { parse } from "csv-parse";
import { generateSeedPhrase } from "near-seed-phrase";

(async () => {
  // const networkId = "testnet";
  const networkId = "mainnet";
  const contractId = "v2new.staging.nadabot.near";
  const callerId = "nadabot.near";
  // const callerId = contractId;
  const callingAccount = await getAccount(networkId, callerId);

  // 1. MIGRATE PROVIDERS

  const providers = await callingAccount.viewFunction(
    "v1.nadabot.near",
    "get_providers",
    {}
  );
  console.log("providers: ", providers);

  const args = {
    providers: providers.map((p) => ({
      ...p,
      provider_name: p.name,
      stamp_count: 0,
    })),
  };
  console.log("register providers args: ", args);
  try {
    const res = await callingAccount.functionCall({
      contractId,
      methodName: "_register_providers_unsafe",
      args,
      gas,
      attachedDeposit,
    });
    console.log("_register_providers_unsafe res: ", res);
  } catch (e) {
    console.log("error registering providers: ", e);
  }

  // 2. MIGRATE STAMPS

  let cur_start = 0;
  let limit = 250;
  let next = true;
  let allStamps = [];
  while (next) {
    try {
      console.log(
        "getting stamps from index: ",
        cur_start,
        " to ",
        cur_start + limit
      );
      const stamps = await callingAccount.viewFunction(
        "v1.nadabot.near",
        "get_stamps",
        {
          from_index: cur_start,
          limit,
        }
      );
      console.log("num stamps fetched: ", stamps.length);
      allStamps = allStamps.concat(stamps);
      if (stamps.length < limit) {
        next = false;
        break;
      } else {
        cur_start += limit;
      }
    } catch (e) {
      console.log("error: ", e);
      break;
    }
  }
  console.log("all stamps: ", allStamps);
  console.log("num stamps: ", allStamps.length);

  const providersMap = {};

  const providersV2 = await callingAccount.viewFunction(
    contractId,
    "get_providers",
    {
      // from_index: cur_start,
      // limit,
    }
  );
  console.log("providersV2: ", providersV2);
  console.log("num providersV2: ", providersV2.length);
  for (let i = 0; i < providersV2.length; i++) {
    const provider = providersV2[i];
    providersMap[`${provider.contract_id}:${provider.method_name}`] =
      provider.id;
  }
  console.log("providersMap: ", providersMap);

  const batchSize = 50;
  for (let i = 0; i < allStamps.length; i += batchSize) {
    const batch = allStamps.slice(i, i + batchSize).map((stamp) => {
      return {
        user_id: stamp.user_id,
        provider_id: providersMap[stamp.provider.provider_id],
        validated_at_ms: stamp.validated_at_ms,
      };
    });
    console.log("batch: ", batch);
    const args = {
      stamps: batch, // Adjust the argument name based on your smart contract's expected parameter
    };

    try {
      console.log(`Migrating batch starting from index ${i}`);
      console.log("args: ", args);
      const res = await callingAccount.functionCall({
        contractId,
        methodName: "_add_stamps_unsafe",
        args,
        gas,
        attachedDeposit,
      });
      console.log(`Migration result for batch starting from index ${i}:`, res);
    } catch (e) {
      console.log(`Error migrating batch starting from index ${i}:`, e);
      // Decide if you want to break out of the loop or just log the error and continue
      break; // Remove this line if you want to continue with the next batch even after an error
    }
  }
})();
