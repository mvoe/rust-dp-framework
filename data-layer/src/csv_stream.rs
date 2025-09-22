// src/csv_stream.rs
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::stream::ScalarStream;

/// A CSV-backed implementation of `ScalarStream`.
///
/// It reads a CSV file line by line, extracts a specific column,
/// and parses each cell into an `f64`.
pub struct CsvScalarStream {
    reader: Box<dyn BufRead + Send>,
    column: usize,
    delimiter: u8,
}

impl CsvScalarStream {
    /// Creates a new `CsvScalarStream` from a file path.
    ///
    /// # Arguments
    /// * `path` – path to the CSV file.
    /// * `column` – which column to parse (0-based index).
    /// * `delimiter` – delimiter as a single byte (e.g. `b','`).
    pub fn from_path(path: &str, column: usize, delimiter: u8) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        Ok(Self {
            reader: Box::new(BufReader::new(file)),
            column,
            delimiter,
        })
    }
}

impl ScalarStream for CsvScalarStream {
    /// Returns the next value from the CSV column.
    ///
    /// - `Some(Ok(f64))` → successfully parsed value.
    /// - `Some(Err(e))` → error while reading/parsing.
    /// - `None` → end of file reached.
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>> {
        let mut buf = String::new();
        loop {
            buf.clear();
            let n = match self.reader.read_line(&mut buf) {
                Ok(0) => return None,
                Ok(_) => {},
                Err(e) => return Some(Err(Box::new(e))),
            };
            let fields = buf.trim_end_matches('\n').split(self.delimiter as char).collect::<Vec<_>>();
            if self.column >= fields.len() {
                continue;
            }
            let raw = fields[self.column].trim();
            if raw.is_empty() {
                continue;
            }
            match raw.parse::<f64>() {
                Ok(v) => return Some(Ok(v)),
                Err(e) => return Some(Err(Box::new(e))),
            }
        }
    }
}
