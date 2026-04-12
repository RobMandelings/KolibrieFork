use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use crossbeam::channel::{RecvTimeoutError, Sender};
use log::{debug, error};
use prototypes::WindowSnapshotStrategy;
use shared::query::{Fallback, SyncPolicy};
use crate::rsp::builder::Consumer;
use crate::rsp::r2r::R2ROperator;
use crate::rsp::s2r::{CSPARQLWindow, ContentContainer, Report};
use crate::rsp_engine::{emit_results, legacy_engine, register_processor_for_window, OperationMode, QueryExecutionMode, RSPEngine, RSPWindow, WindowResult};
use crate::streamertail_optimizer::PhysicalOperator;

/// Creates a closure that receives the content of a window and processes the content through the pipeline
/// After processing, the solutions are sent to the last stage of the pipeline: R2S
pub fn create_window_content_processor<I, O>(
    window_iri: String,
    query: PhysicalOperator,
    query_execution_mode: QueryExecutionMode,
    r2r_store: Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
    has_joins: bool,
    window_result_sender: Sender<WindowResult>,
    r2s_consumer_func: Arc<dyn Fn(Vec<O>, usize) + Send + Sync>, // Consumes the solution mappings to put it onto the output stream
) -> impl FnMut(ContentContainer<I>) + Send + 'static
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    O: Send + 'static,
{
    // You use these to decide which triples to evict from the store
    // The R2R store maintains the current state of triples, so you need to know which ones to remove
    let mut prev_window_triples: Vec<I> = Vec::new();

    // The processor receives the window content from the sliding window
    move |content: ContentContainer<I>| {
        debug!(
            "Processing window {} with query: {:?} using {:?} execution",
            window_iri, query, query_execution_mode
        );

        // First step is to update the store to reflect the last correct changes
        let ts = content.get_last_timestamp_changed();

        // Store is a boxed trait object implementing R2R Operator (Box<dyn R2ROperator>)
        let mut store = r2r_store.lock().unwrap();

        // Evict triples from the previous firing of this window
        for t in &prev_window_triples {
            store.remove(t);
        }
        prev_window_triples.clear();

        // Add current window triples and track them for next eviction
        for t in content.into_iter() {
            prev_window_triples.push(t.clone());
            store.add(t);
        }

        // Run forward-chaining inference to materialise derived facts
        store.materialize();

        // Run the query on the R2R and get solution mappings
        let results = store.execute_query(&query);
        debug!("Got # results {} for window {}", results.len(), window_iri);

        // Release lock early to reduce contention
        drop(store);

        if has_joins {
            // Convert the Vec<Vec<(String, String)> to Vec<HashMap<String,String>> to put in WindowResult
            let mut mapped_results: Vec<HashMap<String, String>> = Vec::new();
            mapped_results.reserve(results.len());

            for res in &results {
                if let Some(bindings) =
                    (res as &dyn std::any::Any).downcast_ref::<Vec<(String, String)>>()
                {
                    let map: HashMap<String, String> = bindings.iter().cloned().collect();
                    mapped_results.push(map);
                }
            }

            // The WindowResult accepts Vec<HashMap<String, String>>, not Vec<Vec<(String,String)>>
            let window_res = WindowResult {
                window_iri: window_iri.clone(),
                results: mapped_results,
                timestamp: ts,
            };

            if let Err(e) = window_result_sender.send(window_res) {
                error!("Failed to send window result to buffer: {:?}", e);
            }
        } else {
            r2s_consumer_func(results, ts);
        }
    }
}

