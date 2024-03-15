use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Decimal};
use std::{collections::VecDeque, fmt, usize};

use cw_storage_plus::{Item, Map};

/// The contract config consists of an admin address and optional
/// transfer chanel ID
#[cw_serde]
pub struct Config {
    /// The admin address will be the ICA address for the account that's
    ///  owned by the source chain and lives on the contract chain
    pub admin_address: Addr,
    /// The transfer channel ID from the Oracle chain to the Controller chain,
    /// as defined on the Oracle chain
    /// This field is only necessary for redemption rate metrics and queries
    pub transfer_channel_id: Option<String>,
}

/// This contract represents a generic key value store
/// A "metric" is the term for a piece of information stored
/// Each metric has a higher level category that helps inform if any other,
/// metric-specific logic needs to be run
/// i.e. For redemption rates, there is an expected format for the attributes
/// field with additional metadata
#[cw_serde]
pub enum MetricType {
    RedemptionRate,
    PurchaseRate,
    Other(String),
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetricType::RedemptionRate => write!(f, "redemption_rate"),
            MetricType::PurchaseRate => write!(f, "purchase_rate"),
            MetricType::Other(inner) => write!(f, "{inner}"),
        }
    }
}

/// The Metric struct represents the base unit for the generic oracle key-value store
///  - key/value represent the main piece of data that is intended to be stored
///  - metric_type represents a high level category for the metric
///  - update_time is the time at which the value was updated on the source chain
///  - block_height is the height at which the value was updated on the source chain
///  - attributes field contains any additional context that's needed
#[cw_serde]
pub struct Metric {
    pub key: String,
    pub value: String,
    pub metric_type: MetricType,
    pub update_time: u64,
    pub block_height: u64,
    pub attributes: Option<Binary>,
}

impl HasTime for Metric {
    fn time(&self) -> u64 {
        self.update_time
    }
}

/// For use in price oracles, the RedemptionRate metric requires the stToken denom
/// as it appears on the controller chain (e.g. `stuosmo`)
#[cw_serde]
pub struct RedemptionRateAttributes {
    pub sttoken_denom: String,
}

/// For use in price oracles, the PurchaseRate metric requires the milkTia denom
/// as it appears on the controller chain (e.g. `stuosmo`)
#[cw_serde]
pub struct PurchaseRateAttributes {
    pub sttoken_denom: String,
}

/// The RedemptionRate struct represents the redemption rate of an stToken
#[cw_serde]
pub struct RedemptionRate {
    /// stToken denom as an IBC hash, as it appears on the oracle chain
    pub denom: String,
    /// The redemption rate of the stToken
    pub redemption_rate: Decimal,
    /// The unix timestamp representing when the redemption rate was last updated
    pub update_time: u64,
}

impl HasTime for RedemptionRate {
    fn time(&self) -> u64 {
        self.update_time
    }
}

/// The PurchaseRate struct represents the purchase rate of an milkTia
#[cw_serde]
pub struct PurchaseRate {
    /// stToken denom as an IBC hash, as it appears on the oracle chain
    pub denom: String,
    /// The purchase rate of the milkTia
    pub purchase_rate: Decimal,
    /// The unix timestamp representing when the purchase rate was last updated
    pub update_time: u64,
}

impl HasTime for PurchaseRate {
    fn time(&self) -> u64 {
        self.update_time
    }
}

/// The history of each metric is also stored in the contract to enable
///   historical queries or averaging/smoothing
/// For each metric, the history is stored in a deque with a max capacity
/// The deque is sorted by the time at which the metric was updated on the source chain
/// This allows for the efficient insertion of new metrics (to the back of the deque in most cases)
///  as well as the efficient range look of the most recent items (also pulled from the back of the deque)
/// Since there is no current use case for storing all history indefinitely, the list is pruned by removing
///  elements from the front of the deque when the capacity has been reached
pub trait HasTime {
    fn time(&self) -> u64;
}

#[cw_serde]
pub struct History<T: HasTime + Clone> {
    deque: VecDeque<T>,
    capacity: u64,
}

const HISTORY_ITEM_CAP: u64 = 100;

impl<T: HasTime + Clone> Default for History<T> {
    fn default() -> Self {
        Self::new(HISTORY_ITEM_CAP)
    }
}

impl<T: HasTime + Clone> History<T> {
    // Instantiates a new deque with a fixed capacity
    pub fn new(capacity: u64) -> Self {
        History {
            deque: VecDeque::with_capacity(capacity as usize),
            capacity,
        }
    }

