//! Paths into the crate's `test_data/` directory, for the lib tests and the CLI test alike.

use std::path::PathBuf;

/// The committed canonical_small_for_tests circuit registry the tests use. Output of
/// `circuit_params --registry` (see `circuit_params`' cli test, which regenerates it).
pub fn canonical_small_for_tests_circuit_registry_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data/canonical_small_for_tests_circuit_registry.json")
}
