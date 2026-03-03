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

use kolibrie::parser::*;
use kolibrie::sparql_database::SparqlDatabase;
use kolibrie::rsp_engine::{RSPBuilder, SimpleR2R, ResultConsumer, QueryExecutionMode};
use ml::MLHandler;
use ml::generate_ml_models;
use datalog::reasoning::Reasoner;
use shared::triple::Triple;
use shared::rule::Rule;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Setup database with initial ontology
    let mut database = setup_knowledge_base();

    // Define and load reasoning rule
    let rule_query = define_comfort_rule();

    // Materialise / Enrich the database with inferred facts from the given rule
    let (rule, _inferred) = process_rule_definition(&rule_query, &mut database)?;

    // Setup RSP Engine with reasoning
    let result_container = Arc::new(Mutex::new(Vec::new()));

    // Create a new pointer to the same data (clone simply increases count) because the closure in ResultConsumer captures all outer variables by value
    let result_container_clone = result_container.clone();

    // The ResultConsumer receives the final results from the RSP engine pipeline after a window is applied and passed through the pipeline.
    let result_consumer = ResultConsumer {
        function: Arc::new(Box::new(move |bindings: Vec<(String, String)>| {
            let mut results = result_container_clone.lock().unwrap();
            results.push(bindings.clone());

            // Print real-time alerts
            let binding_map: HashMap<_, _> = bindings.iter().cloned().collect();
            if let (Some(room), Some(temp)) = (binding_map.get("room"), binding_map.get("temp")) {
                println!("Stream Alert: Room {} at {}", room, temp);
            }
        }))
    };

    // Continuous RSP-QL query: emits an RSTREAM of (?room, ?temp, ?comfort) bindings.
    // Uses a named sliding window :tempWindow over :sensorStream with RANGE 60 and STEP 10,
    // meaning “look back 60 time units” and re-evaluate every 10 time units.
    // Inside the window, match sensors that haveRoom/temperature/comfortLevel triples and output those values.
    let rsp_query = r#"
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

    let mut engine: kolibrie::rsp_engine::RSPEngine<Triple, Vec<(String, String)>> =
        RSPBuilder::new()
            .add_rsp_ql_query(rsp_query)
            .add_consumer(result_consumer)
            .add_r2r(Box::new(SimpleR2R::with_execution_mode(QueryExecutionMode::Volcano)))
            .build()?;

    // Run the combined workflow
    println!("Time | Room    | Temp  | Humidity | Occupancy | ML Predicted | Comfort Level | Action");
    println!("-----|---------|-------|----------|-----------|--------------|---------------|-------");

    run_combined_workflow(&mut engine, &mut database, &rule)?;

    // Stop the engine
    engine.stop();
    thread::sleep(Duration::from_secs(1));

    Ok(())
}

fn setup_ml_model() -> Result<MLHandler, Box<dyn std::error::Error>> {
    let model_dir = {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            let ml_dir = path.join("ml");
            if ml_dir.exists() && ml_dir.is_dir() {
                break ml_dir.join("examples").join("models");
            }
            if !path.pop() {
                break std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("models");
            }
        }
    };

    std::fs::create_dir_all(&model_dir)?;

    // Check if models exist
    let models_exist = std::fs::read_dir(&model_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.is_file() && path.extension().map_or(false, |ext| ext == "pkl")
        })
        .count() >= 3;

    if !models_exist {
        generate_ml_models(&model_dir, "predictor.py")?;
    }

    let mut ml_handler = MLHandler::new()?;
    let model_ids = ml_handler.discover_and_load_models(&model_dir, "predictor")?;

    println!("Loaded {} ML models", model_ids.len());
    println!("Selected best model: {}", ml_handler.best_model.as_ref().unwrap_or(&"unknown".to_string()));

    Ok(ml_handler)
}

