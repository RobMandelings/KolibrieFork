use std::env;
use log::debug;
use prototypes::ExpireStrategy;
use shared::triple::Triple;
use crate::rsp_engine::{OperationMode, QueryExecutionMode, RSPBuilder, RSPEngine, SimpleR2R};
use crate::rsp_engine::csv_graph_iter2::CsvGraphIter;
use crate::rsp_engine::parking_mapper::traffic_mapper;

#[test]
fn rsp_ql_city_bench() {
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    // TODO: I don't think specific prefixes are allowed, or are they?
    let query = r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?obId2 ?v1 ?v2
    FROM NAMED WINDOW :w1 ON :AarhusTrafficData182955 [RANGE PT3S STEP PT1S]
    FROM NAMED WINDOW :w2 ON :AarhusTrafficData158505 [RANGE PT3S STEP PT1S]
    WHERE {
      ?p1 a ct:CongestionLevel .
      ?p2 a ct:CongestionLevel .
      WINDOW :w1 {
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy :AarhusTrafficData182955 .
      }

      WINDOW :w2 {
        ?obId2 ssn:observedProperty ?p2 ;
               sao:hasValue ?v2 ;
               ssn:observedBy :AarhusTrafficData158505 .
      }
    }"#;

    let mut engine: RSPEngine<Triple, Vec<(String, String)>, ExpireStrategy<Triple>> = RSPBuilder::new()
        .set_legacy_window(false)
        .add_rsp_ql_query(query)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build RSTREAM engine");

    let path = env::current_dir().expect("Expected path");
    println!("cwd: {}", path.display());

    let iter = CsvGraphIter::from_path("streams/AarhusTrafficData158505.stream", traffic_mapper("AarhusTrafficData158505")).unwrap();
    let iter2 = CsvGraphIter::from_path("streams/AarhusTrafficData182955.stream", traffic_mapper("AarhusTrafficData182955")).unwrap();
    // CsvGraphIter::export_n3("streams/AarhusTrafficData158505.stream", "streams/outputs/output_traffic.n3", 5, traffic_mapper("AarhusTrafficData158505")).expect("TODO: panic message");

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

#[test]
fn city_bench_q1_single_window() {
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    // TODO: I don't think specific prefixes are allowed, or are they?
    let query = r#"
    PREFIX ses: <http://www.insight-centre.org/dataset/SampleEventService#>
    PREFIX ssn: <http://purl.oclc.org/NET/ssnx/ssn#>
    PREFIX sao: <http://purl.oclc.org/NET/sao/>
    PREFIX ct:  <http://www.insight-centre.org/citytraffic#>
    REGISTER RSTREAM <http://out/stream> AS
    SELECT ?obId1 ?v1
    FROM NAMED WINDOW :w1 ON :AarhusTrafficData158505 [RANGE 3 STEP 1]
    WHERE {
      WINDOW :w1 {
        ?obId1 ssn:observedProperty ?p1 ;
               sao:hasValue ?v1 ;
               ssn:observedBy <AarhusTrafficData158505> .
      }
    }"#;

    let mut engine: RSPEngine<Triple, Vec<(String, String)>, ExpireStrategy<Triple>> = RSPBuilder::new()
        .set_legacy_window(false)
        .add_rsp_ql_query(query)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build RSTREAM engine");

    let path = env::current_dir().expect("Expected path");
    println!("cwd: {}", path.display());

    let iter = CsvGraphIter::from_path("streams/AarhusTrafficData158505.stream", traffic_mapper("AarhusTrafficData158505")).unwrap();

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