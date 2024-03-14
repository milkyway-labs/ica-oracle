use crate::error::ContractError;
use crate::state::{
    History, Metric, MetricType, RedemptionRate, RedemptionRateAttributes, 
    PurchaseRate, PurchaseRateAttributes, CONFIG, METRICS,
    REDEMPTION_RATES, PURCHASE_RATES,
};
use cosmwasm_std::{ensure, from_binary, Binary, Decimal, DepsMut, MessageInfo, Response};
use std::str::FromStr;

/// Stores a given metric passed via an ICA from a source chain
/// The oracle stores each metric generically with key and value attributes
///
/// The metric is added to the store if:
///   * a metric with that key and time combo has never been submitted, AND
///   * the historical list is not at capacity, OR
///      * the historical list is at capacity, but the metric is more recent than the oldest metric in the store
///
/// Only metrics with metric_type "redemption_rate" are added to the REDEMPTION_RATES store
#[allow(clippy::too_many_arguments)]
pub fn post_metric(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    value: String,
    metric_type: MetricType,
    update_time: u64,
    block_height: u64,
    attributes: Option<Binary>,
) -> Result<Response, ContractError> {
    // Only the ICA account can post metrics
    let config = CONFIG.load(deps.storage)?;
    ensure!(
        info.sender == config.admin_address,
        ContractError::Unauthorized {}
    );

    // Build the new metric object
    let new_metric = Metric {
        key: key.clone(),
        value,
        metric_type: metric_type.clone(),
        update_time,
        block_height,
        attributes: attributes.clone(),
    };

    // Add the metric to the store
    // If a duplicate metric is added, it will replace the existing one
    // If the list is at capacity and this metric is older than the oldest one in the list,
    // it will not be added
    let mut metric_history = match METRICS.may_load(deps.storage, &key)? {
        Some(history) => history,
        None => History::<Metric>::default(),
    };
    metric_history.add(new_metric.clone());
    METRICS.save(deps.storage, &key, &metric_history)?;

    // Parse the metric_type field and handle any other metric-type specific cases
    match metric_type {
        // If the metric is a redemption rate update, add record to the redemption rate store
        MetricType::RedemptionRate => {
            // Deserialize the metric attributes to get the denom and base denom
            let attributes: RedemptionRateAttributes = if let Some(attributes) = attributes {
                from_binary(&attributes).map_err(|_| {
                    ContractError::InvalidMetricMetadataAttributes {
                        metric_type: new_metric.metric_type.clone(),
                    }
                })?
            } else {
                return Err(ContractError::MissingMetricMetadataAttributes {
                    metric_type: new_metric.metric_type,
                });
            };

            let sttoken_denom = attributes.sttoken_denom.clone();

            // the contract is deployed on the same chain
            // // Get the transfer channel ID between the oracle and controller chain,
            // // as defined on the oracle chain
            // let transfer_channel_id = match config.transfer_channel_id {
            //     Some(channel_id) => channel_id,
            //     None => return Err(ContractError::MissingTransferChannelID {}),
            // };

            // // Convert the stToken denom to the ibc denom as it lives on the oracle
            // let sttoken_denom = attributes.sttoken_denom.clone();
            // validate_native_denom(&sttoken_denom)?;

            // let sttoken_ibc_denom = denom_trace_to_hash(&sttoken_denom, &transfer_channel_id)?;

            // Store the redemption rate in the redemption rate table
            let redemption_rate_value = Decimal::from_str(&new_metric.value)?;
            let new_redemption_rate = RedemptionRate {
                denom: sttoken_denom.clone(),
                redemption_rate: redemption_rate_value,
                update_time: new_metric.update_time,
            };

            let mut redemption_rate_history =
                match REDEMPTION_RATES.may_load(deps.storage, &sttoken_denom)? {
                    Some(history) => history,
                    None => History::<RedemptionRate>::default(),
                };
            redemption_rate_history.add(new_redemption_rate);
            REDEMPTION_RATES.save(deps.storage, &sttoken_denom, &redemption_rate_history)?;
        }
        MetricType::PurchaseRate => {
            // Deserialize the metric attributes to get the denom and base denom
            let attributes: PurchaseRateAttributes = if let Some(attributes) = attributes {
                from_binary(&attributes).map_err(|_| {
                    ContractError::InvalidMetricMetadataAttributes {
                        metric_type: new_metric.metric_type.clone(),
                    }
                })?
            } else {
                return Err(ContractError::MissingMetricMetadataAttributes {
                    metric_type: new_metric.metric_type,
                });
            };

            let sttoken_denom = attributes.sttoken_denom.clone();
            let purchase_rate_value = Decimal::from_str(&new_metric.value)?;
            let new_purchase_rate = PurchaseRate {
                denom: sttoken_denom.clone(),
                purchase_rate: purchase_rate_value,
                update_time: new_metric.update_time,
            };

            let mut purchase_rate_history =
                match PURCHASE_RATES.may_load(deps.storage, &sttoken_denom)? {
                    Some(history) => history,
                    None => History::<PurchaseRate>::default(),
                };
            purchase_rate_history.add(new_purchase_rate);
            PURCHASE_RATES.save(deps.storage, &sttoken_denom, &purchase_rate_history)?;
        }
        MetricType::Other(_) => {}
    }

    Ok(Response::new()
        .add_attribute("action", "post_metric")
        .add_attribute("metric_key", new_metric.key)
        .add_attribute("metric_value", new_metric.value)
        .add_attribute("metric_type", new_metric.metric_type.to_string())
        .add_attribute("metric_update_time", new_metric.update_time.to_string())
        .add_attribute("metric_block_height", new_metric.block_height.to_string())
        .add_attribute(
            "metric_attributes",
            new_metric
                .attributes
                .map_or("None".to_string(), |bin| bin.to_string()),
        ))
}
