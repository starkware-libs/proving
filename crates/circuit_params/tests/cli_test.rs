use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One registry to exercise: the definition `circuit-params` runs over, and the in-repo path its
/// expected output is committed at (the fix target of the tests below). That path is not part of
/// the definition: the upload publishes to the bucket.
struct RegistryTestFixture {
    /// Repo-root-relative, like the paths inside it.
    definition_path: &'static str,
    /// Read only by the fix-target tests, which are the slow ones.
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
fn run(fixture: &RegistryTestFixture, as_registry: bool, output_path: Option<&Path>) -> String {
    let binary = env!("CARGO_BIN_EXE_circuit-params");
    let mut command = Command::new(binary);
    // The definitions' paths are repo-root-relative.
    command.current_dir(repo_root()).arg("--definition").arg(fixture.definition_path);
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
    run(&RECURSIVE_TREE, false, None);
}

/// Regenerates `fixture`'s registry and asserts it matches the committed one, so a change to the
/// circuits shows up here rather than as a hash mismatch inside a proving binary. Run with `FIX=1`
/// to regenerate.
#[cfg(feature = "slow-tests")]
fn assert_committed_registry_is_up_to_date(fixture: &RegistryTestFixture) {
    let tmp_dir = tempfile::tempdir().expect("Cannot create temporary directory");
    let output_path = tmp_dir.path().join("registry.json");
    let json = run(fixture, true, Some(&output_path));

    let generated: circuit_registry::CircuitRegistry =
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("invalid json: {err}\n{json}"));

    assert_eq!(generated.leaf_verifiers.len(), 1);
    assert_eq!(generated.leaf_verifiers[0].trace_log_size, 20);
    assert_eq!(generated.multiverifiers.len(), 1);
    assert!(generated.circuit_proof_configs.contains_key(&generated.leaf_verifiers[0].config));
    // The circuit hash commits to the config, so distinct circuits must not collide.
    assert_ne!(generated.leaf_verifiers[0].circuit_hash, generated.multiverifiers[0].circuit_hash);

    let committed_path = repo_root().join(fixture.committed_path);
    if std::env::var("FIX").is_ok() {
        std::fs::write(&committed_path, &json)
            .unwrap_or_else(|err| panic!("Cannot write to {}: {err}", committed_path.display()));
        return;
    }
    // Compared as JSON: `CircuitRegistry` holds the prover params, whose type is not `PartialEq`.
    let committed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&committed_path).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).unwrap(),
        committed,
        "{} does not match this run's output; run with FIX=1 to regenerate",
        committed_path.display()
    );
}

// Slow: commits the verified Cairo proofs' preprocessed trace (lifted to trace + blowup).
#[test]
#[cfg(feature = "slow-tests")]
fn test_recursive_tree_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(&RECURSIVE_TREE);
}

// Slow: as above. This registry is used by `leaf_prover`'s CLI test.
#[test]
#[cfg(feature = "slow-tests")]
fn test_leaf_prover_registry_is_up_to_date() {
    assert_committed_registry_is_up_to_date(&LEAF_PROVER);
}
