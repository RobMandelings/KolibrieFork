use std::collections::HashMap;
use std::sync::Arc;
use crate::rsp::builder::{AggregateConsumer, Consumer};
use crate::rsp::builder::Consumer::Aggregate;

/// Solution mapping: hashmap from key to value (= bindings)
pub type SolMap = HashMap<String, String>;
pub type MapAggregateConsumer = Arc<dyn Fn(Vec<SolMap>, usize) -> () + Send + Sync>;

/// Allows for easy input of Hashmap-based consumer, but makes it compatible with the existing engine output
/// (Which is simply Vec<(String, String)>
pub fn create_aggregate_consumer(
    consumer: MapAggregateConsumer,
) -> Consumer<Vec<(String, String)>> {
    let consumer_fn = Arc::new(move |results: Vec<Vec<(String, String)>>, ts| {
        let converted: Vec<HashMap<String, String>> = results
            .into_iter()
            .map(|row| row.into_iter().collect::<HashMap<String, String>>())
            .collect();

        consumer(converted, ts);
    });

    Aggregate(consumer_fn)
}