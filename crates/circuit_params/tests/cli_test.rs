use std::path::Path;
use std::process::Command;

#[cfg(feature = "slow-tests")]
use circuit_params::assert_committed_registry_is_up_to_date;
use circuit_params::{RegistryDefinition, circuits_builder_from_definition};
use stwo_cairo_utils::file_utils::repo_root;

/// A registry to be generated during tests. Paths are repo-root-relative.
struct RegistryTestFixture {
    /// Path to the definition JSON.
    definition_path: &'static str,
    /// Path to the generated registry.
    #[cfg(feature = "slow-tests")]
    committed_path: &'static str,
}

/// The registry the recursive-tree e2e proves with. Its leaf verifiers attest to the leaf simple
/// bootloader, and it pads to the production sizes so the Cairo circuit verifier accepts the e2e's
/// root proofs.
const RECURSIVE_TREE: RegistryTestFixture = RegistryTestFixture {
    definition_path: "circuit_registry_definitions/canonical_small_for_tests/definition.json",
    #[cfg(feature = "slow-tests")]
    committed_path: "crates/stwo_run_and_prove_recursive_tree/test_data/\
                     canonical_small_for_tests_circuit_registry.json",
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

/// A definition whose `opt_n_id_to_big_components` disagrees with its listed `memory_id_to_big`
/// components is rejected.
#[test]
#[should_panic(expected = "but its prover params set opt_n_id_to_big_components to 15")]
fn test_rejects_mismatched_n_id_to_big_components() {
    let mut definition = RegistryDefinition::load(&repo_root(), RECURSIVE_TREE.definition_path);
    definition.cairo_prover_params.opt_n_id_to_big_components = Some(15);
    circuits_builder_from_definition(&definition);
}

/// A definition listing as many `memory_id_to_big` components as its
/// `opt_n_id_to_big_components`, but not the first ones, is rejected.
#[test]
#[should_panic(expected = "but its prover params set opt_n_id_to_big_components to 15")]
fn test_rejects_wrong_id_to_big_components() {
    let mut definition = RegistryDefinition::load(&repo_root(), RECURSIVE_TREE.definition_path);
    definition.cairo_prover_params.opt_n_id_to_big_components = Some(15);
    definition.cairo_components.retain(|name| name != "memory_id_to_big");
    circuits_builder_from_definition(&definition);
}

/// A definition listing its `memory_id_to_big` components in another order is accepted.
#[test]
fn test_accepts_id_to_big_components_in_any_order() {
    let mut definition = RegistryDefinition::load(&repo_root(), RECURSIVE_TREE.definition_path);
    definition.cairo_components.reverse();
    circuits_builder_from_definition(&definition);
}

/// A definition whose `opt_n_id_to_big_components` exceeds the number of `memory_id_to_big`
/// components is rejected.
#[test]
#[should_panic(expected = "but its prover params set opt_n_id_to_big_components to 17")]
fn test_rejects_too_many_id_to_big_components() {
    let mut definition = RegistryDefinition::load(&repo_root(), RECURSIVE_TREE.definition_path);
    definition.cairo_prover_params.opt_n_id_to_big_components = Some(17);
    circuits_builder_from_definition(&definition);
}

/// A definition that doesn't set `opt_n_id_to_big_components` is rejected.
#[test]
#[should_panic(expected = "must set opt_n_id_to_big_components")]
fn test_rejects_missing_n_id_to_big_components() {
    let mut definition = RegistryDefinition::load(&repo_root(), RECURSIVE_TREE.definition_path);
    definition.cairo_prover_params.opt_n_id_to_big_components = None;
    circuits_builder_from_definition(&definition);
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
