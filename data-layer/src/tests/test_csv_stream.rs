// data-layer/src/tests/test_csv_stream.rs
use approx::assert_relative_eq;
use tempfile::NamedTempFile;
use std::io::Write;

use crate::stream::ScalarStream;
use crate::csv_stream::CsvScalarStream;
use crate::stream_queries::{BoundedF64, mean_stream, count_stream};

#[test]
fn csv_stream_reads_column_and_handles_empty_cells() {
    // Prepare a small CSV with delimiter ',' and values in column 1 (0-based)
    let mut tmp = NamedTempFile::new().unwrap();
    // rows: col0, col1
    writeln!(tmp, "id,value").unwrap();        // header-like row (value -> parse error)
    writeln!(tmp, "1,1.5").unwrap();
    writeln!(tmp, "2,").unwrap();              // empty -> skipped
    writeln!(tmp, "3,4.5").unwrap();
    writeln!(tmp, "4,not_a_number").unwrap();  // parse error -> surfaced
    writeln!(tmp, "5,10").unwrap();

    let path = tmp.path().to_str().unwrap().to_string();

    // We’ll iterate manually to observe behavior:
    let mut csv = CsvScalarStream::from_path(&path, 1, b',').unwrap();

    // 1st call: header row "value" -> parse error
    match csv.next_val().unwrap() {
        Ok(_) => panic!("expected parse error from header row"),
        Err(e) => assert!(e.to_string().contains("ParseFloatError") || e.to_string().contains("invalid float")),
    }

    // 2nd: "1.5"
    match csv.next_val().unwrap() {
        Ok(v) => assert_relative_eq!(v, 1.5),
        Err(e) => panic!("unexpected error: {e}"),
    }

    // 3rd: empty cell -> skipped inside implementation; we should see "4.5" next
    match csv.next_val().unwrap() {
        Ok(v) => assert_relative_eq!(v, 4.5),
        Err(e) => panic!("unexpected error: {e}"),
    }

    // 4th: "not_a_number" -> parse error
    assert!(csv.next_val().unwrap().is_err());

    // 5th: "10"
    match csv.next_val().unwrap() {
        Ok(v) => assert_relative_eq!(v, 10.0),
        Err(e) => panic!("unexpected error: {e}"),
    }

    // End
    assert!(csv.next_val().is_none());
}

#[test]
fn csv_with_pipeline_mean() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "id,value").unwrap();
    writeln!(tmp, "1,-100").unwrap();
    writeln!(tmp, "2,-2").unwrap();
    writeln!(tmp, "3,0").unwrap();
    writeln!(tmp, "4,3.5").unwrap();
    writeln!(tmp, "5,200").unwrap();

    let path = tmp.path().to_str().unwrap().to_string();
    let csv = CsvScalarStream::from_path(&path, 1, b',').unwrap();

    // Use mean_stream with clamping [-10,10].
    let dom = BoundedF64::new(-10.0, 10.0);
    let err = mean_stream(csv, dom).unwrap_err();
    assert!(err.to_string().contains("invalid") || err.to_string().contains("ParseFloat"));
    
    let mut tmp2 = NamedTempFile::new().unwrap();
    writeln!(tmp2, "1,-100").unwrap();
    writeln!(tmp2, "2,-2").unwrap();
    writeln!(tmp2, "3,0").unwrap();
    writeln!(tmp2, "4,3.5").unwrap();
    writeln!(tmp2, "5,200").unwrap();
    let path2 = tmp2.path().to_str().unwrap().to_string();

    let csv2 = CsvScalarStream::from_path(&path2, 1, b',').unwrap();
    let (mean2, n2) = mean_stream(csv2, dom).unwrap();

    // Clamped values: [-10, -2, 0, 3.5, 10] => sum = 1.5, mean = 1.5 / 5
    assert_eq!(n2, 5);
    assert_relative_eq!(mean2, 1.5 / 5.0, epsilon = 1e-12);
}

#[test]
fn csv_with_pipeline_count() {
    let mut tmp = NamedTempFile::new().unwrap();
    writeln!(tmp, "v").unwrap();
    writeln!(tmp, "1").unwrap();
    writeln!(tmp, "2").unwrap();
    writeln!(tmp, "3").unwrap();

    let path = tmp.path().to_str().unwrap().to_string();
    let csv = CsvScalarStream::from_path(&path, 0, b',').unwrap();

    // count_stream will error on the header "v".
    // For a header-aware adapter we’d skip it; here we just check that an error surfaces.
    let res = count_stream(csv);
    assert!(res.is_err());

    // Now without header:
    let mut tmp2 = NamedTempFile::new().unwrap();
    writeln!(tmp2, "1").unwrap();
    writeln!(tmp2, "2").unwrap();
    writeln!(tmp2, "3").unwrap();
    let path2 = tmp2.path().to_str().unwrap().to_string();

    let csv2 = CsvScalarStream::from_path(&path2, 0, b',').unwrap();
    let n = count_stream(csv2).unwrap();
    assert_eq!(n, 3);
}