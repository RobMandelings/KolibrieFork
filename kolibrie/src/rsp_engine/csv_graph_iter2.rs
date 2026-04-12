use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use csv::{Reader, StringRecord, StringRecordsIntoIter};

/// A function that takes a CSV record and returns an N3 graph serialization.
pub type RecordMapper =
dyn Fn(&StringRecord) -> Result<String, csv::Error> + Send + Sync;

pub struct CsvGraphIter<R: Read> {
    records: StringRecordsIntoIter<R>,
    mapper: Box<RecordMapper>,
}

impl CsvGraphIter<File> {
    pub fn from_path<P, F>(path: P, mapper: F) -> Result<Self, csv::Error>
    where
        P: AsRef<Path>,
        F: Fn(&StringRecord) -> Result<String, csv::Error> + Send + Sync + 'static,
    {
        let file = File::open(path).map_err(csv::Error::from)?;
        let rdr = Reader::from_reader(file);
        Ok(Self {
            records: rdr.into_records(),
            mapper: Box::new(mapper),
        })
    }

    pub fn export_n3<PIn, POut, F>(
        input_path: PIn,
        output_path: POut,
        num_rows: usize,
        mapper: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        PIn: AsRef<Path>,
        POut: AsRef<Path>,
        F: Fn(&StringRecord) -> Result<String, csv::Error> + Send + Sync + 'static,
    {
        let iter = Self::from_path(input_path, mapper)?;
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
    pub fn from_reader<F>(reader: R, mapper: F) -> Self
    where
        F: Fn(&StringRecord) -> Result<String, csv::Error> + Send + Sync + 'static,
    {
        let rdr = Reader::from_reader(reader);
        Self {
            records: rdr.into_records(),
            mapper: Box::new(mapper),
        }
    }
}

impl<R: Read> Iterator for CsvGraphIter<R> {
    type Item = Result<String, csv::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mapper = &self.mapper;
        match self.records.next()? {
            Ok(record) => Some(mapper(&record)),
            Err(err) => Some(Err(err)),
        }
    }
}

pub fn escape_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}