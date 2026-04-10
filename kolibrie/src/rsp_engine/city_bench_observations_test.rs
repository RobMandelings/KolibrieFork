use std::env;
use std::sync::{Arc, Mutex};
use log::{debug, LevelFilter};
use prototypes::ExpireStrategy;
use shared::triple::Triple;
use crate::csv_graph_iter::CsvGraphIter;
use crate::rsp::builder::Consumer;
use crate::rsp_engine::helpers::{create_aggregate_consumer, init_logger, SolMap};
use crate::rsp_engine::{OperationMode, QueryExecutionMode, RSPBuilder, RSPEngine, SimpleR2R};

fn print_observations_consumer() -> Consumer<Vec<(String, String)>> {
    create_aggregate_consumer(
        Arc::new(move |results: Vec<SolMap>, _ts| {
            // println!("{:?} ({})", results, results.len());

            let mut obs: Vec<String> = results
                .into_iter()
                .filter_map(|m| m.get("a").cloned())
                .filter_map(|iri| {
                    // expects "http://parking.example/observation/2"
                    iri.rsplit('/').next().map(|id| id.to_string())
                })
                .collect();

            // sort by numeric part
            obs.sort_by_key(|s| s.trim_start_matches('X').trim_start_matches('_').parse::<u64>().ok());

            let out = if obs.is_empty() {
                "Observations: (none)".to_string()
            } else {
                format!("Observations: {}", obs.join(", "))
            };

            println!("{out}");
        })
    )
}

#[test]
fn rsp_ql_city_bench() {

    // init_logger(LevelFilter::Info);
    let agg_consumer = print_observations_consumer();
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    let query = r#"
        REGISTER ISTREAM <http://out/stream> AS
        SELECT *
        FROM NAMED WINDOW :w ON ?stream [RANGE 2 STEP 1]
        WHERE { WINDOW :w { ?a <http://example.org/ontology/vehicleCount> ?b . } }
    "#;

    let mut engine: RSPEngine<Triple, Vec<(String, String)>, ExpireStrategy<Triple>> = RSPBuilder::new()
        .set_legacy_window(false)
        .add_rsp_ql_query(query)
        .set_consumer(agg_consumer)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build ISTREAM engine");

    let path = env::current_dir().expect("Expected path");
    println!("cwd: {}", path.display());

    let iter = CsvGraphIter::from_path("streams/AarhusParkingData.stream").unwrap();
    CsvGraphIter::export_n3("streams/AarhusParkingData.stream", "output_file.n3", 5).expect("TODO: panic message");

    // TODO bottleneck file reading, make sure to not measure this by accident
    let mut i = 1;
    for graph_result in iter.take(50) {
        let graph = graph_result.unwrap();
        let triples = engine.parse_data(&graph);
        debug!("Nr triples: {}", triples.len());

        for triple in engine.parse_data(&graph) {
            // choose a timestamp here, e.g. parsed from streamtime/updatetime
            engine.custom_add(triple, i);
        }
        i += 1;
    }
}