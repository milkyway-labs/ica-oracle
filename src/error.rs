use crate::state::MetricType;

use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("The provided metric (type {metric_type:?}) does not contain required attributes")]
    MissingMetricMetadataAttributes { metric_type: MetricType },

    #[error("The provided metric (type {metric_type:?}) has invalid metadata attributes")]
    InvalidMetricMetadataAttributes { metric_type: MetricType },

    #[error("Invalid denom: {reason}")]
    InvalidDenom { reason: String },

    #[error("Invalid channelID: {channel_id}")]
    InvalidChannelID { channel_id: String },

    #[error("The oracle to controller chain transfer channel ID must be configured during instantiation when using redemption rate metrics")]
    MissingTransferChannelID {},

    #[error("The denom for the redemption rate metric must not be an IBC denom, {denom} provided")]
    InvalidRedemptionRateDenom { denom: String },

    #[error("Cannot upgrade to a different contract")]
    InvalidContract {},

    #[error("Invalid contract version")]
    InvalidContractVersion {},
}
