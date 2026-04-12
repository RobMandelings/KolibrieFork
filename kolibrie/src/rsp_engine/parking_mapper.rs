use csv::StringRecord;
use crate::rsp_engine::csv_graph_iter2::{escape_literal, RecordMapper};

pub fn parking_mapper(record: &StringRecord) -> Result<String, csv::Error> {
    let vehiclecount = record.get(0).unwrap_or("").trim();
    let updatetime   = record.get(1).unwrap_or("").trim();
    let id           = record.get(2).unwrap_or("").trim();
    let totalspaces  = record.get(3).unwrap_or("").trim();
    let garagecode   = record.get(4).unwrap_or("").trim();
    let streamtime   = record.get(5).unwrap_or("").trim();

    let subject = format!("<http://parking.example/observation/{}>", id);
    let graph = format!(
        concat!(
        "{s} a <http://example.org/ontology/ParkingObservation> .\n",
        "{s} <http://example.org/ontology/vehicleCount> \"{vehiclecount}\" .\n",
        "{s} <http://example.org/ontology/updateTime> \"{updatetime}\" .\n",
        "{s} <http://example.org/ontology/totalSpaces> \"{totalspaces}\" .\n",
        "{s} <http://example.org/ontology/garageCode> \"{garagecode}\" .\n",
        "{s} <http://example.org/ontology/streamTime> \"{streamtime}\" ."
        ),
        s            = subject,
        vehiclecount = escape_literal(vehiclecount),
        updatetime   = escape_literal(updatetime),
        totalspaces  = escape_literal(totalspaces),
        garagecode   = escape_literal(garagecode),
        streamtime   = escape_literal(streamtime),
    );

    Ok(graph)
}

pub fn traffic_mapper(
    sensor_iri: &str,
) -> RecordMapper {
    let sensor_iri = sensor_iri.to_owned();

    let closure = move |record: &StringRecord| {
        let status            = record.get(0).unwrap_or("").trim();
        let avg_measured_time = record.get(1).unwrap_or("").trim();
        let avg_speed         = record.get(2).unwrap_or("").trim();
        let ext_id            = record.get(3).unwrap_or("").trim();
        let median_meas_time  = record.get(4).unwrap_or("").trim();
        let timestamp         = record.get(5).unwrap_or("").trim();
        let vehicle_count     = record.get(6).unwrap_or("").trim();
        let id                = record.get(7).unwrap_or("").trim();
        let report_id         = record.get(8).unwrap_or("").trim();

        let ob_iri = format!(
            "<http://www.insight-centre.org/dataset/SampleEventService#obs-{id}>",
            id = escape_literal(id),
        );

        // Constant property IRI used in your query as ?p1 / ?p2
        let congestion_property =
            "<http://www.insight-centre.org/citytraffic#congestionLevel>".to_string();

        let graph = format!(
            concat!(
            "{s} a <http://purl.oclc.org/NET/ssnx/ssn#Observation> .\n",
            "{s} <http://purl.oclc.org/NET/ssnx/ssn#observedBy> <{sensor}> .\n",
            "{s} <http://purl.oclc.org/NET/ssnx/ssn#observedProperty> {prop} .\n",
            "{s} <http://purl.oclc.org/NET/sao/hasValue> \"{vehicle_count}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
            "{prop} a <http://www.insight-centre.org/citytraffic#CongestionLevel> .\n",
            "{s} <http://www.insight-centre.org/citytraffic#status> \"{status}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#avgMeasuredTime> \"{avg_measured_time}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#avgSpeed> \"{avg_speed}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#extID> \"{ext_id}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#medianMeasuredTime> \"{median_meas_time}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#timestamp> \"{timestamp}\" .\n",
            "{s} <http://www.insight-centre.org/citytraffic#reportID> \"{report_id}\" ."
            ),
            s = ob_iri,
            sensor = sensor_iri,
            prop = congestion_property,
            vehicle_count = escape_literal(vehicle_count),
            status = escape_literal(status),
            avg_measured_time = escape_literal(avg_measured_time),
            avg_speed = escape_literal(avg_speed),
            ext_id = escape_literal(ext_id),
            median_meas_time = escape_literal(median_meas_time),
            timestamp = escape_literal(timestamp),
            report_id = escape_literal(report_id),
        );

        Ok(graph)
    };

    Box::new(closure)
}