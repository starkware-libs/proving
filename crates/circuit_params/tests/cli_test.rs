use std::path::{Path, PathBuf};
use std::process::Command;

/// The trace log size the committed canonical-small family's leaves are proven at.
const TRACE_LOG_SIZE: &str = "20";

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// Runs the `circuit-params` binary, asserting success; `registry` passes `--registry`.
///
/// The family is the committed canonical-small one: the files the recursive tree's golden e2e
/// (`test_golden_four_leaves_e2e`) proves its circuits with, so the hashes match its goldens.
///
/// Returns the `--output-path` file's contents when `output_path` is given, the binary's stdout
/// otherwise (`run_binary` mixes tracing into stdout, so parsed output must go through a file).
fn run(registry: bool, output_path: Option<&Path>) -> String {
    let binary = env!("CARGO_BIN_EXE_circuit-params");
    let mut command = Command::new(binary);
    command
        .arg("--min-trace-log-size")
        .arg(TRACE_LOG_SIZE)
        .arg("--max-trace-log-size")
        .arg(TRACE_LOG_SIZE)
        .arg("--cairo-prover-params-json")
        .arg(crates_dir().join("leaf_prover/tests/data/cairo_prover_params_canonical_small.json"))
        .arg("--circuit-prover-params-json")
        .arg(
            crates_dir()
                .join("stwo_run_and_prove_recursive_tree/test_data/circuit_prover_params.json"),
        )
        .arg("--program")
        .arg(crates_dir().join(
            "stwo_run_and_prove_recursive_tree/test_data/leaf_simple_bootloader_compiled.json",
        ));
    if registry {
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
    run(false, None);
}

// Slow: commits the verified Cairo proofs' preprocessed trace (lifted to trace + blowup).
#[test]
#[cfg(feature = "slow-tests")]
fn run_circuit_params_binary_json() {
    let tmp_dir = tempfile::tempdir().expect("Cannot create temporary directory");
    let output_path = tmp_dir.path().join("registry.json");
    let json = run(true, Some(&output_path));

    let registry: circuit_registry::CircuitRegistry =
        serde_json::from_str(&json).unwrap_or_else(|err| panic!("invalid json: {err}\n{json}"));

    assert_eq!(registry.leaf_verifiers.len(), 1);
    assert_eq!(registry.leaf_verifiers[0].trace_log_size, 20);
    assert_eq!(registry.multiverifiers.len(), 1);
    assert!(registry.circuit_proof_configs.contains_key(&registry.leaf_verifiers[0].config));
    // The circuit hash commits to the config, so distinct circuits must not collide.
    assert_ne!(registry.leaf_verifiers[0].circuit_hash, registry.multiverifiers[0].circuit_hash);
}
