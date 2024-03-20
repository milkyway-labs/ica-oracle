#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use semver::Version;

use crate::error::ContractError;
use crate::helpers::validate_channel_id;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg};
use crate::state::{Config, CONFIG};
use crate::{execute, query};

const CONTRACT_NAME: &str = "crates.io:stride-ica-oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if let Some(channel_id) = msg.transfer_channel_id.as_ref() {
        validate_channel_id(channel_id)?;
    }

    let config = Config {
        admin_address: deps.api.addr_validate(&msg.admin_address)?,
        transfer_channel_id: msg.transfer_channel_id.clone(),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin_address", msg.admin_address)
        .add_attribute(
            "transfer_channel_id",
            msg.transfer_channel_id
                .unwrap_or_else(|| "None".to_string()),
        ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostMetric {
            key,
            value,
            metric_type,
            update_time,
            block_height,
            attributes,
        } => execute::post_metric(
            deps,
            info,
            key,
            value,
            metric_type,
            update_time,
            block_height,
            attributes,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::Metric { key } => to_binary(&query::get_latest_metric(deps, key)?),
        QueryMsg::HistoricalMetrics { key, limit } => {
            to_binary(&query::get_historical_metrics(deps, key, limit)?)
        }
        QueryMsg::AllLatestMetrics {} => to_binary(&query::get_all_latest_metrics(deps)?),
        QueryMsg::RedemptionRate { denom, params, .. } => {
            to_binary(&query::get_latest_redemption_rate(deps, denom, params)?)
        }
        QueryMsg::HistoricalRedemptionRates {
            denom,
            params,
            limit,
            ..
        } => to_binary(&query::get_historical_redemption_rates(
            deps, denom, params, limit,
        )?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if &CONTRACT_NAME != &current_version.contract.as_str() {
        return Err(ContractError::InvalidContract {});
    }

    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| ContractError::InvalidContractVersion {})?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| ContractError::InvalidContractVersion {})?;

    // current version not launchpad v2
    if version > new_version {
        return Err(ContractError::InvalidContractVersion {});
    }
    // if same version return
    if version == new_version {
        return Err(ContractError::InvalidContractVersion {});
    }

    // migrate data
    // none

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::contract::{execute, instantiate, query};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{
        attr, from_binary, to_binary, Addr, Decimal, Empty, Env, MessageInfo, OwnedDeps,
    };

    use crate::error::ContractError;
    use crate::msg::{
        ExecuteMsg, InstantiateMsg, Metrics, QueryMsg, RedemptionRateResponse, RedemptionRates,
    };
    use crate::state::{Config, Metric, MetricType, RedemptionRate, RedemptionRateAttributes};

    const ADMIN_ADDRESS: &str = "admin";
    const TRANSFER_CHANNEL_ID: &str = "channel-0";

    const STTOKEN_DENOM: &str = "stdenom";
    const STTOKEN_IBC_DENOM: &str =
        "ibc/975F1E9343A04E8A4B174D0B72C68CFE05FE3BCBE0B6F71EEE8A6EB325870D62";

    // Helper function to return the default mocked env, info and deps
    fn default_mock() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        Env,
        MessageInfo,
    ) {
        let deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADMIN_ADDRESS, &[]);

        (deps, env, info)
    }

    // Helper function to instantiate the contract using the default ADMIN_ADDRESS
    fn default_instantiate() -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
        Env,
        MessageInfo,
    ) {
        let (mut deps, env, info) = default_mock();

        let msg = InstantiateMsg {
            admin_address: ADMIN_ADDRESS.to_string(),
            transfer_channel_id: Some(TRANSFER_CHANNEL_ID.to_string()),
        };

        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("admin_address", ADMIN_ADDRESS.to_string()),
                attr("transfer_channel_id", TRANSFER_CHANNEL_ID.to_string()),
            ]
        );

        (deps, env, info)
    }

    // Helper function to build a redemption rate object
    // The time field is used for both the time and the block_height
    // It uses a generic denom and ibc/denom
    fn get_test_redemption_rate_metric(key: &str, value: &str, time: u64) -> Metric {
        let redemption_rate_attributes = RedemptionRateAttributes {
            sttoken_denom: STTOKEN_DENOM.to_string(),
        };
        let redemption_rate_attributes = Some(to_binary(&redemption_rate_attributes).unwrap());

        Metric {
            key: key.to_string(),
            value: value.to_string(),
            metric_type: MetricType::RedemptionRate {},
            update_time: time,
            block_height: time,
            attributes: redemption_rate_attributes,
        }
    }

    // Helper function to build a redemption rate object
    fn get_test_redemption_rate(value: &str, time: u64) -> RedemptionRate {
        RedemptionRate {
            denom: STTOKEN_IBC_DENOM.to_string(),
            redemption_rate: Decimal::from_str(value).unwrap(),
            update_time: time,
        }
    }

    // Helper function to build the PostMetric message, given a metric
    fn get_post_metric_msg(metric: &Metric) -> ExecuteMsg {
        ExecuteMsg::PostMetric {
            key: metric.key.clone(),
            value: metric.value.clone(),
            metric_type: metric.metric_type.clone(),
            update_time: metric.update_time.clone(),
            block_height: metric.block_height.clone(),
            attributes: metric.attributes.clone(),
        }
    }

    #[test]
    fn test_instantiate_with_channel_id() {
        let (deps, env, _) = default_instantiate();

        // Confirm config was set properly
        let msg = QueryMsg::Config {};
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let config: Config = from_binary(&resp).unwrap();
        assert_eq!(
            config,
            Config {
                admin_address: Addr::unchecked(ADMIN_ADDRESS.to_string()),
                transfer_channel_id: Some(TRANSFER_CHANNEL_ID.to_string()),
            }
        )
    }

    #[test]
    fn test_instantiate_no_channel_id() {
        let (mut deps, env, info) = default_mock();

        let msg = InstantiateMsg {
            admin_address: ADMIN_ADDRESS.to_string(),
            transfer_channel_id: None,
        };

        let resp = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "instantiate"),
                attr("admin_address", ADMIN_ADDRESS.to_string()),
                attr("transfer_channel_id", "None"),
            ]
        );

        // Confirm config was set properly
        let msg = QueryMsg::Config {};
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let config: Config = from_binary(&resp).unwrap();
        assert_eq!(
            config,
            Config {
                admin_address: Addr::unchecked(ADMIN_ADDRESS.to_string()),
                transfer_channel_id: None,
            }
        )
    }

    #[test]
    fn test_instantiate_invalid_channel_id() {
        let (mut deps, env, info) = default_mock();

        let invalid_channel_id = "channel-";
        let msg = InstantiateMsg {
            admin_address: ADMIN_ADDRESS.to_string(),
            transfer_channel_id: Some(invalid_channel_id.to_string()),
        };

        let resp = instantiate(deps.as_mut(), env, info, msg);
        assert_eq!(
            resp,
            Err(ContractError::InvalidChannelID {
                channel_id: invalid_channel_id.to_string()
            })
        )
    }

    #[test]
    fn test_post_redemption_rate_metric() {
        // Instantiate contract
        let (mut deps, env, info) = default_instantiate();

        // Post a metric
        let metric = get_test_redemption_rate_metric("key1", "1", 1);
        let post_msg = get_post_metric_msg(&metric);

        let resp = execute(deps.as_mut(), env.clone(), info, post_msg).unwrap();
        let attributes_string = metric.attributes.clone().unwrap().to_string();
        assert_eq!(
            resp.attributes,
            vec![
                attr("action", "post_metric"),
                attr("metric_key", "key1"),
                attr("metric_value", "1"),
                attr("metric_type", "redemption_rate"),
                attr("metric_update_time", "1"),
                attr("metric_block_height", "1"),
                attr("metric_attributes", attributes_string),
            ]
        );

        // Confirm the metric is present
        let query_latest_msg = QueryMsg::Metric {
            key: metric.key.clone(),
        };

        let resp = query(deps.as_ref(), env.clone(), query_latest_msg).unwrap();
        let latest_response: Metric = from_binary(&resp).unwrap();
        assert_eq!(latest_response, metric);

        // Confirm the metric was added to the redemption rate store
        let query_redemption_rate_msg = QueryMsg::RedemptionRate {
            denom: STTOKEN_IBC_DENOM.to_string(),
            params: None,
        };
        let resp = query(deps.as_ref(), env, query_redemption_rate_msg).unwrap();
        let redemption_rate_response: RedemptionRateResponse = from_binary(&resp).unwrap();
        let expected_redemption_rate = Decimal::one();
        assert_eq!(
            redemption_rate_response,
            RedemptionRateResponse {
                redemption_rate: expected_redemption_rate,
                update_time: 1
            }
        );
    }

    #[test]
    fn test_historical_queries() {
        // Instantiate contract
        let (mut deps, env, info) = default_instantiate();

        // Build four metrics (all with the same key, and with a duplicate value) and post messages for each
        let metric_key = "key1";
        let metric1 = get_test_redemption_rate_metric(metric_key, "1", 1);
        let metric2 = get_test_redemption_rate_metric(metric_key, "2", 2);
        let metric3 = get_test_redemption_rate_metric(metric_key, "3", 2); // replaces previous
        let metric4 = get_test_redemption_rate_metric(metric_key, "4", 3);

        let rr1 = get_test_redemption_rate("1", 1);
        let rr2 = get_test_redemption_rate("3", 2);
        let rr3 = get_test_redemption_rate("4", 3);

        let msg1 = get_post_metric_msg(&metric1);
        let msg2 = get_post_metric_msg(&metric2);
        let msg3 = get_post_metric_msg(&metric3);
        let msg4 = get_post_metric_msg(&metric4);

        // Execute each message out of order, and with msg2 coming before msg3
        execute(deps.as_mut(), env.clone(), info.clone(), msg2).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg1).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg3).unwrap(); // should get ignored bc duplicate time
        execute(deps.as_mut(), env.clone(), info, msg4).unwrap();

        // Confirm metrics 1, 2 and 4 are preset and are sorted
        let msg = QueryMsg::HistoricalMetrics {
            key: metric_key.to_string(),
            limit: None,
        };
        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let history_response: Metrics = from_binary(&resp).unwrap();
        assert_eq!(
            history_response,
            Metrics {
                metrics: vec![metric4.clone(), metric3.clone(), metric1.clone()]
            }
        );

        // Check querying metrics with a limit
        let msg = QueryMsg::HistoricalMetrics {
            key: metric_key.to_string(),
            limit: Some(2),
        };
        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let history_response: Metrics = from_binary(&resp).unwrap();
        assert_eq!(
            history_response,
            Metrics {
                metrics: vec![metric4, metric3]
            }
        );

        // Check the corresponding redemption rate query
        let msg = QueryMsg::HistoricalRedemptionRates {
            denom: rr1.denom.clone(),
            params: None,
            limit: None,
        };
        let resp = query(deps.as_ref(), env.clone(), msg).unwrap();
        let history_response: RedemptionRates = from_binary(&resp).unwrap();
        assert_eq!(
            history_response,
            RedemptionRates {
                redemption_rates: vec![rr3.clone(), rr2.clone(), rr1.clone()]
            }
        );

        // Check the redemption rate query with a limit
        let msg = QueryMsg::HistoricalRedemptionRates {
            denom: rr1.denom.clone(),
            params: None,
            limit: Some(2),
        };
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let history_response: RedemptionRates = from_binary(&resp).unwrap();
        assert_eq!(
            history_response,
            RedemptionRates {
                redemption_rates: vec![rr3, rr2]
            }
        );
    }

    #[test]
    fn test_all_latest_metrics() {
        // Instantiate contract
        let (mut deps, env, info) = default_instantiate();

        // Build three metrics - each with a new and an old record
        let metric1_old = get_test_redemption_rate_metric("key1", "1", 0);
        let metric2_old = get_test_redemption_rate_metric("key2", "2", 0);
        let metric3_old = get_test_redemption_rate_metric("key3", "3", 0);

        let metric1_new = get_test_redemption_rate_metric("key1", "1", 1);
        let metric2_new = get_test_redemption_rate_metric("key2", "2", 2);
        let metric3_new = get_test_redemption_rate_metric("key3", "3", 3);

        // Build a post message for each
        let msg1_old = get_post_metric_msg(&metric1_old);
        let msg2_old = get_post_metric_msg(&metric2_old);
        let msg3_old = get_post_metric_msg(&metric3_old);

        let msg1_new = get_post_metric_msg(&metric1_new);
        let msg2_new = get_post_metric_msg(&metric2_new);
        let msg3_new = get_post_metric_msg(&metric3_new);

        // Execute each message
        execute(deps.as_mut(), env.clone(), info.clone(), msg1_old).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg2_old).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg3_old).unwrap();

        execute(deps.as_mut(), env.clone(), info.clone(), msg1_new).unwrap();
        execute(deps.as_mut(), env.clone(), info.clone(), msg2_new).unwrap();
        execute(deps.as_mut(), env.clone(), info, msg3_new).unwrap();

        // Confirm all metrics are preset and are sorted
        let msg = QueryMsg::AllLatestMetrics {};
        let resp = query(deps.as_ref(), env, msg).unwrap();
        let metric_responses: Metrics = from_binary(&resp).unwrap();
        assert_eq!(
            metric_responses,
            Metrics {
                metrics: vec![metric1_new, metric2_new, metric3_new]
            }
        )
    }

    #[test]
    fn test_post_metric_unauthorized() {
        // Instantiate contract
        let (mut deps, env, _) = default_instantiate();

        // Change info to have non-admin sender
        let invalid_info = mock_info("not_admin", &[]);

        // Attempt to post the message - it should fail
        let metric = get_test_redemption_rate_metric("key1", "1", 1);
        let post_msg = get_post_metric_msg(&metric);
        let resp = execute(deps.as_mut(), env, invalid_info, post_msg);
        assert_eq!(resp, Err(ContractError::Unauthorized {}))
    }

    #[test]
    fn test_post_redemption_rate_missing_attributes() {
        // Instantiate contract
        let (mut deps, env, info) = default_instantiate();

        // Build a metric object with None for the attributes
        let mut invalid_metric = get_test_redemption_rate_metric("key1", "1", 1);
        invalid_metric.attributes = None;

        // Attempt to post the message, it should fail
        let post_msg_failure = get_post_metric_msg(&invalid_metric);
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), post_msg_failure);
        assert_eq!(
            resp,
            Err(ContractError::MissingMetricMetadataAttributes {
                metric_type: invalid_metric.metric_type.clone(),
            })
        );

        // Now change the metric_type, so that it's not redemption_rate
        let valid_metric = Metric {
            key: "key2".to_string(),
            metric_type: MetricType::Other("something_else".to_string()),
            ..invalid_metric
        };

        // Now the message should succeed
        let post_msg_success = get_post_metric_msg(&valid_metric);
        execute(deps.as_mut(), env.clone(), info, post_msg_success).unwrap();

        // Confirm the metric is present
        let query_latest_msg = QueryMsg::Metric {
            key: valid_metric.key.clone(),
        };
        let resp = query(deps.as_ref(), env, query_latest_msg).unwrap();
        let latest_response: Metric = from_binary(&resp).unwrap();
        assert_eq!(latest_response, valid_metric);
    }

    #[test]
    fn test_post_redemption_rate_invalid_attributes() {
        // Instantiate contract
        let (mut deps, env, info) = default_instantiate();

        // Build a metric object with an gibberish string so that it can't be deserialized as attributes
        let mut invalid_metric = get_test_redemption_rate_metric("key1", "1", 1);
        invalid_metric.attributes = Some(to_binary(&"{cantparse}".to_string()).unwrap());

        // Attempt to post the message, it should fail
        let post_msg_failure = get_post_metric_msg(&invalid_metric);
        let resp = execute(deps.as_mut(), env.clone(), info.clone(), post_msg_failure);
        assert_eq!(
            resp,
            Err(ContractError::InvalidMetricMetadataAttributes {
                metric_type: invalid_metric.metric_type.clone(),
            })
        );

        // Now change the metric_type, so that it's not redemption_rate
        let valid_metric = Metric {
            key: "key2".to_string(),
            metric_type: MetricType::Other("something_else".to_string()),
            ..invalid_metric
        };

        // Now the message should succeed
        let post_msg_success = get_post_metric_msg(&valid_metric);
        execute(deps.as_mut(), env.clone(), info, post_msg_success).unwrap();

        // Confirm the metric is present
        let query_latest_msg = QueryMsg::Metric {
            key: valid_metric.key.clone(),
        };
        let resp = query(deps.as_ref(), env, query_latest_msg).unwrap();
        let latest_response: Metric = from_binary(&resp).unwrap();
        assert_eq!(latest_response, valid_metric);
    }
}
