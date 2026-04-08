use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crossbeam::channel::Sender;
use log::debug;
use prototypes::prototype::slide_strategy::ItemsReport;
use prototypes::prototype::slide_strategy::iter_expire_strategy::IterExpireContainer;
use prototypes::WindowSnapshotStrategy;
use crate::rsp::r2r::R2ROperator;
use crate::rsp_engine::{QueryExecutionMode, WindowResult};
use crate::streamertail_optimizer::PhysicalOperator;

pub fn process_window_report<I, R, O>(
    report: R,
    window_iri: &str,
    query: &PhysicalOperator,
    query_execution_mode: &QueryExecutionMode,
    r2r_store: &Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
    prev_window_triples: &mut Vec<I>,
    window_result_sender: &Sender<WindowResult>,
    r2s_consumer_func: &Arc<dyn Fn(Vec<O>, usize) + Send + Sync>,
) where
    R: ItemsReport<I>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    O: Send + 'static,
{
    debug!(
        "Processing window {} with query: {:?} using {:?} execution",
        window_iri, query, query_execution_mode
    );

    let ts = report.get_last_timestamp_changed() as usize;

    let mut store = r2r_store.lock().unwrap();

    // Evict previous triples
    for t in &*prev_window_triples {
        store.remove(t);
    }
    prev_window_triples.clear();

    // Add current triples
    for t in report.iter_items() {
        prev_window_triples.push(t.clone());
        store.add(t.clone());
    }

    store.materialize();

    let results = store.execute_query(query);
    debug!("Got # results {} for window {}", results.len(), window_iri);

    drop(store);
    r2s_consumer_func(results, ts);
}


/// Creates a closure that receives the content of a window and processes the content through the pipeline
/// After processing, the solutions are sent to the last stage of the pipeline: R2S
/// R stands for 'report'
pub fn create_iter_expire_window_processor<I, O, S>(
    strategy: S,
    window_iri: String,
    query: PhysicalOperator,
    query_execution_mode: QueryExecutionMode,
    r2r_store: Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
    window_result_sender: Sender<WindowResult>,
    r2s_consumer_func: Arc<dyn Fn(Vec<O>, usize) + Send + Sync>, // Consumes the solution mappings to put it onto the output stream
) -> impl FnMut(S::ReportType<'_>) + Send + 'static
where
    S: WindowSnapshotStrategy<I>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    O: Send + 'static,
{
    // You use these to decide which triples to evict from the store
    // The R2R store maintains the current state of triples, so you need to know which ones to remove
    let mut prev_window_triples: Vec<I> = Vec::new();

    // The processor receives the window content from the sliding window
    move |report: S::ReportType<'_>| {
        process_window_report(
            report,
            &window_iri,
            &query,
            &query_execution_mode,
            &r2r_store,
            &mut prev_window_triples,
            &window_result_sender,
            &r2s_consumer_func,
        );
    }
}