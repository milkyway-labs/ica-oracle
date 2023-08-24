set -eu
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
source ${SCRIPT_DIR}/vars.sh

echo ">>> osmosisd q txs --events 'ics27_packet.module=interchainaccounts' and 'ics_27packet.error=*'"
$OSMOSISD q txs --events 'ics27_packet.module=interchainaccounts' and 'ics_27packet.error=*'
