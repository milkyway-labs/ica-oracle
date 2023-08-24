set -eu
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source ${SCRIPT_DIR}/vars.sh

contract_address=$(cat $METADATA/contract_address.txt)

echo "Adding metric..."

key=${KEY:-key1}
value=${VALUE:-value1}

attributes=$(cat << EOF | base64
{
    "sttoken_denom": "stuosmo",
    "native_token_denom": "ibc/ED07A3391A112B175915CD8FAF43A2DA8E4790EDE12566649D0C2F97716B8518"
}
EOF
)

msg=$(cat << EOF
{
    "post_metric": {
        "key": "stuosmo_redemption_rate",
        "value": "1.0303",
        "metric_type": "redemption_rate",
        "update_time": 100,
        "block_height": 1000,
        "attributes": "$attributes"
    }
}
EOF
)

echo ">>> osmosisd tx wasm execute $contract_address $msg"
tx_hash=$($OSMOSISD tx wasm execute $contract_address "$msg" --from oval1 -y $GAS | grep -E "txhash:" | awk '{print $2}')

echo "Tx Hash: $tx_hash"
echo $tx_hash > $METADATA/store_tx_hash.txt