    // Adds a new items to the deque such that the list remains sorted in
    // reverse time order (meaning the newest item is at index 0)
    //
    // It first checks if a metric with the same timestamp is found
    //   If a metric is found with the same timestamp
    //     -> that implies the metric is a duplciate
    //     -> binary_search_by_key will return Ok
    //     -> replace the old metric with the new metric
    //   If the same timestamp is not found
    //      -> that implies this metric is new
    //      -> binary_search_by_key will return Err
    //      -> we insert the new metric to the list
    //
    // Old items are removed from the front of the deque when capacity is reached
    pub fn add(&mut self, item: T) {
        match self.deque.binary_search_by_key(&item.time(), |m| m.time()) {
            Ok(index) => {
                self.deque.remove(index);
                self.deque.insert(index, item)
            }
            Err(index) => {
                self.deque.insert(index, item);
                if self.deque.len() > self.capacity as usize {
                    self.deque.pop_front();
                }
            }
        }
    }

    // Grabs the most recent item from the deque
    pub fn get_latest(&self) -> Option<T> {
        self.deque.back().cloned()
    }

    // Grabs the most recent N items from the deque
    pub fn get_latest_range(&self, n: usize) -> Vec<T> {
        self.deque.iter().rev().take(n).cloned().collect()
    }

    // Returns all items as a list
    pub fn get_all(&self) -> Vec<T> {
        self.deque.iter().rev().cloned().collect()
    }
}

/// The CONFIG store stores contract configuration such as the admin address
pub const CONFIG: Item<Config> = Item::new("config");

/// The METRICS store stores the full history of a metric
/// It is key'd on the metric "key" field, but consists of a list (deque) of each metric sorted by update time
pub const METRICS: Map<&str, History<Metric>> = Map::new("metrics");

/// The REDEMPTION_RATES store is dedicated to redemption rate metrics
/// It is key'd on the stToken denom, and consists of a list (deque) of each redemption rate sorted by time
pub const REDEMPTION_RATES: Map<&str, History<RedemptionRate>> = Map::new("redemption_rates");

/// The PURCHASE_RATE store is dedicated to redemption rate metrics
/// It is key'd on the stToken denom, and consists of a list (deque) of each redemption rate sorted by time
pub const PURCHASE_RATES: Map<&str, History<PurchaseRate>> = Map::new("purchase_rates");

#[cfg(test)]
mod tests {
    use crate::state::{HasTime, History};
    use cosmwasm_schema::cw_serde;

    // Test item used to test the History deque
    #[cw_serde]
    pub struct DummyItem {
        pub value: u64,
        pub update_time: u64,
    }
    impl DummyItem {
        pub fn new(value: u64, update_time: u64) -> Self {
            DummyItem { value, update_time }
        }
    }
    impl HasTime for DummyItem {
        fn time(&self) -> u64 {
            self.update_time
        }
    }

    // Helper function to check the state/ordering of the deque
    fn check_deque_values(history: History<DummyItem>, expected: Vec<u64>) {
        let actual: Vec<u64> = history.deque.into_iter().map(|item| item.value).collect();
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_deque() {
        // Add 5 items, with the last item falling in the middle
        let mut history = History::<DummyItem>::new(5);
        history.add(DummyItem::new(100, 1)); // [(100,1)]
        history.add(DummyItem::new(200, 2)); // [(100,1) (200,2)
        history.add(DummyItem::new(300, 4)); // [(100,1) (200,2) (300,3)]
        history.add(DummyItem::new(400, 5)); // [(100,1) (200,2) (300,4) (400,5)]
        history.add(DummyItem::new(500, 3)); // [(100,1) (200,2) (500,3) (300,4) (400,5)]

        let expected = vec![100, 200, 500, 300, 400];
        check_deque_values(history.clone(), expected);

        // Add a new item that goes over capacity, it should kick out the oldest
        history.add(DummyItem::new(600, 6)); // [(200,2) (500,3) (300,4) (400,5) (600,6)]
        let expected = vec![200, 500, 300, 400, 600];
        check_deque_values(history.clone(), expected.clone());

        // Attempt to add an old item to the front, the deque should not change since it's at capacity
        history.add(DummyItem::new(700, 0));
        check_deque_values(history.clone(), expected);

        // Replace the oldest item
        history.add(DummyItem::new(800, 2)); // [(800,2) (500,3) (300,4) (400,5) (600,6)
        let expected = vec![800, 500, 300, 400, 600];
        check_deque_values(history.clone(), expected);

        // Test get_latest
        let expected: u64 = 600;
        assert_eq!(expected, history.get_latest().unwrap().value);

        // Test get_latest_range
        let expected: Vec<u64> = vec![600, 400, 300]; // list is flipped cause newest items are first
        let actual: Vec<u64> = history
            .get_latest_range(3)
            .iter()
            .map(|i| i.value)
            .collect();
        assert_eq!(expected, actual);

        // Test get_all
        let expected: Vec<u64> = vec![600, 400, 300, 500, 800]; // list is flipped cause newest items are first
        let actual: Vec<u64> = history.get_all().iter().map(|i| i.value).collect();
        assert_eq!(expected, actual);
    }
}
