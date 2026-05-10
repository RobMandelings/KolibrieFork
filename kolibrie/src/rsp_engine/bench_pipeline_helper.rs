use std::path::{Path, PathBuf};
use crate::rsp_engine::csv_graph_iter2::build_stream_iter;
use crate::rsp_engine::parking_mapper::traffic_mapper;
use crate::rsp_engine::{OperationMode, QueryExecutionMode, RSPBuilder, RSPEngine, SimpleR2R};
use prototypes::{SliceStrategy, WindowSnapshotStrategy};
use prototypes::prototype::event::Time;
use prototypes::workloads::Workload;
use shared::triple::Triple;

pub type CachedGraphs = Vec<(String, Vec<String>, u64)>;

// Expire strategy is simply placeholder because we don't actually use it
fn make_legacy_q1_engine(query: &str) -> RSPEngine<Triple, Vec<(String, String)>, SliceStrategy<Triple>>
{
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    RSPBuilder::new()
        .set_print_output(false)
        .set_legacy_window(true)
        .add_rsp_ql_query(query)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build RSTREAM engine")
}

fn make_q1_engine<S>(query: &str) -> RSPEngine<Triple, Vec<(String, String)>, S>
where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    RSPBuilder::new()
        .set_print_output(false)
        .set_legacy_window(false)
        .add_rsp_ql_query(query)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build RSTREAM engine")
}

pub fn preload_city_q1(workload: &Workload, stream_path: &PathBuf) -> CachedGraphs {
    let mut streams = vec![
        build_stream_iter(stream_path, traffic_mapper)
            .expect("failed to build first stream")
    ];

    let mut cached: CachedGraphs = Vec::new();

    for i in 0..workload.nr_events {
        let ts = (i as Time) * workload.stream_config.spread + workload.stream_config.offset;

        for stream in &mut streams {
            let batch = stream
                .iter
                .next()
                .expect("Expected non-exhausted stream");

            let graphs: Vec<String> = batch.into_iter().collect();

            cached.push((stream.stream_iri.clone(), graphs, ts));
        }
    }

    cached
}

pub fn run_full_pipeline_legacy(cached_graphs: &CachedGraphs, query: &str)
{
    let mut engine: RSPEngine<Triple, Vec<(String, String)>, SliceStrategy<Triple>> = make_legacy_q1_engine(query);
    for (stream_iri, graphs, ts) in cached_graphs {
        for graph_str in graphs {
            engine.add_graph_to_stream(stream_iri, graph_str, *ts);
        }
    }
}

pub fn run_full_pipeline<S>(cached_graphs: &CachedGraphs, query: &str)
where
    S: WindowSnapshotStrategy<Triple> + 'static,
{
    let mut engine: RSPEngine<Triple, Vec<(String, String)>, S> = make_q1_engine::<S>(query);

    for (stream_iri, graphs, ts) in cached_graphs {
        for graph_str in graphs {
            engine.add_graph_to_stream(stream_iri, graph_str, *ts);
        }
    }
}