/// All functinality related to legacy window implementations go here
impl<I, O, S> RSPEngine<I, O, S>
where
    O: Clone + Hash + Eq + Send + 'static + From<Vec<(String, String)>>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    S: WindowSnapshotStrategy<I>,
{

    pub fn create_legacy_windows(configs: &Vec<RSPWindow>) -> Vec<CSPARQLWindow<I>> {
        let mut windows = Vec::new();
        for window_config in configs {
            let mut report = Report::new();
            report.add(window_config.report_strategy.clone());
            let window = CSPARQLWindow::new(
                window_config.width,
                window_config.slide,
                report,
                window_config.tick.clone(),
                window_config.window_iri.clone(),
            );
            windows.push(window);
        }
        windows
    }

    /// Register windows using macros to eliminate code duplication
    pub fn register_legacy_windows(&mut self, operation_mode: OperationMode) {

        // First: collect all processors per window_idx
        let mut to_register = Vec::new();

        for (window_idx, _) in self.windows.iter().enumerate() {
            let query = self.rsp_query_plan.window_plans[window_idx].clone();
            let window_iri = self.window_configs[window_idx].window_iri.clone();

            /// In my API this is called the consumer
            let content_processor = self.create_legacy_window_processor(
                window_iri.clone(),
                query,
                self.query_execution_mode,
                self.r2r.clone(),
                self.window_result_sender.clone(),
                self.r2s_consumer.clone(),
            );

            to_register.push((window_idx, window_iri, content_processor));
        }


        // Second: mutably register all processors on the windows
        for (window_idx, window_iri, content_processor) in to_register {
            let window = &mut self.windows[window_idx];

            // Window IRI is moved inside closure of thread, therefore you clone it
            register_processor_for_window(
                operation_mode,
                window,
                content_processor,
                window_iri.clone(),
            );
        }
    }

    pub fn process_single_thread_window_results(&mut self)
    where
        O: From<Vec<(String, String)>>,
    {
        // The consumer function that will be called. We clone to get ownership, but it points to the same function
        let consumer = self.r2s_consumer.clone();
        let num_windows = self.windows.len();
        let sync_policy = self.sync_policy.clone();

        // Drain all pending channel results; update last_materialized with replace semantics.
        let mut last_mat = self.single_thread_last_materialized.lock().unwrap();
        let mut had_new_results = false;
        let mut max_ts: usize = 0;

        // Non-blocking drain of the channel: receive all results
        while let Ok(window_result) = self.window_result_receiver.try_recv() {
            max_ts = max_ts.max(window_result.timestamp);
            last_mat.insert(window_result.window_iri.clone(), window_result.results);
            had_new_results = true;
        }

        if !had_new_results {
            return;
        }

        // Check whether to emit based on policy.
        // Only emit when all windows have been materialized
        if last_mat.len() == num_windows {
            debug!(
                "SingleThread: all {} windows materialized, emitting",
                num_windows
            );
            let static_data_plan = self.rsp_query_plan.static_data_plan.clone();
            emit_results(
                &*last_mat,
                &static_data_plan,
                &self.static_db,
                &self.r2s_operator,
                max_ts,
                &consumer,
            );

            match sync_policy {
                // Wait: require all windows to fire again before next emission.
                // Timeout: no wall-clock timer in single-threaded context; treat as Wait.
                SyncPolicy::Wait | SyncPolicy::Timeout { .. } => {
                    last_mat.clear();
                }
                // Steal: keep last_mat so stale data from non-firing windows is reused.
                SyncPolicy::Steal => {}
            }
        } else {
            debug!(
                "SingleThread: waiting for more windows ({}/{})",
                last_mat.len(),
                num_windows
            );
        }
    }

    /// Legacy method for backward compatibility
    pub fn legacy_add(&mut self, event_item: I, ts: usize) {
        // Add to all windows (for backward compatibility)
        for window in &mut self.windows {
            window.add_to_window(event_item.clone(), ts);
        }
    }

    pub fn stop(&mut self) {
        for window in &mut self.windows {
            window.flush();
            window.stop();
        }
        if matches!(self.operation_mode, OperationMode::SingleThread) {
            self.process_single_thread_window_results();
        }
    }

    /// Start a coordinator thread that collects and joins results from multiple windows
    /// (and optionally joins with static background data), respecting `sync_policy`.
    pub(crate) fn start_cross_window_coordinator(&self)
    where
        O: From<Vec<(String, String)>>,
    {
        let receiver = self.window_result_receiver.clone();
        let consumer = self.r2s_consumer.clone();
        let num_windows = self.windows.len();
        let static_data_plan = self.rsp_query_plan.static_data_plan.clone();
        let static_db = self.static_db.clone();
        let sync_policy = self.sync_policy.clone();
        let r2s_operator = Arc::clone(&self.r2s_operator);

        thread::spawn(move || {
            // Latest results per window (replace semantics)
            let mut last_materialized: HashMap<String, Vec<HashMap<String, String>>> =
                HashMap::new();
            // Windows that have fired since the last reset
            let mut cycle_triggered: HashSet<String> = HashSet::new();
            // When the first window fired in the current cycle
            let mut cycle_start: Option<Instant> = None;
            let mut max_ts: usize = 0;

            loop {
                // Compute recv timeout when policy has a finite deadline
                let timeout_remaining = match &sync_policy {
                    SyncPolicy::Timeout { duration, .. } => {
                        cycle_start.map(|start| duration.saturating_sub(start.elapsed()))
                    }
                    _ => None,
                };

                // Receive next window result (or timeout/disconnect)
                let maybe_result: Option<WindowResult> = if let Some(remaining) = timeout_remaining
                {
                    match receiver.recv_timeout(remaining) {
                        Ok(r) => Some(r),
                        Err(RecvTimeoutError::Timeout) => {
                            // Deadline elapsed
                            if !cycle_triggered.is_empty() {
                                match &sync_policy {
                                    SyncPolicy::Timeout {
                                        fallback: Fallback::Steal,
                                        ..
                                    } => {
                                        if last_materialized.len() == num_windows {
                                            emit_results(
                                                &last_materialized,
                                                &static_data_plan,
                                                &static_db,
                                                &r2s_operator,
                                                max_ts,
                                                &consumer,
                                            );
                                        }
                                    }
                                    SyncPolicy::Timeout {
                                        fallback: Fallback::Drop,
                                        ..
                                    } => {
                                        // discard this cycle
                                    }
                                    _ => {}
                                }
                                cycle_triggered.clear();
                                cycle_start = None;
                                max_ts = 0;
                            }
                            continue;
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                } else {
                    match receiver.recv() {
                        Ok(r) => Some(r),
                        Err(_) => break,
                    }
                };

                if let Some(window_result) = maybe_result {
                    debug!(
                        "Coordinator received {} results from window: {}",
                        window_result.results.len(),
                        window_result.window_iri
                    );

                    max_ts = max_ts.max(window_result.timestamp);
                    // Update last_materialized (replace)
                    last_materialized.insert(
                        window_result.window_iri.clone(),
                        window_result.results.clone(),
                    );
                    if cycle_triggered.is_empty() {
                        cycle_start = Some(Instant::now());
                    }
                    cycle_triggered.insert(window_result.window_iri.clone());

                    // Drain any additional pending results
                    while let Ok(wr) = receiver.try_recv() {
                        max_ts = max_ts.max(wr.timestamp);
                        last_materialized.insert(wr.window_iri.clone(), wr.results.clone());
                        cycle_triggered.insert(wr.window_iri.clone());
                    }

                    if cycle_triggered.len() == num_windows {
                        // All windows fired this cycle
                        emit_results(
                            &last_materialized,
                            &static_data_plan,
                            &static_db,
                            &r2s_operator,
                            max_ts,
                            &consumer,
                        );
                        cycle_triggered.clear();
                        cycle_start = None;
                        max_ts = 0;
                    } else {
                        match &sync_policy {
                            SyncPolicy::Steal => {
                                // Emit immediately using stale data from non-firing windows
                                if last_materialized.len() == num_windows {
                                    emit_results(
                                        &last_materialized,
                                        &static_data_plan,
                                        &static_db,
                                        &r2s_operator,
                                        max_ts,
                                        &consumer,
                                    );
                                }
                                cycle_triggered.clear();
                                cycle_start = None;
                                max_ts = 0;
                            }
                            SyncPolicy::Wait | SyncPolicy::Timeout { .. } => {
                                // Keep waiting for remaining windows
                                debug!(
                                    "Coordinator: waiting for more windows ({}/{}) — have: {:?}",
                                    cycle_triggered.len(),
                                    num_windows,
                                    cycle_triggered.iter().collect::<Vec<_>>()
                                );
                            }
                        }
                    }
                }
            }

            debug!("Coordinator: shutdown complete");
        });
    }
}
