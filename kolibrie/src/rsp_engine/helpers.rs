use std::collections::HashMap;
use std::sync::Arc;
use log::LevelFilter;
use crate::rsp::builder::{AggregateConsumer, Consumer};
use crate::rsp::builder::Consumer::Aggregate;

pub fn init_logger(level_filter: LevelFilter) {
    use env_logger::Builder;
    use log::LevelFilter;

    let mut builder = Builder::from_default_env();
    builder
        .is_test(true)
        .filter_level(level_filter) // or Info/Warn/Error
        .init();
}

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