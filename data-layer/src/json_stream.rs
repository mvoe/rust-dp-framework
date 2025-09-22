use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use serde_json::{self as json, Value};
use crate::stream::ScalarStream;

/// Extract a floating-point value (`f64`) from a JSON object
/// by following a dotted key path (e.g., `"metrics.value"`).
///
/// - If the target is a number, it is returned directly.
/// - If the target is a string, the string is parsed as `f64`.
/// - Otherwise, an error is returned.
fn extract_f64_by_path<'a>(v: &'a Value, path: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
    let mut cur = v;
    if !path.is_empty() {
        for key in path.split('.') {
            cur = cur.get(key)
                .ok_or_else(|| format!("key '{}' not found in JSON object", key))?;
        }
    }
    if let Some(n) = cur.as_f64() {
        return Ok(n);
    }
    if let Some(s) = cur.as_str() {
        return s.parse::<f64>()
            .map_err(|e| format!("cannot parse '{}' as f64: {}", s, e).into());
    }
    Err("value is not f64 or numeric string".into())
}

/* ----------------------------- NDJSON (newline-delimited JSON) ----------------------------- */

/// A `ScalarStream` implementation for **NDJSON files** (one JSON object per line).
///
/// Each call to `next_val` reads one line, parses it as JSON,
/// extracts the requested key path, and converts it into an `f64`.
///
/// - Blank lines are skipped.
/// - Errors (e.g., malformed JSON or missing key) are returned
///   as `Some(Err(..))` instead of panicking.
pub struct NdjsonScalarStream {
    reader: Box<dyn BufRead + Send>,
    key_path: String,
    buf: String,
}

impl NdjsonScalarStream {
    /// Creates a new NDJSON-backed scalar stream.
    ///
    /// # Arguments
    /// * `path` – path to the NDJSON file.
    /// * `key_path` – dotted key path to extract from each JSON object.
    pub fn from_path(path: &str, key_path: impl Into<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        Ok(Self {
            reader: Box::new(BufReader::new(file)),
            key_path: key_path.into(),
            buf: String::new(),
        })
    }
}

impl ScalarStream for NdjsonScalarStream {
    /// Reads the next line of the NDJSON file and returns the extracted value.
    ///
    /// - `Some(Ok(f64))` → successfully parsed value.
    /// - `Some(Err(e))` → error while reading/parsing/extracting.
    /// - `None` → end of file reached.
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>> {
        loop {
            self.buf.clear();
            match self.reader.read_line(&mut self.buf) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    let line = self.buf.trim();
                    if line.is_empty() { continue; } // skip blanks
                    match json::from_str::<Value>(line) {
                        Ok(v) => return Some(extract_f64_by_path(&v, &self.key_path)),
                        Err(e) => return Some(Err(Box::new(e))),
                    }
                }
                Err(e) => return Some(Err(Box::new(e))),
            }
        }
    }
}

/* ----------------------------- JSON Array (streaming) ----------------------------- */

/// A `ScalarStream` implementation for a **JSON array** of objects.
///
/// This uses `serde_json::StreamDeserializer` to iterate element by element,
/// without loading the entire array into memory. Suitable for very large JSON
/// arrays when streaming is required.
///
/// # Example file
/// ```json
/// [ {"v": 1.0}, {"v": -2.0}, {"v": "3.5"} ]
/// ```
///
/// With `key_path = "v"`, this stream will yield `1.0`, `-2.0`, and `3.5`.
pub struct JsonArrayScalarStream<R: Read> {
    iter: json::StreamDeserializer<'static, json::de::IoRead<R>, Value>,
    key_path: String,
}

impl JsonArrayScalarStream<File> {
    /// Creates a new JSON-array-backed scalar stream.
    ///
    /// # Arguments
    /// * `path` – path to the JSON file.
    /// * `key_path` – dotted key path to extract from each JSON element.
    pub fn from_path(path: &str, key_path: impl Into<String>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        // Note: if the top-level is not an array, this will treat each
        // top-level JSON value as one item instead.
        let iter = json::Deserializer::from_reader(file).into_iter::<Value>();
        Ok(Self { iter, key_path: key_path.into() })
    }
}

impl<R: Read> ScalarStream for JsonArrayScalarStream<R> {
    /// Reads the next element of the JSON array and returns the extracted value.
    ///
    /// - `Some(Ok(f64))` → successfully parsed value.
    /// - `Some(Err(e))` → error while reading/parsing/extracting.
    /// - `None` → end of array (or file) reached.
    fn next_val(&mut self) -> Option<Result<f64, Box<dyn Error + Send + Sync>>> {
        match self.iter.next() {
            None => None,
            Some(Ok(v)) => Some(extract_f64_by_path(&v, &self.key_path)),
            Some(Err(e)) => Some(Err(Box::new(e))),
        }
    }
}