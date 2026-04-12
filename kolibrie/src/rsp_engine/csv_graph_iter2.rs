use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use csv::{Reader, StringRecord, StringRecordsIntoIter};

/// A function that takes a CSV record and returns an N3 graph serialization.
pub type RecordMapper =
Box<dyn Fn(&StringRecord) -> Result<String, csv::Error> + Send + Sync>;

pub struct CsvGraphIter<R: Read> {
    records: StringRecordsIntoIter<R>,
    mapper: RecordMapper,
}

impl CsvGraphIter<File> {
    pub fn from_path<P>(path: P, mapper: RecordMapper) -> Result<Self, csv::Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).map_err(csv::Error::from)?;
        let rdr = Reader::from_reader(file);
        Ok(Self {
            records: rdr.into_records(),
            mapper,
        })
    }

    pub fn export_n3<PIn, POut>(
        input_path: PIn,
        output_path: POut,
        num_rows: usize,
        mapper: RecordMapper,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        PIn: AsRef<Path>,
        POut: AsRef<Path>,
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
    pub fn from_reader<F>(reader: R, mapper: RecordMapper) -> Self
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

type MapperFactory = fn(&str) -> RecordMapper;

pub struct BuiltStreamIter {
    pub stream_iri: String,
    pub iter: CsvGraphIter<File>,
}

pub fn build_stream_iter(
    stream_name: &str,
    mapper_factory: MapperFactory,
) -> Result<BuiltStreamIter, csv::Error> {
    let stream_iri = format!(":{}", stream_name.to_string());
    let path = format!("streams/{name}.stream", name = stream_name);
    let mapper: RecordMapper = mapper_factory(&stream_name);
    let iter = CsvGraphIter::from_path(path, mapper)?;
    Ok(BuiltStreamIter { stream_iri, iter })
}