fn setup_knowledge_base() -> SparqlDatabase {
    let mut database = SparqlDatabase::new();

    // Register prefixes
    database.prefixes.insert("ex".to_string(), "http://example.org/".to_string());
    database.prefixes.insert("rdf".to_string(), "http://www.w3.org/1999/02/22-rdf-syntax-ns#".to_string());

    // Add initial ontology - room definitions
    database.add_triple_parts("http://example.org/Office1", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Room");
    database.add_triple_parts("http://example.org/Office2", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Room");

    database
}

fn define_comfort_rule() -> String {
    r#"PREFIX ex: <http://example.org/>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

RULE :ComfortLevelRule :-
CONSTRUCT {
    ?sensor ex:comfortLevel "uncomfortable" .
}
WHERE {
    ?sensor ex:temperature ?temp .
    FILTER(?temp > 25)
}
    "#.to_string()
}

fn run_combined_workflow(
    engine: &mut kolibrie::rsp_engine::RSPEngine<Triple, Vec<(String, String)>>,
    database: &mut SparqlDatabase,
    comfort_rule: &Rule,
) -> Result<(), Box<dyn std::error::Error>> {

    let rooms = vec!["Office1", "Office2"];

    for time in 0..8 {
        for (room_idx, room) in rooms.iter().enumerate() {
            let base_temp = 20.0 + (time as f64 * 1.5) + (room_idx as f64 * 2.0);
            let temp = base_temp + (time as f64 * 0.5);
            let humidity = 50.0 + (time as f64 * 2.0);
            let occupancy = 5 + time + room_idx;

            // ML Prediction
            let input_data = vec![vec![temp, humidity, occupancy as f64]];

            // Create sensor and room URIs
            let sensor_uri = format!("http://example.org/Sensor_{}", room);
            let room_uri = format!("http://example.org/{}", room);

            // Add triples. These triples provide information about the sensor
            database.add_triple_parts(&sensor_uri, "http://example.org/hasRoom", &room_uri);
            database.add_triple_parts(&sensor_uri, "http://example.org/temperature", &temp.to_string());
            database.add_triple_parts(&sensor_uri, "http://example.org/humidity", &humidity.to_string());
            database.add_triple_parts(&sensor_uri, "http://example.org/occupancy", &occupancy.to_string());

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

            // Create a NEW reasoner with its own dictionary
            let mut reasoner = Reasoner::new();

            // Decode all triples FIRST with proper scoping
            let decoded_triples: Vec<(String, String, String)> = {
                let dict = database.dictionary.read().unwrap();
                database.triples.iter()
                    .filter_map(|triple| {
                        let s = dict.decode(triple.subject)?.to_string();
                        let p = dict.decode(triple.predicate)?.to_string();
                        let o = dict.decode(triple.object)?.to_string();
                        Some((s, p, o))
                    })
                    .collect()
            };

            // Now add to reasoner (reasoner has its own dictionary)
            for (s, p, o) in decoded_triples {
                reasoner.add_abox_triple(&s, &p, &o);
            }

            // Add the rule
            reasoner.add_rule(comfort_rule.clone());

            // Perform inference
            let inferred_facts = reasoner.infer_new_facts_semi_naive();

            // Add inferred facts back with proper encoding and error handling
            for fact in &inferred_facts {
                let decoded = {
                    let reasoner_dict = reasoner.dictionary.read().unwrap();
                    if let (Some(s), Some(p), Some(o)) = (
                        reasoner_dict.decode(fact.subject),
                        reasoner_dict.decode(fact.predicate),
                        reasoner_dict.decode(fact.object)
                    ) {
                        Some((s.to_string(), p.to_string(), o.to_string()))
                    } else {
                        None
                    }
                }; // Reasoner dict lock dropped

                if let Some((s, p, o)) = decoded {
                    database.add_triple_parts(&s, &p, &o);
                }
            }

            // Query for the inferred comfort level
            let comfort_level = query_comfort_level(database, &sensor_uri);

            println!(
                "{:4} | {:7} | {:5.1} | {:8.1} | {:9} | {:13}",
                time, room, temp, humidity, occupancy, comfort_level
            );
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

// Helper function to query the inferred comfort level
fn query_comfort_level(database: &SparqlDatabase, sensor_uri: &str) -> String {
    // Proper lock scoping
    let (comfort_pred_id, sensor_id) = {
        let dict = database.dictionary.read().unwrap();
        let comfort = dict.string_to_id.get("http://example.org/comfortLevel").copied();
        let sensor = dict.string_to_id.get(sensor_uri).copied();
        (comfort, sensor)
    };

    if let (Some(comfort_pred_id), Some(sensor_id)) = (comfort_pred_id, sensor_id) {
        if let Some(triple) = database.triples.iter()
            .find(|t| t.subject == sensor_id && t.predicate == comfort_pred_id)
        {
            let dict = database.dictionary.read().unwrap();
            if let Some(value) = dict.decode(triple.object) {
                return value.to_string();
            }
        }
    }

    "comfortable".to_string()
}