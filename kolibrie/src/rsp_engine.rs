/*
* Copyright © 2025 Volodymyr Kadzhaia
* Copyright © 2025 Pieter Bonte
* KU Leuven — Stream Intelligence Lab, Belgium
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* you can obtain one at [https://mozilla.org/MPL/2.0/](https://mozilla.org/MPL/2.0/).
*/
mod content_processor;
pub mod helpers;
mod city_bench_observations_test;
mod city_bench_q1;
mod csv_graph_iter2;
mod parking_mapper;
mod legacy_engine;

use crate::rsp::r2r::R2ROperator;
use crate::rsp::r2s::Relation2StreamOperator;
use crate::rsp::s2r::{CSPARQLWindow, ContentContainer, Report, ReportStrategy, Tick, Window};

use crate::parser::process_rule_definition;
use crate::sparql_database::SparqlDatabase;
use crate::streamertail_optimizer::{ExecutionEngine, LogicalOperator, PhysicalOperator};
use crossbeam::channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use datalog::reasoning::Reasoner;
#[cfg(not(test))]
use log::{debug, error}; // Use log crate when building application
use prototypes::prototype::event::Time;
// use prototypes::prototype::slide_strategy::slice_expire_strategy::{SliceExpireStrategy, ContReport};
use prototypes::prototype::slide_strategy::iter_expire_strategy::{
    IterConsumer, IterExpireContainer, IterExpireStrategy, IterReport,
};
use prototypes::prototype::slide_strategy::ItemsReport;
use prototypes::prototype::window_params::S2RWindowConfig;
use prototypes::{
    ExpireStrategy, SlidingWindowOperator, WindowParams, WindowSnapshotStrategy, IRI,
};
use shared::query::{Fallback, SyncPolicy};
use shared::rule::Rule;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
#[cfg(test)]
use std::{println as debug, println as error};
// Re-exports to preserve the public API used by kolibrie-http-server and examples.
use crate::rsp::builder::{AggregateConsumer, Consumer, SingleConsumer};
pub use crate::rsp::builder::{RSPBuilder, RSPQueryConfig};
use crate::rsp::builder::Consumer::Aggregate;
pub use crate::rsp::simple_r2r::SimpleR2R;
use crate::rsp_engine::content_processor::{process_window_report};
use crate::sliding_window::SlidingWindow;

/// For compatibility with the existing architecture: map (stream_iri, window_iri) to the original index of that specific window
/// So that you can find the original query plans etc for that specific window
type WindowMapping = HashMap<(IRI, IRI), usize>;

#[derive(Clone, Copy)]
pub enum OperationMode {
    SingleThread,
    MultiThread,
}

#[derive(Clone, Copy, Debug)]
pub enum QueryExecutionMode {
    Standard,
    Volcano,
}

/// Window configuration extracted from parsed RSP-QL query
#[derive(Debug, Clone)]
pub struct RSPWindow {
    pub window_iri: String,
    pub stream_iri: String,
    pub width: usize,
    pub slide: usize,
    pub tick: Tick,
    pub report_strategy: ReportStrategy,
    pub query: LogicalOperator, // The SPARQL query to execute on this window
}

/// RSP-QL Query Plan using Volcano optimizer
#[derive(Debug, Clone)]
pub struct RSPQueryPlan {
    pub window_plans: Vec<PhysicalOperator>,
    pub static_data_plan: Option<PhysicalOperator>,
}

/// Result from a single window execution
#[derive(Debug, Clone)]
pub struct WindowResult {
    pub window_iri: String,
    pub results: Vec<HashMap<String, String>>, // Variable bindings
    pub timestamp: usize,
}

/// Whether results from windows should be combined or not (combining the solution mappings)
fn has_joins(windows: &Vec<RSPWindow>, rsp_query_plan: &RSPQueryPlan) -> bool {
    let has_joins = windows.len() > 1 || rsp_query_plan.static_data_plan.is_some();
    has_joins
}


