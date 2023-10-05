#!/bin/sh

if [ $? -ne 0 ]; then
  echo ">> Error building contract"
  exit 1
fi

echo ">> Deploying PotDeployer contract!"

# NB: Not using near dev-deploy here because the PotDeployer contract creates subaccounts of itself, and account IDs created via near dev-deploy cannot be subaccounted.

MASTER_ACCOUNT_NAME="test-contracts.potlock.testnet"

# Generate account name
account_name=$(date +"%s").$MASTER_ACCOUNT_NAME
echo ">> Generated account name: $account_name"

# 1. Create a new account
# Assuming you have a master account that will be used to create the new account.
# Replace MASTER_ACCOUNT and MASTER_ACCOUNT_KEY_PATH with appropriate values
near create-account $account_name --masterAccount $MASTER_ACCOUNT_NAME --initialBalance 15 --publicKey ed25519:4PAtgM8Hvz4MamxUaoSN2q3HJ3Npc3oCzyyHK7Y1vncf

# 2. Funding is implicitly done in the create-account step by setting the initial balance.

# 3. Create a JSON file for the new account credentials
# Assuming MASTER_ACCOUNT_NAME.json is present in the ~/.near-credentials/testnet/ directory
# Make sure jq is installed (you can install it using `apt install jq` or appropriate command for your OS)
MASTER_ACCOUNT_JSON_PATH=~/.near-credentials/testnet/$MASTER_ACCOUNT_NAME.json
NEW_ACCOUNT_JSON_PATH=~/.near-credentials/testnet/$account_name.json

jq --arg account_name "$account_name" '.account_id = $account_name' $MASTER_ACCOUNT_JSON_PATH > $NEW_ACCOUNT_JSON_PATH
echo ">> Created new account credentials at $NEW_ACCOUNT_JSON_PATH."

# 4. Deploy the contract to the new account
near deploy --wasmFile ./out/main.wasm --accountId $account_name


# 5. Update ./neardev/dev-account with the new account name
mkdir -p ./neardev
echo $account_name > ./neardev/dev-account
echo ">> Updated ./neardev/dev-account with new account name."

echo ">> Deployment completed!"