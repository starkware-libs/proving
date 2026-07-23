use itertools::zip_eq;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::SMALL_MAX_SEQUENCE_LOG_SIZE;
use stwo_cairo_prover::witness::prelude::PreProcessedTrace;

use crate::preprocessed_columns::CANONICAL_SMALL_PREPROCESSED_COLUMNS;

#[test]
fn test_canonical_small_preprocessed_columns() {
    let canonical_small_preprocessed_columns = PreProcessedTrace::canonical_small();

    for (column, name) in
        zip_eq(canonical_small_preprocessed_columns.columns, CANONICAL_SMALL_PREPROCESSED_COLUMNS)
    {
        assert_eq!(column.id().id, name);
    }

    let last_seq =
        CANONICAL_SMALL_PREPROCESSED_COLUMNS.iter().rfind(|name| name.starts_with("seq_"));

    let expected = format!("seq_{SMALL_MAX_SEQUENCE_LOG_SIZE}");
    assert_eq!(last_seq, Some(&expected.as_str()));
}
