set -eu
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source ${SCRIPT_DIR}/vars.sh

contract_address=$(cat $METADATA/contract_address.txt)

echo "ALL_LATEST_METRICS"
msg='{ "all_latest_metrics" : { } }'
echo ">>> osmosisd q wasm contract-state smart $contract_address $msg"
$OSMOSISD q wasm contract-state smart $contract_address "$msg"
sleep 1

printf "\nMETRICS\n"
msg='{ "historical_metrics" : { "key": "stuosmo_redemption_rate" } }'
echo ">>> osmosisd q wasm contract-state smart $contract_address $msg"
$OSMOSISD q wasm contract-state smart $contract_address "$msg"
sleep 1

printf "\nREDEMPTION_RATE\n"
msg='{ "redemption_rate" : { "denom": "'$STOSMO_IBC_DENOM'" } }'
echo ">>> osmosisd q wasm contract-state smart $contract_address $msg"
$OSMOSISD q wasm contract-state smart $contract_address "$msg"
sleep 1

printf "\nREDEMPTION_RATES\n"
msg='{ "historical_redemption_rates" : { "denom": "'$STOSMO_IBC_DENOM'" } }'
echo ">>> osmosisd q wasm contract-state smart $contract_address $msg"
$OSMOSISD q wasm contract-state smart $contract_address "$msg"
sleep 1