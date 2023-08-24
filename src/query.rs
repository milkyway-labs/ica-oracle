use crate::msg::{Metrics, RedemptionRateResponse, RedemptionRates};
use crate::state::{Metric, METRICS, REDEMPTION_RATES};
use cosmwasm_std::{Binary, Deps, Order, StdError, StdResult};

/// Returns the most up-to-date metric for all metrics stored
pub fn get_all_latest_metrics(deps: Deps) -> StdResult<Metrics> {
    let metrics: Vec<Metric> = METRICS
        .range(deps.storage, None, None, Order::Ascending)
        .filter_map(|r| r.ok().and_then(|(_, history)| history.get_latest()))
        .collect();

    Ok(Metrics { metrics })
}

/// Returns the most recent metric value for a given key
pub fn get_latest_metric(deps: Deps, key: String) -> StdResult<Metric> {
    let metrics_history = METRICS.load(deps.storage, &key)?;
    match metrics_history.get_latest() {
        Some(metric) => Ok(metric),
        None => Err(StdError::generic_err("metric key not found")),
    }
}

/// Returns the full history of a given metric, sorted by the time at which it was updated
pub fn get_historical_metrics(deps: Deps, key: String, limit: Option<u64>) -> StdResult<Metrics> {
    let metrics_history = METRICS.load(deps.storage, &key)?;
    let metrics = match limit {
        Some(limit) => metrics_history.get_latest_range(limit as usize),
        None => metrics_history.get_all(),
    };
    Ok(Metrics { metrics })
}

/// Returns the redemption rate of a given stToken and the time that it was last updated (used for price oracles)
pub fn get_latest_redemption_rate(
    deps: Deps,
    denom: String,
    params: Option<Binary>,
) -> StdResult<RedemptionRateResponse> {
    // The params field of the redemption rate query should always be None
    // It is included so that the query is at parity with other price oracles that
    // may use it for things like TWAP
    if params.is_some() {
        return Err(StdError::generic_err(
            "invalid query request - params must be None",
        ));
    }

    let redemption_rates_history = REDEMPTION_RATES.load(deps.storage, &denom)?;

    match redemption_rates_history.get_latest() {
        Some(response) => Ok(RedemptionRateResponse {
            redemption_rate: response.redemption_rate,
            update_time: response.update_time,
        }),
        None => Err(StdError::generic_err("redemption rate not found")),
    }
}

/// Returns the full redemption rate history of an stToken, sorted by the time at which it was updated
pub fn get_historical_redemption_rates(
    deps: Deps,
    denom: String,
    params: Option<Binary>,
    limit: Option<u64>,
) -> StdResult<RedemptionRates> {
    // The params field of the redemption rate query should always be None
    // It is included so that the query is at parity with other price oracles that
    // may use it for things like TWAP
    if params.is_some() {
        return Err(StdError::generic_err(
            "invalid query request - params must be None",
        ));
    }

    let redemption_rates_history = REDEMPTION_RATES.load(deps.storage, &denom)?;

    let redemption_rates = match limit {
        Some(limit) => redemption_rates_history.get_latest_range(limit as usize),
        None => redemption_rates_history.get_all(),
    };
    Ok(RedemptionRates { redemption_rates })
}
