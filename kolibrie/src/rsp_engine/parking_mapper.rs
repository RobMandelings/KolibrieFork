use csv::StringRecord;
use crate::rsp_engine::csv_graph_iter2::escape_literal;

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

pub fn traffic_mapper(record: &StringRecord) -> Result<String, csv::Error> {
    let status             = record.get(0).unwrap_or("").trim();
    let avg_measured_time  = record.get(1).unwrap_or("").trim();
    let avg_speed          = record.get(2).unwrap_or("").trim();
    let ext_id             = record.get(3).unwrap_or("").trim();
    let median_meas_time   = record.get(4).unwrap_or("").trim();
    let timestamp          = record.get(5).unwrap_or("").trim();
    let vehicle_count      = record.get(6).unwrap_or("").trim();
    let id                 = record.get(7).unwrap_or("").trim();
    let report_id          = record.get(8).unwrap_or("").trim();

    let subject = format!("<http://traffic.example/observation/{}>", id);
    let graph = format!(
        concat!(
        "{s} a <http://example.org/ontology/TrafficObservation> .\n",
        "{s} <http://example.org/ontology/status> \"{status}\" .\n",
        "{s} <http://example.org/ontology/avgMeasuredTime> \"{avg_measured_time}\" .\n",
        "{s} <http://example.org/ontology/avgSpeed> \"{avg_speed}\" .\n",
        "{s} <http://example.org/ontology/extID> \"{ext_id}\" .\n",
        "{s} <http://example.org/ontology/medianMeasuredTime> \"{median_meas_time}\" .\n",
        "{s} <http://example.org/ontology/timestamp> \"{timestamp}\" .\n",
        "{s} <http://example.org/ontology/vehicleCount> \"{vehicle_count}\" .\n",
        "{s} <http://example.org/ontology/reportID> \"{report_id}\" ."
        ),
        s                 = subject,
        status            = escape_literal(status),
        avg_measured_time = escape_literal(avg_measured_time),
        avg_speed         = escape_literal(avg_speed),
        ext_id            = escape_literal(ext_id),
        median_meas_time  = escape_literal(median_meas_time),
        timestamp         = escape_literal(timestamp),
        vehicle_count     = escape_literal(vehicle_count),
        report_id         = escape_literal(report_id),
    );

    Ok(graph)
}