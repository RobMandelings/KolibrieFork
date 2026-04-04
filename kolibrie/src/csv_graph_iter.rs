use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use csv::{Reader, StringRecord, StringRecordsIntoIter};

pub struct CsvGraphIter<R: Read> {
    records: StringRecordsIntoIter<R>,
}

impl CsvGraphIter<File> {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, csv::Error> {
        let file = File::open(path).map_err(csv::Error::from)?;
        let rdr = Reader::from_reader(file);
        Ok(Self {
            records: rdr.into_records(),
        })
    }

    pub fn export_n3<PIn, POut>(
        input_path: PIn,
        output_path: POut,
        num_rows: usize,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        PIn: AsRef<Path>,
        POut: AsRef<Path>,
    {
        let iter = Self::from_path(input_path)?;
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        for graph_result in iter.take(num_rows) {
            let graph = graph_result?;
            writer.write_all(graph.as_bytes())?;
            writer.write_all(b"\n\n")?;
        }

        writer.flush()?;
        Ok(())
    }
}

impl<R: Read> CsvGraphIter<R> {
    pub fn from_reader(reader: R) -> Self {
        let rdr = Reader::from_reader(reader);
        Self {
            records: rdr.into_records(),
        }
    }

    fn record_to_graph(record: &StringRecord) -> Result<String, csv::Error> {
        let vehiclecount = record.get(0).unwrap_or("").trim();
        let updatetime = record.get(1).unwrap_or("").trim();
        let id = record.get(2).unwrap_or("").trim();
        let totalspaces = record.get(3).unwrap_or("").trim();
        let garagecode = record.get(4).unwrap_or("").trim();
        let streamtime = record.get(5).unwrap_or("").trim();

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
            s = subject,
            vehiclecount = escape_literal(vehiclecount),
            updatetime = escape_literal(updatetime),
            totalspaces = escape_literal(totalspaces),
            garagecode = escape_literal(garagecode),
            streamtime = escape_literal(streamtime),
        );

        Ok(graph)
    }
}

impl<R: Read> Iterator for CsvGraphIter<R> {
    type Item = Result<String, csv::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.records.next()? {
            Ok(record) => Some(Self::record_to_graph(&record)),
            Err(err) => Some(Err(err)),
        }
    }
}

fn escape_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}