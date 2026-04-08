/*
* Copyright © 2025 Volodymyr Kadzhaia
* Copyright © 2025 Pieter Bonte
* KU Leuven — Stream Intelligence Lab, Belgium
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* you can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crate::parser::parse_combined_query;
use crate::rsp::r2r::R2ROperator;
use crate::rsp::r2s::StreamOperator;
use crate::rsp::s2r::{ReportStrategy, Tick};
use crate::rsp_engine::{AggregateConsumer, OperationMode, QueryExecutionMode, RSPEngine, RSPQueryPlan, RSPWindow, ResultConsumer};
use crate::sparql_database::SparqlDatabase;
use crate::streamertail_optimizer::{
    build_logical_plan, LogicalOperator, PhysicalOperator, Streamertail,
};
use shared::query::{StreamType, SyncPolicy, WindowBlock, WindowClause};
use shared::rule::Rule;
use shared::terms::Term;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use prototypes::WindowSnapshotStrategy;

/// RSP Query configuration extracted from parsed RSP-QL
#[derive(Debug)]
pub struct RSPQueryConfig<'a> {
    pub windows: Vec<RSPWindow>,
    pub output_stream: String,
    pub stream_type: StreamOperator,
    /// Graph pattern is a set of triple patterns (def. RSP-QL paper)
    pub static_patterns: Vec<(&'a str, &'a str, &'a str)>, // Static graph patterns outside windows
    pub database: SparqlDatabase,                           // used prefixes
    /// Effective synchronization policy for multi-window coordination.
    pub sync_policy: SyncPolicy,
}

pub struct RSPBuilder<'a, I, O, S> {
    _phantom: std::marker::PhantomData<S>,
    rsp_ql_query: Option<&'a str>,
    triples: Option<&'a str>, /// Initial triples to load into R2R store. Still need to be parsed.
    rules: Option<&'a str>,
    result_consumer: Option<ResultConsumer<O>>,
    aggregate_result_consumer: Option<AggregateConsumer<O>>,
    r2r: Option<Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>>,
    operation_mode: OperationMode,
    query_execution_mode: QueryExecutionMode,
    syntax: String, /// Syntax format of the triples to parse (e.g. "ntriples")
    /// Engine-level default policy; overridden by per-window `WITH POLICY` clause.
    sync_policy: SyncPolicy,
    reasoning_rules: Vec<Rule>,
    sparql_rules: Vec<String>,
}

impl<'a, I, O, S> RSPBuilder<'a, I, O, S>
where
    O: Clone + Eq + Send + Debug + Hash + 'static + From<Vec<(String, String)>>,
    I: Eq + PartialEq + Clone + Debug + Hash + Send + 'static,
    S: WindowSnapshotStrategy<I>
{
    pub fn new() -> Self {
        RSPBuilder::<I, O, S> {
            _phantom: Default::default(),
            rsp_ql_query: None,
            triples: None,
            rules: None,
            result_consumer: None,
            aggregate_result_consumer: None,
            r2r: None,
            operation_mode: OperationMode::MultiThread,
            query_execution_mode: QueryExecutionMode::Volcano,
            syntax: "ntriples".to_string(),
            sync_policy: SyncPolicy::default(),
            reasoning_rules: Vec::new(),
            sparql_rules: Vec::new(),
        }
    }

    /// Override the engine-level default synchronization policy.
    /// A per-window `WITH POLICY` clause in the query string takes precedence.
    pub fn set_sync_policy(mut self, policy: SyncPolicy) -> Self {
        self.sync_policy = policy;
        self
    }

    /// Supply pre-parsed reasoning rules (forward-chaining inference applied per window firing).
    pub fn add_reasoning_rules(mut self, rules: Vec<Rule>) -> Self {
        self.reasoning_rules = rules;
        self
    }

    /// Supply SPARQL RULE strings (`RULE :Name :- CONSTRUCT{} WHERE{}`) to be parsed after
    /// dictionary merge in `RSPEngine::new()` so term IDs are consistent.
    pub fn add_sparql_rules(mut self, rules: Vec<String>) -> Self {
        self.sparql_rules = rules;
        self
    }

    /// Add RSP-QL query instead of separate window parameters and SPARQL query
    pub fn add_rsp_ql_query(mut self, query: &'a str) -> Self {
        self.rsp_ql_query = Some(query);
        self
    }

    /// Adds triples (as strings, not parsed yet) to the RSPBuilder
    pub fn add_triples(mut self, triples: &'a str) -> Self {
        self.triples = Some(triples);
        self
    }

    /// Adds rules (as strings, not parsed yet) to the RSPBuilder
    pub fn add_rules(mut self, rules: &'a str) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Add a consumer that will consume the results of reported window content
    pub fn add_consumer(mut self, consumer: ResultConsumer<O>) -> Self {
        self.result_consumer = Some(consumer);
        self
    }

    pub fn add_aggregate_consumer(mut self, consumer: AggregateConsumer<O>) -> Self {
        self.aggregate_result_consumer = Some(consumer);
        self
    }

    pub fn add_r2r(
        mut self,
        r2r: Box<dyn R2ROperator<I, Vec<PhysicalOperator>, O>>,
    ) -> Self {
        self.r2r = Some(r2r);
        self
    }

    pub fn set_operation_mode(mut self, operation_mode: OperationMode) -> Self {
        self.operation_mode = operation_mode;
        self
    }

    pub fn set_query_execution_mode(mut self, mode: QueryExecutionMode) -> Self {
        self.query_execution_mode = mode;
        self
    }

    /// Parse the RSP-QL query (includes window configurations etc)
    /// Result: if ok, the RSPQueryConfig, otherwise error message
    fn parse_rsp_ql_query<'b>(&self, query: &'b str) -> Result<RSPQueryConfig<'b>, String> {
        match parse_combined_query(query) {
            Ok((_, parsed_query)) => {
                if let Some(register_clause) = &parsed_query.register_clause {
                    let mut windows = Vec::new();
                    let mut database = SparqlDatabase::new();
                    database.set_prefixes(parsed_query.prefixes.clone());
                    // Extract windows from the register clause
                    for window_clause in &register_clause.query.window_clause {
                        let window = self.create_rsp_window(
                            window_clause,
                            &register_clause.query.window_blocks,
                            &mut database,
                        )?;
                        windows.push(window);
                    }

                    // Convert stream type
                    let stream_type = match &register_clause.stream_type {
                        StreamType::RStream => StreamOperator::RSTREAM,
                        StreamType::IStream => StreamOperator::ISTREAM,
                        StreamType::DStream => StreamOperator::DSTREAM,
                        StreamType::Custom(_) => StreamOperator::RSTREAM, // Default fallback
                    };

                    // Extract static patterns from WHERE clause (outside window blocks)
                    let static_patterns = register_clause.query.where_clause.0.clone();

                    // Resolve policy: per-window WITH POLICY > builder default
                    let sync_policy = register_clause
                        .query
                        .window_clause
                        .iter()
                        .find_map(|wc| wc.policy.clone())
                        .unwrap_or_else(|| self.sync_policy.clone());

                    Ok(RSPQueryConfig {
                        windows,
                        output_stream: register_clause.output_stream_iri.to_string(),
                        stream_type,
                        static_patterns,
                        database,
                        sync_policy,
                    })
                } else {
                    Err("No REGISTER clause found in RSP-QL query".to_string())
                }
            }
            Err(e) => Err(format!("Failed to parse RSP-QL query: {:?}", e)),
        }
    }

    /// Create RSP window from parsed window clause
    fn create_rsp_window(
        &self,
        window_clause: &WindowClause,
        window_blocks: &[WindowBlock],
        database: &mut SparqlDatabase,
    ) -> Result<RSPWindow, String> {
        // Find the corresponding window block for this window
        let spo_query = LogicalOperator::scan((
            Term::Variable("s".to_string()),
            Term::Variable("p".to_string()),
            Term::Variable("o".to_string()),
        ));
        let window_query = window_blocks
            .iter()
            .find(|block| block.window_name == window_clause.window_iri)
            .map(|block| {
                // Convert window block patterns to query plan
                for (j, (s, p, o)) in block.patterns.iter().enumerate() {
                    println!(" Registering     {}: {} {} {}", j + 1, s, p, o);
                }
                let op = build_logical_plan(
                    Vec::new(),
                    block.patterns.clone(),
                    Vec::new(),
                    &database.prefixes.clone(),
                    database,
                    &[],
                    None,
                );
                println!("\tResults in {:?}", op);
                op
            })
            .unwrap_or_else(|| spo_query);

        // Convert window specification
        let slide = window_clause
            .window_spec
            .slide
            .unwrap_or(window_clause.window_spec.width);

        let tick = match window_clause.window_spec.tick {
            Some("TIME_DRIVEN") => Tick::TimeDriven,
            Some("TUPLE_DRIVEN") => Tick::TupleDriven,
            Some("BATCH_DRIVEN") => Tick::BatchDriven,
            _ => Tick::TimeDriven, // Default
        };

        let report_strategy = match window_clause.window_spec.report_strategy {
            Some("ON_WINDOW_CLOSE") => ReportStrategy::OnWindowClose,
            Some("ON_CONTENT_CHANGE") => ReportStrategy::OnContentChange,
            Some("NON_EMPTY_CONTENT") => ReportStrategy::NonEmptyContent,
            Some("PERIODIC") => ReportStrategy::Periodic(1000), // Default 1 second
            _ => ReportStrategy::OnWindowClose,                  // Default
        };

        Ok(RSPWindow {
            window_iri: window_clause.window_iri.to_string(),
            stream_iri: window_clause.stream_iri.to_string(),
            width: window_clause.window_spec.width,
            slide,
            tick,
            report_strategy,
            query: window_query,
        })
    }

    /// Create RSP-QL query plan using Volcano optimizer
    fn create_rsp_query_plan(query_config: &RSPQueryConfig) -> Result<RSPQueryPlan, String> {
        let mut window_plans = Vec::new();

        // Create individual window plans
        for window in &query_config.windows {
            window_plans.push(window.query.clone());
        }
        let mut database = query_config.database.clone();

        // Create static data plan if there are static patterns
        let static_data_plan = if !query_config.static_patterns.is_empty() {
            let logical_plan = build_logical_plan(
                Vec::new(),
                query_config.static_patterns.clone(),
                Vec::new(),
                &database.prefixes.clone(),
                &mut database,
                &[],
                None,
            );
            Some(logical_plan)
        } else {
            None
        };

        // Create physical plans from the logical ones
        let mut optimizer = Streamertail::new(&database);

        let static_data_plan = match static_data_plan {
            Some(v) => Some(optimizer.find_best_plan(&v)),
            None => None,
        };

        println!("logical window plans {:?}", window_plans);

        let window_plans = window_plans
            .iter()
            .map(|v| optimizer.find_best_plan(v))
            .collect();
        println!("physical window plans {:?}", window_plans);

        Ok(RSPQueryPlan {
            window_plans,
            static_data_plan,
        })
    }

    pub fn build(mut self) -> Result<RSPEngine<I, O, S>, String> {
        let rsp_ql_query = self
            .rsp_ql_query
            .take()
            .ok_or("Please provide RSP-QL query")?;
        let r2r = self.r2r.take().ok_or("Please provide R2R operator!")?;
        let triples = self.triples.take().unwrap_or("");
        let syntax = self.syntax.clone();
        let rules = self.rules.take().unwrap_or("");
        let result_consumer = self.result_consumer.take().unwrap_or(ResultConsumer {
            function: Arc::new(Box::new(|r| println!("Bindings: {:?}", r))),
        });
        let aggregate_result_consumer = self.aggregate_result_consumer.take().unwrap_or(AggregateConsumer {
            function: Arc::new(Box::new(|r, ts| println!("Bindings: {:?}", r))),
        });
        let operation_mode = self.operation_mode;

        // Parse RSP-QL query and return RSPQueryConfig object instead
        let query_config = self.parse_rsp_ql_query(rsp_ql_query)?;

        let sync_policy = query_config.sync_policy.clone();

        // Create RSP-QL query plan using Volcano optimizer
        let rsp_query_plan = Self::create_rsp_query_plan(&query_config)?;

        Ok(RSPEngine::new(
            query_config,
            triples,
            syntax,
            rules,
            result_consumer,
            aggregate_result_consumer,
            r2r,
            operation_mode,
            self.query_execution_mode,
            rsp_query_plan,
            sync_policy,
            self.reasoning_rules,
            self.sparql_rules,
        ))
    }
}
