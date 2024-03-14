# ICA Oracle CW Contract

## Overview
The `icaoracle` facilities trustless data publication to cosmwasm outposts on adjacent chains. The full ICA Oracle solution consists of two components: the `icaoracle` module deployed on the source chain, as well as a corresponding cosmwasm oracle contract deployed on the destination chain (described here). The contract features a standard key-value store, that accepts push messages from this module via interchain accounts. The data sent is referred to as a `Metric`. For Stride, the primary application of this module is to enable integrations to trustlessly retrieve the redemption rate (internal exchange rate) of stTokens.

## Pushing Metrics
This contract consists of a single transaction `PostMetric` that is responsible for publishing data to the oracle. Only the admin can push metrics to the oracle. The source chain controlled interchain account controlled is the contract admin. 

Each metrics is represented as a generic key-value pair. When a metric is pushed, it's added to the `METRICS` store, which keeps track of the latest 100 values for the particular key. 

Additionally, there's a `REDEMPTION_RATES` store that is meant specifically for redemption rate metrics. These redemption rate metrics are identified by the `metric_type` field which enables additional handling for metrics that fall into the same category.

## Diagram
![alt text](https://github.com/Stride-Labs/ica-oracle/blob/main/docs/post-metric-1.png?raw=true)
![alt text](https://github.com/Stride-Labs/ica-oracle/blob/main/docs/post-metric-2.png?raw=true)

## Transactions
```rust
pub struct InstantiateMsg {
    /// Contract admin address which has authority to post metrics
    /// This will be an ICA account
    pub admin_address: String,
    /// The transfer channel ID from the Oracle chain to the Controller chain
    /// This field is only necessary for redemption rate metrics and queries
    ///
    /// Ex: If this oracle contract is deployed on Osmosis, but is controlled
    ///     by Stride, then the channel ID should be the Osmosis channel ID
    ///     for the Osmosis <> Stride transfer channel
    pub transfer_channel_id: Option<String>,
}

pub enum ExecuteMsg {
    /// Uploads and stores a new metric
    PostMetric {
        /// Key identifying the metric (e.g. `stuatom_redemption_rate`)
        key: String,
        /// Value for the metric (e.g. `1.1`)
        value: String,
        /// Category for the metric(e.g. `redemption_rate`)
        /// Helps determine handling of additional context
        metric_type: MetricType,
        /// Unix timestamp with which the metric was updated on the source chain
        update_time: u64,
        /// Block height with which the metric was updated on the source chain
        block_height: u64,
        /// Additional metric-specific attributes
        attributes: Option<Binary>,
    },
}
```

## Queries
```rust
pub enum QueryMsg {
    /// Returns the contract's config
    #[returns(crate::state::Config)]
    Config {},

    /// Returns the latest metric, given the metric's key
    #[returns(Metric)]
    Metric { key: String },

    /// Returns the full history of values for a given metric key, up to the capacity
    /// Includes optional limit on the number of metrics returned
    #[returns(Metrics)]
    HistoricalMetrics { key: String, limit: Option<u64> },

    /// Returns the latest metric for each key
    #[returns(Metrics)]
    AllLatestMetrics {},

    /// Returns the redemption rate of an stToken
    #[returns(RedemptionRateResponse)]
    RedemptionRate {
        /// The denom should be the ibc hash of an stToken as it lives on the oracle chain
        /// (e.g. ibc/{hash(transfer/channel-326/stuatom)} on Osmosis)
        denom: String,
        /// Params should always be None, but was included in this query
        /// to align with other price oracles that take additional parameters such as TWAP
        params: Option<Binary>,
    },

    /// Returns a list of redemption rates over time for an stToken
    #[returns(RedemptionRates)]
    HistoricalRedemptionRates {
        /// The denom should be the ibc hash of an stToken as it lives on the oracle chain
        /// (e.g. ibc/{hash(transfer/channel-326/stuatom)} on Osmosis)
        denom: String,
        /// Params should always be None, but was included in this query
        /// to align with other price oracles that take additional parameters such as TWAP
        params: Option<Binary>,
        /// Optional limit on the number of entries to return
        limit: Option<u64>,
    },

    /// Returns the purchase rate of an stToken
    #[returns(PurchaseRateResponse)]
    PurchaseRate {
        /// The denom should be the ibc hash of an stToken as it lives on the oracle chain
        denom: String,
        /// Params should always be None, but was included in this query
        /// to align with other price oracles that take additional parameters such as TWAP
        params: Option<Binary>,
    },

}
```

## Instructions for Testing Locally
<!-- TODO: Change these instructions once this contract in the Stride repo -->
* Clone this repo so that it sits at the same directory level as the Stride repo
* Navigate to the stride repo
```bash
cd ../stride
```
* Update the `HOST_CHAINS` variable in [config.sh](https://github.com/Stride-Labs/stride/blob/4b1c63332452b2772dc1b26b47547975b8cbd8e0/dockernet/config.sh#L19) to run only osmosis
```bash
HOST_CHAINS=(OSMO)
```
* Start dockernet from Stride repo home directory
```bash
git submodule update --init --recursive
make start-docker build=sor

# Each ensuing run, you can just run `make start-docker` which will only rebuild the Stride binary
```
* Get intergration tests running in background so redemption rate updates
```
make test-integration-docker
```
<!-- * Navigate to this contract
```
cd x/icaoracle/contracts/icaoracle
``` -->
* Navigate to the ica-oracle repo
```bash
cd ../ica-oracle
```
* Build the contract
```
make build-optimized
```
* Upload the contract
```
make store-contract 
```
* Add the oracle to Stride which will instantiate the contract via an ICA
```
make add-oracle
```
* Query the metrics and watch the redemption rate grow. Note: the query will fail if no redemption rate updates have been pushed yet.
```
make query-metrics
```
* See makefile for additional commands
