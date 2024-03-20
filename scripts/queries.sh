osmosisd query wasm contract-state smart execute osmo1zd4l5u7m55aq8cdrd48uahslxx067kw9cktj2882d7dulhtqhc9qtygq9p '{"all_latest_metrics":{}}' \
    --chain-id osmosis-dev-1 \
    --gas 800000 \
    --gas-prices 0.025stake \
    --gas-adjustment 1.4 \
    --node tcp://localhost:26657 \
    -b block \
    --yes