/// This registers the processor of the window content so that the window knows how to send consumer.
fn register_processor_for_window2<I, P>(
    operation_mode: OperationMode,
    window: &mut CSPARQLWindow<I>,
    mut processor: P,
    window_iri: String,
) where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    P: FnMut(ContentContainer<I>) + Send + 'static,
{
    match operation_mode {
        OperationMode::SingleThread => {
            window.register_callback(Box::new(processor));
        }
        OperationMode::MultiThread => {
            unimplemented!("Not implemented!");
        }
    }
}

/// This registers the processor of the window content so that the window knows how to send consumer.
fn register_processor_for_window<I, P>(
    operation_mode: OperationMode,
    window: &mut CSPARQLWindow<I>,
    mut processor: P,
    window_iri: String,
) where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    P: FnMut(ContentContainer<I>) + Send + 'static,
{
    match operation_mode {
        OperationMode::SingleThread => {
            window.register_callback(Box::new(processor));
        }
        OperationMode::MultiThread => {
            let receiver = window.register_channel();

            thread::spawn(move || {
                loop {
                    match receiver.recv() {
                        Ok(content) => {
                            processor(content);
                        }
                        Err(_) => {
                            debug!("Shutting down window {}!", window_iri);
                            break;
                        }
                    }
                }
                debug!("Shutdown complete for window {}!", window_iri);
            });
        }
    }
}

/// RSP input with generic input type I and output type O
pub struct RSPEngine<I, O, S>
where
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    O: Hash,
    S: WindowSnapshotStrategy<I>,
{
    legacy_window: bool, // Whether or not to use the legacy CSPARQL window implementation or not
    window_mapping: WindowMapping, // Helper mapping that maps the window_idx (from the previous implementation) to (stream_iri, window_iri) pair
    custom_windows: HashMap<IRI, SlidingWindowOperator<I, S>>,
    windows: Vec<CSPARQLWindow<I>>,
    r2r: Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
    r2s_consumer: AggregateConsumer<O>,
    window_configs: Vec<RSPWindow>,
    query_execution_mode: QueryExecutionMode,
    operation_mode: OperationMode,
    // Channel for collecting window results for cross-window joins
    window_result_sender: Sender<WindowResult>,
    window_result_receiver: Receiver<WindowResult>,
    // RSP-QL Query Plan using Volcano optimizer
    rsp_query_plan: RSPQueryPlan,
    /// Latest materialized results per window (replace semantics); SingleThread only.
    single_thread_last_materialized: Arc<Mutex<HashMap<String, Vec<HashMap<String, String>>>>>,
    /// Synchronization policy governing multi-window coordination.
    sync_policy: SyncPolicy,
    /// Separate store for static background triples (never touched by window processors).
    static_db: Arc<Mutex<SparqlDatabase>>,
    /// R2S operator for stream-type filtering (RSTREAM/ISTREAM/DSTREAM).
    r2s_operator: Arc<Mutex<Relation2StreamOperator<O>>>,
}

