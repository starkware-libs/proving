use std::path::Path;
use std::process::Command;

#[cfg(feature = "slow-tests")]
use circuit_params::assert_committed_registry_is_up_to_date;
use stwo_cairo_utils::file_utils::repo_root;

/// A registry to be generated during tests. Paths are repo-root-relative.
struct RegistryTestFixture {
    /// Path to the definition JSON.
    definition_path: &'static str,
    /// Path to the generated registry.
    #[cfg(feature = "slow-tests")]
    committed_path: &'static str,
}

/// The registry the recursive tree's golden e2e (`test_golden_four_leaves_e2e`) proves with; its
/// leaves attest to the leaf simple bootloader, and its circuits are padded to the production
/// shape (so the goldens' root proof is what the Cairo circuit verifier consumes). Its committed
/// definition is the same one the registry upload generates from.
const RECURSIVE_TREE: RegistryTestFixture = RegistryTestFixture {
    definition_path: "circuit_registry_definitions/canonical_small/definition.json",
    #[cfg(feature = "slow-tests")]
    committed_path: "crates/stwo_run_and_prove_recursive_tree/test_data/circuit_registry.json",
};

/// A registry over the leaf prover's own test data; only its committed registry is checked, by a
/// slow test.
#[cfg(feature = "slow-tests")]
const LEAF_PROVER: RegistryTestFixture = RegistryTestFixture {
    definition_path: "crates/leaf_prover/tests/data/circuit_registry_definition_canonical_small.\
                      json",
    committed_path: "crates/leaf_prover/tests/data/circuit_registry_canonical_small.json",
};

/// Runs the `circuit-params` binary over `fixture`'s definition, asserting success; `as_registry`
/// passes `--registry`.
///
/// Returns the `--output-path` file's contents when `output_path` is given, the binary's stdout
/// otherwise (`run_binary` mixes tracing into stdout, so parsed output must go through a file).
fn run(definition_path: &str, as_registry: bool, output_path: Option<&Path>) -> String {
    let binary = env!("CARGO_BIN_EXE_circuit-params");
    let mut command = Command::new(binary);
    // The definitions' paths are repo-root-relative.
    command.current_dir(repo_root()).arg("--definition").arg(definition_path);
    if as_registry {
        command.arg("--registry");
    }
    if let Some(path) = output_path {
        command.arg("--output-path").arg(path);
    }
    let output = command.output().expect("Cannot run circuit-params");

    assert!(
        output.status.success(),
        "circuit-params failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    match output_path {
        Some(path) => std::fs::read_to_string(path).expect("Cannot read the output file"),
        None => String::from_utf8(output.stdout).expect("stdout is not valid UTF-8"),
    }
}

/// Run the human-readable report path; only a successful run is checked.
#[test]
fn run_circuit_params_binary_info() {
    run(RECURSIVE_TREE.definition_path, false, None);
}

// Slow: commits the verified Cairo proofs' preprocessed trace (lifted to trace + blowup).
#[test]
#[cfg(feature = "slow-tests")]
fn test_recursive_tree_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(
        RECURSIVE_TREE.definition_path,
        RECURSIVE_TREE.committed_path,
    );
}

// Slow: as above. This registry is used by `leaf_prover`'s CLI test.
#[test]
#[cfg(feature = "slow-tests")]
fn test_leaf_prover_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(
        LEAF_PROVER.definition_path,
        LEAF_PROVER.committed_path,
    );
}
