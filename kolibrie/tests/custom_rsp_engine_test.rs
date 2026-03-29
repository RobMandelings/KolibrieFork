use std::sync::{Arc, Mutex};
use kolibrie::rsp_engine::{OperationMode, QueryExecutionMode, RSPBuilder, RSPEngine, ResultConsumer, SimpleR2R};
use shared::triple::Triple;
//
fn init_logger() {
    use env_logger::Builder;
    use log::LevelFilter;

    let mut builder = Builder::from_default_env();
    builder
        .is_test(true)
        .filter_level(LevelFilter::Debug) // or Info/Warn/Error
        .init();
}


#[test]
fn rsp_ql_istream_semantics() {

    init_logger();

    let result_container = Arc::new(Mutex::new(Vec::<Vec<(String, String)>>::new()));
    let rc = Arc::clone(&result_container);

    // What is eventually called after the processing
    let result_consumer = ResultConsumer {
        function: Arc::new(move |r: Vec<(String, String)>| {
            println!("Result arrived:");
            for (var, val) in &r {
                println!("  {} = {}", var, val);
            }

            rc.lock().unwrap().push(r);
        }),
    };
    let r2r = Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano));

    let query = r#"
        REGISTER ISTREAM <http://out/stream> AS
        SELECT *
        FROM NAMED WINDOW :w ON ?stream [RANGE 3 STEP 1]
        WHERE { WINDOW :w { ?s a <http://test/IType> . } }
    "#;

    let mut engine: RSPEngine<Triple, Vec<(String, String)>> = RSPBuilder::new()
        .add_rsp_ql_query(query)
        .add_consumer(result_consumer)
        .add_r2r(r2r)
        .set_operation_mode(OperationMode::SingleThread)
        .build()
        .expect("Failed to build ISTREAM engine");

    // Prime dictionary so query and data share term IDs.
    // engine.parse_data("<http://test/s0> a <http://test/IType> .");

    // ts=1: A → no fire (opens first window).
    for t in engine.parse_data("<http://test/subjectA> a <http://test/IType> .") {
        engine.add(t, 1);
    }

    // ts=2: B → fires [-1,1] with {A}; ISTREAM: old=∅ → emit A.
    for t in engine.parse_data("<http://test/subjectB> a <http://test/IType> .") {
        engine.add(t, 2);
    }

    // ts=3: C → fires [0,2] with {A,B}; ISTREAM: old={A} → emit B only.
    for t in engine.parse_data("<http://test/subjectC> a <http://test/IType> .") {
        engine.add(t, 3);
    }

    // ts=4: D → fires [1,3] with {A,B,C}; ISTREAM: old={A,B} → emit C only.
    for t in engine.parse_data("<http://test/subjectD> a <http://test/IType> .") {
        engine.add(t, 4);
    }

    let results = result_container.lock().unwrap();
    assert_eq!(
        results.len(),
        3,
        "ISTREAM: 3 firings → 3 consumer calls. Got: {:?}",
        *results
    );
    // Firing 1: [-1,1] → {A}, new since ∅ → emit A.
    assert_eq!(results[0].len(),1);
    assert!(
        results[0].iter().any(|(k, v)| k == "s" && v.contains("subjectA")),
        "ISTREAM firing 1 must emit subjectA, got: {:?}",
        results[0]
    );
    // Firing 2: [0,2] → {A,B}, new since {A} → emit B only.
    assert_eq!(results[1].len(),1);
    assert!(
        results[1].iter().any(|(k, v)| k == "s" && v.contains("subjectB")),
        "ISTREAM firing 2 must emit subjectB, got: {:?}",
        results[1]
    );
    // Firing 3: [1,3] → {A,B,C}, new since {A,B} → emit C only.
    assert_eq!(results[2].len(),1);
    assert!(
        results[2].iter().any(|(k, v)| k == "s" && v.contains("subjectC")),
        "ISTREAM firing 3 must emit subjectC, got: {:?}",
        results[2]
    );
}