impl<I, O, S> RSPEngine<I, O, S>
where
    O: Clone + Hash + Eq + Send + 'static + From<Vec<(String, String)>>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    S: WindowSnapshotStrategy<I>,
{
    pub fn new(
        legacy_window: bool,
        query_config: RSPQueryConfig,
        triples: &str,
        syntax: String,
        rules: &str,
        r2s_consumer_undecorated: Consumer<O>,
        r2r: Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>,
        operation_mode: OperationMode,
        query_execution_mode: QueryExecutionMode,
        rsp_query_plan: RSPQueryPlan,
        sync_policy: SyncPolicy,
        reasoning_rules: Vec<Rule>,
        sparql_rules: Vec<String>,
    ) -> RSPEngine<I, O, S> {
        // Not only an operator but also stores the triples inside for e.g. executing queries
        let mut store = r2r;

        // The PhysicalOperator plans created in `rsp_query_plan` contain integer IDs (constants)
        // that were generated by the Dictionary in `query_config.database`.
        // The `store` (R2R operator) has its own Dictionary. If we don't sync them,
        // the store will assign different IDs to incoming data, and the execution engine
        // will fail to match them against the plan.
        let shared_dict = store
            .as_any_mut()
            .downcast_mut::<SimpleR2R>()
            .map(|s| Arc::clone(&s.item.dictionary));

        // Store exposes as_any_mut() -> Any, need to downcast to get SimpleR2R
        if let Some(simple_r2r) = store.as_any_mut().downcast_mut::<SimpleR2R>() {
            debug!("Synchronizing R2R dictionary with Query dictionary");

            // Acquire locks on both dictionaries, then merge the query_dict into store_dict
            let mut store_dict = simple_r2r.item.dictionary.write().unwrap();
            let query_dict = query_config.database.dictionary.read().unwrap();

            // The store dict is the same thing as the shared_dict (to which the static_sdb dictionary gets assigned to)
            store_dict.merge(&*query_dict);

            drop(store_dict);
            drop(query_dict);
        }

        // Build the static-data store sharing the same dictionary as the R2R store.
        let mut static_sdb = SparqlDatabase::new();
        if let Some(d) = shared_dict {
            static_sdb.dictionary = d;
        }
        let static_db = Arc::new(Mutex::new(static_sdb));

        // Load initial triples into the R2R store
        match store.load_triples(triples, syntax) {
            Err(parsing_error) => error!("Unable to load ABox: {:?}", parsing_error.to_string()),
            _ => (),
        }

        match store.load_rules(rules) {
            Ok(_) => debug!("Rules loaded successfully"),
            Err(e) => error!("Failed to load rules: {:?}", e),
        }

        if !reasoning_rules.is_empty() {
            if let Some(simple_r2r) = store.as_any_mut().downcast_mut::<SimpleR2R>() {
                simple_r2r.add_reasoning_rules(reasoning_rules);
            }
        }

        if !sparql_rules.is_empty() {
            if let Some(dict) = store
                .as_any_mut()
                .downcast_mut::<SimpleR2R>()
                .map(|s| Arc::clone(&s.item.dictionary))
            {
                for rule_str in &sparql_rules {
                    let mut temp_db = SparqlDatabase::new();
                    temp_db.dictionary = Arc::clone(&dict);
                    match process_rule_definition(rule_str, &mut temp_db) {
                        Ok((rule, _)) => {
                            if let Some(simple_r2r) = store.as_any_mut().downcast_mut::<SimpleR2R>()
                            {
                                simple_r2r.rules.push(rule);
                            }
                        }
                        Err(e) => error!("Failed to parse SPARQL rule: {:?}", e),
                    }
                }
            }
        }

        let legacy_windows = Self::create_legacy_windows(&query_config.windows);
        let custom_windows = Self::create_windows(&query_config.windows);

        let window_mapping = Self::build_window_mapping(&query_config.windows);

        // Create channel for cross-window result coordination
        // Unbounded so unlimited capacity (might run into memory issues)
        let (result_sender, result_receiver) = unbounded::<WindowResult>();

        let stream_type = query_config.stream_type.clone();
        let r2s_operator = Arc::new(Mutex::new(Relation2StreamOperator::new(stream_type, 0)));

        let has_joins = has_joins(&query_config.windows, &rsp_query_plan);
        let r2s_consumer = Self::decorate_consumer_with_r2s_operator(&r2s_consumer_undecorated, has_joins, &r2s_operator);

        let mut engine = RSPEngine {
            legacy_window,
            window_mapping,
            custom_windows,
            windows: legacy_windows,
            r2r: Arc::new(Mutex::new(store)),
            r2s_consumer,
            window_configs: query_config.windows.clone(),
            query_execution_mode,
            operation_mode,
            window_result_sender: result_sender,
            window_result_receiver: result_receiver,
            rsp_query_plan,
            single_thread_last_materialized: Arc::new(Mutex::new(HashMap::new())),
            sync_policy,
            static_db,
            r2s_operator,
        };

        engine.register_custom_windows();

        match operation_mode {
            mode @ (OperationMode::SingleThread | OperationMode::MultiThread) => {
                engine.register_legacy_windows(mode);
                if matches!(mode, OperationMode::MultiThread) {
                    let has_joins = engine.windows.len() > 1
                        || engine.rsp_query_plan.static_data_plan.is_some();
                    if has_joins {
                        engine.start_cross_window_coordinator();
                    }
                }
            }
        }

        engine
    }

    fn group_by_stream_iri(windows: &Vec<RSPWindow>) -> HashMap<String, Vec<&RSPWindow>> {
        let mut groups: HashMap<String, Vec<&RSPWindow>> = HashMap::new();

        for w in windows {
            groups
                .entry(w.stream_iri.clone())
                .or_insert_with(Vec::new)
                .push(w);
        }
        groups
    }

    fn build_window_mapping(window_configs: &Vec<RSPWindow>) -> WindowMapping {
        let mut mapping: WindowMapping = HashMap::new();

        for (idx, w) in window_configs.iter().enumerate() {
            if mapping
                .insert((w.stream_iri.clone(), w.window_iri.clone()), idx)
                .is_some()
            {
                panic!(
                    "Duplicate (stream_iri, window_iri) pair detected: ({:?}, {:?})",
                    w.window_iri, w.window_iri
                );
            }
        }

        mapping
    }

    fn create_windows(
        window_configs: &Vec<RSPWindow>,
    ) -> (HashMap<IRI, SlidingWindowOperator<I, S>>) {
        let mut ops = HashMap::new();

        let grouped = Self::group_by_stream_iri(window_configs);
        for (stream_iri, windows) in grouped {
            let window_params: Vec<S2RWindowConfig> = windows
                .iter()
                .map(|window| S2RWindowConfig {
                    window_iri: window.window_iri.clone(),
                    window_params: WindowParams {
                        size: window.width as Time,
                        slide: window.slide as Time,
                        offset: 0,
                    },
                })
                .collect();

            let op: SlidingWindowOperator<I, S> =
                SlidingWindowOperator::new(stream_iri.clone(), window_params, S::new());

            ops.insert(stream_iri, op);
        }
        ops
    }

    /// Whether results from windows should be combined or not (combining the solution mappings)
    fn has_joins(&self) -> bool {
        let has_joins = self.windows.len() > 1 || self.rsp_query_plan.static_data_plan.is_some();
        has_joins
    }

    pub fn create_legacy_window_processor(
        &self,
        window_iri: String,
        query: PhysicalOperator,
        query_execution_mode: QueryExecutionMode,
        r2r_store: Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
        window_result_sender: Sender<WindowResult>,
        r2s_consumer_func: AggregateConsumer<O>, // Consumes the solution mappings to put it onto the output stream
    ) -> impl FnMut(ContentContainer<I>) + Send + 'static {
        // You use these to decide which triples to evict from the store
        // The R2R store maintains the current state of triples, so you need to know which ones to remove
        let mut prev_window_triples: Vec<I> = Vec::new();

        // The processor receives the window content from the sliding window
        let has_joins = self.has_joins();
        move |report: ContentContainer<I>| {
            process_window_report(
                report,
                &window_iri,
                &query,
                &query_execution_mode,
                &r2r_store,
                &mut prev_window_triples,
                &window_result_sender,
                &r2s_consumer_func,
                has_joins
            );
        }
    }

    /// Creates a closure that receives the content of a window and processes the content through the pipeline
    /// After processing, the solutions are sent to the last stage of the pipeline: R2S
    /// R stands for 'report'
    pub fn create_custom_window_processor(
        &self,
        window_iri: String,
        query: PhysicalOperator,
        query_execution_mode: QueryExecutionMode,
        r2r_store: Arc<Mutex<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>>,
        window_result_sender: Sender<WindowResult>,
        r2s_consumer_func: AggregateConsumer<O>, // Consumes the solution mappings to put it onto the output stream
    ) -> impl FnMut(S::ReportType<'_>) + Send + 'static {
        // You use these to decide which triples to evict from the store
        // The R2R store maintains the current state of triples, so you need to know which ones to remove
        let mut prev_window_triples: Vec<I> = Vec::new();

        // The processor receives the window content from the sliding window
        let has_joins = self.has_joins();
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
                has_joins
            );
        }
    }

    // let (stream_iri, window_iri) = &self.window_mapping[window_idx];
    // This function is called to actually output the emitted results back into the
    // Consumer function that was provided as parameter to the RSPEngine
    /// A single consumer was provided but the window always reports aggregates (all window content at once)
    fn decorate_consumer_with_r2s_operator(
        consumer: &Consumer<O>,
        has_joins: bool,
        r2s_operator: &Arc<Mutex<Relation2StreamOperator<O>>>
    ) -> AggregateConsumer<O> {

        let consumer_fn = consumer.clone();

        // Takes ALL solution mappings along with their timestamp
        // Then runs consumer_fn for each of the solution mappings
        let r2s_aggregate_consumer: Arc<dyn Fn(Vec<O>, usize) + Send + Sync> = if has_joins {
            // emit_results is responsible for checking this at the moment. So you can simply bypass now.
            // TODO do the check HERE! Makes much more sense.
            Arc::new(move |results: Vec<O>, ts: usize| {
                match &consumer_fn {
                    Consumer::Single(f) => {
                        for r in results {
                            f(r)
                        }
                    } Aggregate(f) => {
                        f(results, ts);
                    }
                }
            })
        } else {
            let r2s_op = Arc::clone(&r2s_operator);
            Arc::new(move |results: Vec<O>, ts: usize| {
                let filtered = r2s_op.lock().unwrap().eval(results, ts);
                match &consumer_fn {
                    Consumer::Single(f) => {
                        for r in filtered {
                            // For each solution mapping in the set of solution mappings, call the consumer function
                            f(r)
                        }
                    } Aggregate(f) => {
                        f(filtered, ts)
                    }
                }
            })
        };

        r2s_aggregate_consumer
    }

    /// Register windows using macros to eliminate code duplication
    fn register_custom_windows(&mut self) {
        // First: collect all processors per (stream_iri, window_iri)
        let mut to_register: Vec<(IRI, Vec<(IRI, Box<dyn for<'a> FnMut(S::ReportType<'a>)>)>)> =
            Vec::new();

        for (stream_iri, s2r_operator) in &self.custom_windows {
            let mut consumers_by_window_iri: Vec<(IRI, Box<dyn for<'a> FnMut(S::ReportType<'a>)>)> =
                Vec::new();

            for window_iri in s2r_operator.sliding_windows.keys() {
                let idx = *self
                    .window_mapping
                    .get(&(stream_iri.clone(), window_iri.clone()))
                    .expect("Mapping should exist");

                // Generalise to aggregate consumer because that is what the window reports
                // (Aggregates are passed as arguments always)
                let content_processor = Box::new(self.create_custom_window_processor(
                    window_iri.clone(),
                    self.rsp_query_plan.window_plans[idx].clone(),
                    self.query_execution_mode,
                    self.r2r.clone(),
                    self.window_result_sender.clone(),
                    self.r2s_consumer.clone(),
                ));

                consumers_by_window_iri.push((window_iri.clone(), content_processor));
            }

            to_register.push((stream_iri.clone(), consumers_by_window_iri));
        }

        // Second: look up the right s2r_operator mutably and add all consumers
        for (stream_iri, consumers_by_window_iri) in to_register {
            if let Some(s2r_operator) = self.custom_windows.get_mut(&stream_iri) {
                for (iri, processor) in consumers_by_window_iri {
                    s2r_operator.add_consumer(&iri, processor);
                }
            } else {
                // optional: debug_assert! or panic if you expect it to always exist
                // panic!("s2r_operator for stream {stream_iri} disappeared");
            }
        }
    }

    pub fn decode(&self, input: &I) -> String {
        let guard = self.r2r.lock().expect("mutex poisoned");
        guard.decode(input)
    }

    fn normalize_stream_iri(s: &str) -> String {
        let s = s.trim();
        // Some callers might pass a full IRI in `<...>` form.
        let s = s.trim_start_matches('<').trim_end_matches('>');
        // Accept prefixed notation with an optional leading colon, e.g. `:stream1`.
        let s = s.strip_prefix(':').unwrap_or(s);
        s.to_string()
    }

    /// Item arrives on all streams. Matches on legacy window to decide which type of window to add the event to (CSPARQLWindow or S2ROperator)
    pub fn custom_add(&mut self, event_item: I, ts: usize) {
        if self.legacy_window {
            self.legacy_add(event_item, ts);
        } else {
            for s2r in self.custom_windows.values_mut() {
                s2r.event_arrives_with_ts(event_item.clone(), ts as Time);
            }
        }
    }

    /// Add data to appropriate window based on stream IRI
    pub fn legacy_add_to_stream(&mut self, stream_iri: &str, event_item: I, ts: usize) {

        let input_norm = Self::normalize_stream_iri(stream_iri);

        // Find windows that match this stream IRI and add the event to these windows
        for (window_idx, window_config) in self.window_configs.iter().enumerate() {
            // Variable stream (e.g. `?s`) matches any stream.
            if window_config.stream_iri.starts_with('?') {
                if let Some(window) = self.windows.get_mut(window_idx) {
                    window.add_to_window(event_item.clone(), ts);
                }
                continue;
            }

            let cfg_norm = Self::normalize_stream_iri(&window_config.stream_iri);
            if cfg_norm == input_norm {
                if let Some(window) = self.windows.get_mut(window_idx) {
                    window.add_to_window(event_item.clone(), ts);
                }
            }
        }
    }


    /// Item arrives on specific stream. Matches on legacy window to decide which type of window to add the event to
    pub fn custom_add_to_stream(&mut self, stream_iri: &str, event_item: I, ts: Time) {
        if matches!(self.operation_mode, OperationMode::SingleThread)
            && self.has_joins()
        {
            self.process_single_thread_window_results();
        }

        if self.legacy_window {
            self.legacy_add_to_stream(stream_iri, event_item, ts as usize);
        } else {
            self.custom_windows
                .get_mut(stream_iri)
                .expect(&format!("Could not find window for stream IRI: {stream_iri}"))
                .event_arrives_with_ts(event_item, ts);
        }
    }

    pub fn add_graph_to_stream(&mut self,
                           stream_iri: &str,
                           graph_str: &str,
                           ts: Time,
    )
    {
        for triple in self.parse_data(graph_str) {
            self.custom_add_to_stream(stream_iri, triple, ts);
        }
    }

    /// Parses the data and returns a Vec of input elements (whatever you said would be the input)
    pub fn parse_data(&self, data: &str) -> Vec<I> {
        self.r2r.lock().unwrap().parse_data(data)
    }

    /// Pre-populate the static background store with N-Triples data.
    /// These triples are never placed in the window R2R store, so they cannot
    /// leak into window query results.  They are only visible when `emit_results`
    /// joins the window output with the static-data plan.
    pub fn add_static_ntriples(&mut self, data: &str) {
        let mut db = self.static_db.lock().unwrap();
        db.parse_ntriples_and_add(data);
        db.get_or_build_stats();
        db.build_all_indexes();
    }

    /// Get information about configured windows
    pub fn get_window_info(&self) -> Vec<&RSPWindow> {
        self.window_configs.iter().collect()
    }

    /// Get the RSP-QL query plan information
    pub fn get_query_plan(&self) -> &RSPQueryPlan {
        &self.rsp_query_plan
    }

    /// Return the stream IRIs registered across all configured windows.
    pub fn stream_iris(&self) -> Vec<String> {
        self.window_configs
            .iter()
            .map(|w| w.stream_iri.clone())
            .collect()
    }
}

