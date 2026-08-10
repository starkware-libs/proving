use std::path::PathBuf;

use leaf_prover::prove_leaf::prove_leaf_from_files;

fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data")
}

#[test]
#[should_panic(expected = "Cannot get program from")]
fn test_missing_program_panics() {
    prove_leaf_from_files(
        &test_data_dir().join("no_such_program.json"),
        &None,
        &test_data_dir().join("circuit_registry_canonical_small.json"),
    );
}
#[test]
#[should_panic(expected = "Cannot read the circuit registry from")]
fn test_missing_circuit_registry_panics() {
    prove_leaf_from_files(
        &test_data_dir().join("use_all_opcodes_and_builtins_compiled.json"),
        &None,
        &test_data_dir().join("no_such_registry.json"),
    );
}
