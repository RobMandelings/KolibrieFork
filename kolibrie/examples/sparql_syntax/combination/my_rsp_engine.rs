/*!
Example of a combined workflow using RSP and ML.

This example demonstrates a “kitchen sink” integration where the same synthetic sensor readings
are fed into multiple, mostly independent workflows: (1) an ML model produces predictions used
for decision printing, (2) a growing in-memory knowledge base is enriched via rule-based
inference using the `Reasoner`, and (3) an RSP-QL query is evaluated over a timestamped stream
using the RSP engine’s sliding window semantics to emit real-time query results. Note that the
RSP engine does not invoke the `Reasoner` during window evaluation; reasoning is performed
separately and any inferred facts are not automatically propagated into the stream/window store.

*/

use kolibrie::rsp_engine::{RSPBuilder, SimpleR2R, ResultConsumer, QueryExecutionMode};
use shared::triple::Triple;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Continuous RSP-QL query: emits an RSTREAM of (?room, ?temp, ?comfort) bindings.
/// Uses a named sliding window :tempWindow over :sensorStream with RANGE 60 and STEP 10,
/// meaning “look back 60 time units” and re-evaluate every 10 time units.
/// Inside the window, match sensors that haveRoom/temperature/comfortLevel triples and output those values.
const RSP_QUERY: &str = r#"
        PREFIX ex: <http://example.org/>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        REGISTER RSTREAM <http://out/comfort> AS
        SELECT ?room ?temp ?comfort
        FROM NAMED WINDOW :tempWindow ON :sensorStream [RANGE 60 STEP 10]
        WHERE {
            WINDOW :tempWindow {
                ?sensor ex:hasRoom ?room ;
                       ex:temperature ?temp ;
                       ex:comfortLevel ?comfort .
            }
        }
    "#;

/// Represents a single solution mapping, represented as a vector of (variable, value) pairs.
type Bindings = Vec<(String, String)>;
type ResultContainer = Arc<Mutex<Vec<Bindings>>>;

/// Creates a result consumer that consumes a single solution mapping.
/// This consumer adds the new solution mapping to the result container
fn create_result_consumer(result_container_clone: ResultContainer) -> ResultConsumer<Bindings> {
    ResultConsumer {

        // Bindings: a single solution mapping
        function: Arc::new(Box::new(move |bindings: Bindings| {
            print!("Consumed!");
            let mut results = result_container_clone.lock().unwrap();

            // Push the bindings to the result container
            results.push(bindings.clone());

            // Print real-time alerts
            let binding_map: HashMap<_, _> = bindings.iter().cloned().collect();
            if let (Some(room), Some(temp)) = (binding_map.get("room"), binding_map.get("temp")) {
                println!("Stream Alert: Room {} at {}", room, temp);
            }
        }))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Setup RSP Engine with reasoning
    let result_container: ResultContainer = Arc::new(Mutex::new(Vec::new()));

    // Create a new pointer to the same data (clone simply increases count) because the closure in ResultConsumer captures all outer variables by value
    let result_container_clone = result_container.clone();

    // The ResultConsumer receives the final results from the RSP engine pipeline after a window is applied and passed through the pipeline.
    let result_consumer = create_result_consumer(result_container_clone);

    let mut engine: kolibrie::rsp_engine::RSPEngine<Triple, Vec<(String, String)>> =
        RSPBuilder::new()
            .add_rsp_ql_query(RSP_QUERY)
            .add_consumer(result_consumer)
            .add_r2r(Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano)))
            .build()?;

    // Run the combined workflow
    println!("Time | Room    | Temp  | Humidity | Occupancy | ML Predicted | Comfort Level | Action");
    println!("-----|---------|-------|----------|-----------|--------------|---------------|-------");

    run_combined_workflow(&mut engine)?;

    // Stop the engine
    engine.stop();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}

fn run_combined_workflow(
    engine: &mut kolibrie::rsp_engine::RSPEngine<Triple, Vec<(String, String)>>,
) -> Result<(), Box<dyn std::error::Error>> {

    let rooms = vec!["Office1", "Office2"];

    for time in 0..8 {
        for (room_idx, room) in rooms.iter().enumerate() {
            let base_temp = 20.0 + (time as f64 * 1.5) + (room_idx as f64 * 2.0);
            let temp = base_temp + (time as f64 * 0.5);
            let humidity = 50.0 + (time as f64 * 2.0);
            let occupancy = 5 + time + room_idx;

            // Create sensor and room URIs
            let sensor_uri = format!("http://example.org/Sensor_{}", room);
            let room_uri = format!("http://example.org/{}", room);

            // Triples represented as raw text; they still need to be parsed into Kolibrie’s internal data structures.
            let triples_data = format!(
                "<{}> <http://example.org/hasRoom> <{}> .
                 <{}> <http://example.org/temperature> \"{}\" .
                 <{}> <http://example.org/humidity> \"{}\" .
                 <{}> <http://example.org/occupancy> \"{}\" .",
                sensor_uri, room_uri,
                sensor_uri, temp,
                sensor_uri, humidity,
                sensor_uri, occupancy
            );

            // Parse the 'raw' triples into the internal data structures
            let triples = engine.parse_data(&triples_data);
            for triple in triples {
                // Add triple to the stream at specific time point
                engine.add_to_stream("sensorStream", triple.clone(), time);
            }

            println!(
                "{:4} | {:7} | {:5.1} | {:8.1} | {:9} ",
                time, room, temp, humidity, occupancy
            );
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}