/// Join all window results, optionally apply the static-data join, apply the R2S operator,
/// and call `consumer` for each output binding.
/// Called from both the coordinator thread and the SingleThread processor.
fn emit_results<O>(
    last_materialized: &HashMap<String, Vec<HashMap<String, String>>>,
    static_data_plan: &Option<PhysicalOperator>,
    static_db: &Arc<Mutex<SparqlDatabase>>,
    r2s: &Arc<Mutex<Relation2StreamOperator<O>>>,
    ts: usize,
    consumer: &AggregateConsumer<O>,
) where
    O: 'static + Clone + Hash + Eq + From<Vec<(String, String)>>,
{
    let joined = join_window_results(last_materialized);

    let final_results = if let Some(ref plan) = static_data_plan {
        let static_bindings = execute_plan_as_bindings(static_db, plan);
        debug!("emit_results: static bindings = {}", static_bindings.len());
        natural_join(&joined, &static_bindings)
    } else {
        joined
    };

    debug!(
        "emit_results: emitting {} bindings before R2S filter",
        final_results.len()
    );
    let outputs: Vec<O> = final_results
        .into_iter()
        .map(|b| {
            let mut kv: Vec<(String, String)> = b.into_iter().collect();
            kv.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            kv.into()
        })
        .collect();

    // This performs the r2s filter!
    // TODO code is weird. Put this into the decorator instead.
    let filtered = r2s.lock().unwrap().eval(outputs, ts);
    consumer(filtered, ts);
}

