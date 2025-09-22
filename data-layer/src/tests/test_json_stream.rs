use approx::assert_relative_eq;
use tempfile::NamedTempFile;
use std::io::Write;

use crate::json_stream::NdjsonScalarStream;
use crate::stream::ScalarStream;
use crate::stream_queries::{BoundedF64, mean_stream};

#[test]
fn ndjson_streams_values_by_key_path() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "{}", r#"{"metrics":{"value": 1.5}}"#).unwrap();
    writeln!(tmp, "{}", "").unwrap(); // blank line
    writeln!(tmp, "{}", r#"{"metrics":{"value": "4.5"}}"#).unwrap(); // numeric string

    let path = tmp.path().to_str().unwrap();
    let mut s = NdjsonScalarStream::from_path(path, "metrics.value").unwrap();

    assert_relative_eq!(s.next_val().unwrap().unwrap(), 1.5);
    assert_relative_eq!(s.next_val().unwrap().unwrap(), 4.5);
    assert!(s.next_val().is_none());
}

#[test]
fn ndjson_missing_key_path_is_error() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "{}", r#"{"foo": 1}"#).unwrap();
    let path = tmp.path().to_str().unwrap();

    let mut s = NdjsonScalarStream::from_path(path, "metrics.value").unwrap();
    let err = s.next_val().unwrap().unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn ndjson_mean_integration_with_clamping() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "{}", r#"{"v": -100}"#).unwrap();
    writeln!(tmp, "{}", r#"{"v": -2}"#).unwrap();
    writeln!(tmp, "{}", r#"{"v": 0}"#).unwrap();
    writeln!(tmp, "{}", r#"{"v": 3.5}"#).unwrap();
    writeln!(tmp, "{}", r#"{"v": 200}"#).unwrap();

    let path = tmp.path().to_str().unwrap();
    let s = NdjsonScalarStream::from_path(path, "v").unwrap();

    let dom = BoundedF64::new(-10.0, 10.0);
    let (mean, n) = mean_stream(s, dom).unwrap();

    assert_eq!(n, 5);
    assert_relative_eq!(mean, 0.3, epsilon = 1e-12);
}