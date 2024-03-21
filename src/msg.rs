use crate::state::{Metric, MetricType, RedemptionRate, PurchaseRate};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Decimal};

#[cw_serde]
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

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
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

    /// Returns the purchase rate of an milkTia
    #[returns(PurchaseRateResponse)]
    PurchaseRate {
        /// The denom should be the ibc hash of an milkTia as it lives on the oracle chain
        denom: String,
        /// Params should always be None, but was included in this query
        /// to align with other price oracles that take additional parameters such as TWAP
        params: Option<Binary>,
    },

    /// Returns a list of redemption rates over time for an stToken
    #[returns(PurchaseRates)]
    HistoricalPurchaseRates {
        /// The denom should be the ibc hash of an stToken as it lives on the oracle chain
        /// (e.g. ibc/{hash(transfer/channel-326/stuatom)} on Osmosis)
        denom: String,
        /// Params should always be None, but was included in this query
        /// to align with other price oracles that take additional parameters such as TWAP
        params: Option<Binary>,
        /// Optional limit on the number of entries to return
        limit: Option<u64>,
    },
}

#[cw_serde]
pub struct Metrics {
    pub metrics: Vec<Metric>,
}

#[cw_serde]
pub struct RedemptionRateResponse {
    pub redemption_rate: Decimal,
    pub update_time: u64,
}

#[cw_serde]
pub struct PurchaseRateResponse {
    pub purchase_rate: Decimal,
    pub update_time: u64,
}

#[cw_serde]
pub struct RedemptionRates {
    pub redemption_rates: Vec<RedemptionRate>,
}

#[cw_serde]
pub struct PurchaseRates {
    pub purchase_rates: Vec<PurchaseRate>,
}

#[cw_serde]
pub struct MigrateMsg {}