/// Natural join of two binding sets: compatible bindings are merged, incompatible ones are dropped.
/// Produces the Cartesian product when the two sets share no variables.
fn natural_join(
    left: &[HashMap<String, String>],
    right: &[HashMap<String, String>],
) -> Vec<HashMap<String, String>> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();

    for left_binding in left {
        for right_binding in right {
            // Check compatibility: shared variables must agree on value
            let mut compatible = true;
            for (var, val) in left_binding {
                if let Some(right_val) = right_binding.get(var) {
                    if val != right_val {
                        compatible = false;
                        break;
                    }
                }
            }

            if compatible {
                let mut merged = left_binding.clone();
                for (k, v) in right_binding {
                    merged.insert(k.clone(), v.clone());
                }
                result.push(merged);
            }
        }
    }

    result
}

/// Join results from multiple windows using natural join semantics.
fn join_window_results(
    window_buffers: &HashMap<String, Vec<HashMap<String, String>>>,
) -> Vec<HashMap<String, String>> {
    if window_buffers.is_empty() {
        return Vec::new();
    }

    let mut all_windows: Vec<Vec<HashMap<String, String>>> =
        window_buffers.values().cloned().collect();

    if all_windows.len() == 1 {
        return all_windows.into_iter().next().unwrap();
    }

    // Iteratively natural-join all window result sets
    let mut joined = all_windows.remove(0);
    for window_results in all_windows {
        joined = natural_join(&joined, &window_results);
    }

    joined
}

/// Execute a physical plan against the static-data `SparqlDatabase` and return the results as
/// a list of variable-binding maps.
fn execute_plan_as_bindings(
    static_db: &Arc<Mutex<SparqlDatabase>>,
    plan: &PhysicalOperator,
) -> Vec<HashMap<String, String>> {
    let mut db = static_db.lock().unwrap();
    ExecutionEngine::execute(plan, &mut *db)
}
