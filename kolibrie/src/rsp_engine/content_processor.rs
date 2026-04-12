use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crossbeam::channel::Sender;
use log::{debug, error};
use prototypes::prototype::slide_strategy::ItemsReport;
use prototypes::WindowSnapshotStrategy;
use crate::rsp::r2r::R2ROperator;
use crate::rsp_engine::{QueryExecutionMode, WindowResult};
use crate::streamertail_optimizer::PhysicalOperator;

/// When there are joins and a window reports, you need to collect the results elsewhere
fn send_window_result<O: 'static>(
    results: &Vec<O>,
    window_iri: &str,
    ts: usize,
    window_result_sender: &Sender<WindowResult>,
) {
    // Convert the Vec<Vec<(String, String)> to Vec<HashMap<String,String>> to put in WindowResult
    let mut mapped_results: Vec<HashMap<String, String>> = Vec::new();
    mapped_results.reserve(results.len());

    for res in results {
        if let Some(bindings) =
            (res as &dyn std::any::Any).downcast_ref::<Vec<(String, String)>>()
        {
            let map: HashMap<String, String> = bindings.iter().cloned().collect();
            mapped_results.push(map);
        }
    }

    // The WindowResult accepts Vec<HashMap<String, String>>, not Vec<Vec<(String,String)>>
    let window_res = WindowResult {
        window_iri: window_iri.to_string(),
        results: mapped_results,
        timestamp: ts,
    };

    if let Err(e) = window_result_sender.send(window_res) {
        error!("Failed to send window result to buffer: {:?}", e);
    }
}

fn refresh_window_store<I, R, O>(
    report: &R,
    store: &mut dyn R2ROperator<I, Vec<PhysicalOperator>, O>,
    prev_window_triples: &mut Vec<I>,
) where
    R: ItemsReport<I>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
{

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
}

pub fn process_window_report<I, R, O>(
    report: R,
    window_iri: &str,
    query: &PhysicalOperator,
    query_execution_mode: &QueryExecutionMode,
    r2r_store: &Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
    prev_window_triples: &mut Vec<I>,
    window_result_sender: &Sender<WindowResult>,
    r2s_consumer_func: &Arc<dyn Fn(Vec<O>, usize) + Send + Sync>,
    has_joins: bool,
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

    refresh_window_store(&report, store.as_mut(), prev_window_triples);

    let results = store.execute_query(query);
    debug!("Got # results {} for window {}", results.len(), window_iri);

    drop(store);

    if has_joins {
        send_window_result(&results, window_iri, ts, window_result_sender);
    } else {
        // Immediately send to r2s_consumer
        r2s_consumer_func(results, ts);
    }
}